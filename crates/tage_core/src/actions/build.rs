use tracing::warn;

use crate::{prelude::*, v};

#[derive(Debug, Clone)]
pub struct ActBuild {
    pub this: UnitTarget,
    pub build_id: UnitId,
    pub area: BuildArea,
}

impl Act for ActBuild {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let tile = board.grid.get_at(&pre.at);
        let Some(ref unit) = tile.unit else {
            return vec![];
        };
        if let Some(_) = tile.building {
            return vec![];
        }
        let unit_bp = board.bp.get_unit(&unit.blueprint_id);
        unit_bp
            .build_list
            .iter()
            .map(|idref| idref.unit())
            .map(|id| {
                ActBuild::get_build_placements(board.bp(), board, id, pre.at)
                    .map(move |area| (id, area))
            })
            .flatten()
            .map(|(id, area)| ActBuild {
                this: pre.clone(),
                build_id: id.clone(),
                area: area.clone(),
            })
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        if !self.area.contains(&self.this.at) {
            return false;
        }
        self.is_placement_valid(board.bp(), board)
    }

    fn apply(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let tile = board.grid.get_at(&self.this.at);
        let unit = tile.get_top_unit().unwrap();
        let owner = unit.owner.clone();
        let build_bp = bp.get_unit(&self.build_id);

        let bonus = board.get_player_bonus(&owner, Some(&self.build_id));
        let player = board.get_player_mut(&owner);
        player.resources -= build_bp.resources.cost.apply_cost(bonus);

        for pos in self.area.iter() {
            let linked_units = if self.area.iter().count() == 1 {
                vec![]
            } else {
                self.area.iter().collect()
            };
            board.grid.get_at_mut(&pos).building = Some(Unit {
                blueprint_id: self.build_id.clone(),
                health: 50,
                done: true,
                owner: owner.clone(),
                in_construction: true,
                linked_units,
                ..Default::default()
            });
        }
    }

    fn undo(&self, board: &mut Board) {
        let bp = board.bp.clone();
        for pos in self.area.iter() {
            board.grid.get_at_mut(&pos).building = None;
        }
        let owner = &self.this.unit.owner;
        let bonus = board.get_player_bonus(owner, Some(&self.build_id));
        let player = board.get_player_mut(&owner);
        let build_bp = bp.as_ref().get_unit(&self.build_id);
        player.resources += build_bp.resources.cost.apply_cost(bonus);
    }
}

impl ActBuild {
    fn get_build_placements<'a>(
        bp: &Blueprints,
        board: &'a Board,
        building_id: &UnitId,
        from: IVec2,
    ) -> impl Iterator<Item = BuildArea> + 'a {
        let potential_build = bp.get_unit(building_id);
        let size = IVec2::splat(potential_build.unit_size.size);
        iter_area(size)
            .filter(move |off| {
                board.grid.contains(&(from - *off))
                    && board.grid.contains(&(from + size - *off - v!(1, 1)))
            })
            .map(move |off| BuildArea {
                min: from - off,
                max: from + size - off,
            })
    }

    fn is_placement_valid(&self, bp: &Blueprints, board: &Board) -> bool {
        let potential_build = bp.get_unit(&self.build_id);
        let player = board.get_player(&self.this.unit.owner);

        self.area.iter().all(|from| {
            let tile = board.grid.get_at(&from);
            let terrain_bp = bp.get_terrain(&tile.terrain.blueprint_id);

            let constraints =
                potential_build
                    .build_constraints
                    .iter()
                    .all(|constraint| match constraint {
                        BuildConstraint::IsAdjacentTo(adjacent_id) => {
                            board.grid.get_adjacent(&from).iter().any(|(_dir, t)| {
                                match &t.building {
                                    Some(building) if building.owner == player.id => {
                                        building.blueprint_id == *adjacent_id.unit()
                                    }
                                    _ => false,
                                }
                            })
                        }
                        BuildConstraint::IsDiagonalTo(diagonal_id) => {
                            board.grid.get_diagonal(&from).iter().any(|(_dir, t)| {
                                match &t.building {
                                    Some(building) if building.owner == player.id => {
                                        building.blueprint_id == *diagonal_id.unit()
                                    }
                                    _ => false,
                                }
                            })
                        }
                        BuildConstraint::DistanceFrom(id, comp, val) => {
                            let closest = min_cost_search(board, &from, *val, |u| {
                                &u.blueprint_id == id.unit()
                            })
                            .map_or_else(|| 100, |pos| (from - pos).length());
                            comp.compare(closest, *val)
                        }
                        BuildConstraint::NumberOf(id, comp, val) => {
                            let number = board
                                .get_player_units(&player.id)
                                .filter(|u| &u.blueprint_id == id.unit())
                                .count() as i32;
                            comp.compare(number, *val)
                        }
                        BuildConstraint::OnlyOnFoodResource => {
                            tile.terrain.resource == Some(Resource::Food)
                        }
                        BuildConstraint::OnlyOnGoldResource => {
                            tile.terrain.resource == Some(Resource::Gold)
                        }
                        BuildConstraint::NotOnFoodResource => {
                            tile.terrain.resource != Some(Resource::Food)
                        }
                        BuildConstraint::NotOnGoldResource => {
                            tile.terrain.resource != Some(Resource::Gold)
                        }
                        _ => true,
                    });

            let any_terrain =
                potential_build
                    .build_constraints
                    .iter()
                    .any(|constraint| match constraint {
                        BuildConstraint::OnTerrain(terrain_id) => {
                            terrain_bp.header.id == *terrain_id.terrain()
                        }
                        _ => false,
                    });

            let occupied = board.grid.get_at(&from).building.is_some();

            let enemy_on_top = board
                .grid
                .get_at(&from)
                .unit
                .as_ref()
                .map(|u| board.get_player(&u.owner).is_hostile(player))
                .unwrap_or(false);

            let level = player.level >= potential_build.header.level;

            let unlocked = potential_build
                .required_tech
                .iter()
                .all(|tech_id| player.researched_technologies.contains(tech_id.tech()));

            let bonus = board.get_player_bonus(&player.id, Some(&self.build_id));
            let affordable = player
                .resources
                .contains(&potential_build.resources.cost.apply_cost(bonus));

            let civ = potential_build.required_civilization.is_empty()
                || potential_build
                    .required_civilization
                    .iter()
                    .find(|id| id.civilization() == &player.civilization)
                    .is_some();

            constraints
                && any_terrain
                && !occupied
                && !enemy_on_top
                && level
                && unlocked
                && affordable
                && civ
        })
    }

    pub fn is_building_active(board: &Board, upos: UnitPos) -> bool {
        let Some(building) = board.get_unit(&upos) else {
            return false;
        };

        let player = board.get_player(&building.owner);
        let building_bp = board.bp.get_unit(&building.blueprint_id);
        let tile = board.grid.get_at(&upos.xy);
        let terrain_bp = board.bp.get_terrain(&tile.terrain.blueprint_id);

        let constraints = building_bp
            .build_constraints
            .iter()
            .all(|constraint| match constraint {
                BuildConstraint::IsAdjacentTo(adjacent_id) => board
                    .grid
                    .get_adjacent(&upos.xy)
                    .iter()
                    .any(|(_dir, t)| match &t.building {
                        Some(building) if building.owner == player.id => {
                            building.blueprint_id == *adjacent_id.unit()
                        }
                        _ => false,
                    }),
                BuildConstraint::IsDiagonalTo(diagonal_id) => board
                    .grid
                    .get_diagonal(&upos.xy)
                    .iter()
                    .any(|(_dir, t)| match &t.building {
                        Some(building) if building.owner == player.id => {
                            building.blueprint_id == *diagonal_id.unit()
                        }
                        _ => false,
                    }),
                BuildConstraint::DistanceFrom(id, comp, val) => {
                    let closest =
                        min_cost_search(board, &upos.xy, *val, |u| &u.blueprint_id == id.unit())
                            .map_or_else(|| 100, |pos| (upos.xy - pos).length());
                    comp.compare(closest, *val)
                }
                BuildConstraint::OnlyOnFoodResource => {
                    tile.terrain.resource == Some(Resource::Food)
                }
                BuildConstraint::OnlyOnGoldResource => {
                    tile.terrain.resource == Some(Resource::Gold)
                }
                BuildConstraint::NotOnFoodResource => tile.terrain.resource != Some(Resource::Food),
                BuildConstraint::NotOnGoldResource => tile.terrain.resource != Some(Resource::Gold),
                _ => true,
            });

        let any_terrain = building_bp
            .build_constraints
            .iter()
            .any(|constraint| match constraint {
                BuildConstraint::OnTerrain(terrain_id) => {
                    terrain_bp.header.id == *terrain_id.terrain()
                }
                _ => false,
            });

        constraints && any_terrain
    }
}

impl From<ActBuild> for UnitAction {
    fn from(value: ActBuild) -> Self {
        UnitAction::Build(value.build_id, value.area)
    }
}

fn min_cost_search(
    board: &Board,
    from: &IVec2,
    max_steps: i32,
    predicate: impl Fn(&Unit) -> bool,
) -> Option<IVec2> {
    let mut frontier: Vec<(i32, IVec2)> = vec![(0, *from)];
    let mut visited: Vec<IVec2> = vec![*from];
    const MAX_TRAVEL_ITER: i32 = 10000;
    let mut i = -1;
    while !frontier.is_empty() {
        i += 1;
        if i >= MAX_TRAVEL_ITER {
            warn!(target: "action", "out of travel iterations");
            break;
        }
        let Some(index) = frontier
            .iter()
            .enumerate()
            .min_by_key(|(_, kv)| kv.0)
            .map(|(j, _)| j)
        else {
            panic!();
        };
        let (points, pos) = frontier.remove(index);
        for (dir, look_tile) in board.grid.get_adjacent(&pos) {
            let look = pos + *dir;
            if visited.contains(&look) {
                continue;
            }
            if look_tile.get_units().iter().any(|u| predicate(u)) {
                return Some(look);
            }
            if points + 1 < max_steps {
                frontier.push((points + 1, look));
            }
        }
        if visited.contains(&pos) {
            continue;
        }
        visited.push(pos);
    }
    None
}
