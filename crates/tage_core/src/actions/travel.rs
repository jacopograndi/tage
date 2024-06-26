use std::collections::HashMap;

use tracing::warn;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActTravel {
    pub this: UnitTarget,
    pub destination: IVec2,
    pub path: Vec<IVec2>,
}

impl Act for ActTravel {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        if !ActTravel::can_travel(&pre.unit, board) {
            return vec![];
        }
        ActTravel::get_reachable(pre, board)
            .into_iter()
            .map(|Reachable { destination, path }| ActTravel {
                this: pre.clone(),
                destination,
                path,
            })
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        ActTravel::get_reachable(&self.this, board)
            .iter()
            .find(|Reachable { destination, .. }| destination == &self.destination)
            .is_some()
            && ActTravel::can_travel(&self.this.unit, board)
    }

    fn apply(&self, board: &mut Board) {
        self.teleport(board, self.this.at, self.destination);
        board.modify_unit(&UnitPos::top(self.destination), |unit| unit.moved = true);
    }

    fn undo(&self, board: &mut Board) {
        board.modify_unit(&UnitPos::top(self.destination), |unit| unit.moved = false);
        self.teleport(board, self.destination, self.this.at);
    }
}

impl ActTravel {
    fn can_travel(unit: &Unit, board: &Board) -> bool {
        !unit.done && !unit.in_construction && unit.owner == board.current_player_turn
    }

    fn teleport(&self, board: &mut Board, from: IVec2, to: IVec2) {
        // note: no support for multitile moving units
        if from != to {
            board.grid.get_at_mut(&to).unit = board.grid.get_at_mut(&from).unit.take();
        }
    }

    // When the move is made on a fog-stripped board there may be units in the fog.
    // If we are going through a hostile unit, stop just before.
    // Use this function after generating a move from a fog-stripped board passing the clear board.
    pub fn has_bonked(&self, clear_board: &Board) -> Option<Reachable> {
        for i in 1..self.path.len() {
            if let Some(look_unit) = &clear_board.grid.get_at(&self.path[i]).get_top_unit() {
                if look_unit.owner != self.this.unit.owner {
                    // bonk
                    return Some(Reachable {
                        destination: self.path[i - 1],
                        path: self.path.iter().take(i - 1).cloned().collect(),
                    });
                }
            }
        }
        None
    }

    pub fn get_reachable(pre: &UnitTarget, board: &Board) -> Vec<Reachable> {
        let tile = board.grid.get_at(&pre.at);
        let Some(ref unit) = tile.unit else {
            return vec![];
        };
        let unit_bp = board.bp.units.get(&unit.blueprint_id).unwrap();
        let base_bonus: Bonus = board
            .bp
            .base_bonuses
            .iter()
            .filter_map(|battle_bonus| {
                let this_check = battle_bonus.require_this.satisfied(board.bp(), unit_bp);
                (this_check && !battle_bonus.bonus.terrain_movement_cost_override.is_empty())
                    .then(|| battle_bonus.bonus.clone())
            })
            .sum();

        let bonus: Bonus = base_bonus
            + board.get_player_bonus(&unit.owner, Some(&unit.blueprint_id))
            + board.get_unit_bonus(&unit.blueprint_id)
            + board.get_power_bonus(unit);

        //todo: consider moving the search to a generic function
        let points = unit_bp.stats.apply(bonus.clone()).movement as i32;
        let mut frontier: Vec<(i32, IVec2)> = vec![(points, pre.at)];
        let mut visited: Vec<IVec2> = vec![pre.at];
        let mut precedent = HashMap::<IVec2, (i32, IVec2)>::new();
        const MAX_TRAVEL_ITER: i32 = 10000;
        let mut i = -1;
        while !frontier.is_empty() {
            i += 1;
            if i >= MAX_TRAVEL_ITER {
                warn!("out of travel iterations");
                break;
            }
            let Some(index) = frontier
                .iter()
                .enumerate()
                .max_by_key(|(_, kv)| kv.0)
                .map(|(j, _)| j)
            else {
                warn!("cannot find a max in frontier");
                return vec![];
            };
            let (points, pos) = frontier.remove(index);
            if points < 0 {
                continue;
            }
            for (dir, look_tile) in board.grid.get_adjacent(&pos) {
                let look = pos + *dir;
                if visited.contains(&look) {
                    continue;
                }
                let look_points = if let Some(point_override) = bonus
                    .terrain_movement_cost_override
                    .iter()
                    .find(|(id, _)| id.terrain() == &look_tile.terrain.blueprint_id)
                    .map(|(_, ov)| ov)
                {
                    *point_override
                } else {
                    look_tile.get_movement_cost(board.bp())
                };
                if let Some(look_unit) = &look_tile.get_top_unit() {
                    if look_unit.owner != unit.owner {
                        continue;
                    }
                }
                if look_points >= 0 {
                    let diff = points - look_points;
                    frontier.push((diff, look));
                    if diff >= 0 {
                        if let Some((p, _)) = precedent.get(&look) {
                            if *p < diff {
                                precedent.insert(look, (diff, pos));
                            }
                        } else {
                            precedent.insert(look, (diff, pos));
                        }
                    }
                }
            }
            let visited_tile = board.grid.get_at(&pos);
            if visited_tile.unit.is_some() {
                continue;
            }
            if visited.contains(&pos) {
                continue;
            }

            visited.push(pos);
        }

        let mut reachable = vec![];
        for destination in visited {
            let mut path = vec![];
            let mut cursor = destination;
            for _ in 0..100 {
                if cursor == pre.at {
                    break;
                }
                if let Some((_, prec)) = precedent.get(&cursor) {
                    path.push(*prec);
                    cursor = *prec;
                }
            }
            path.reverse();
            reachable.push(Reachable { destination, path });
        }

        reachable
    }
}

#[derive(Clone, Debug)]
pub struct Reachable {
    pub destination: IVec2,
    pub path: Vec<IVec2>,
}

impl From<ActTravel> for UnitTarget {
    fn from(value: ActTravel) -> Self {
        UnitTarget::new(
            Unit {
                moved: true,
                ..value.this.unit
            },
            value.destination,
        )
    }
}

#[cfg(test)]
mod travel_path {
    use std::sync::Arc;

    use crate::v;

    use super::*;

    impl Reachable {
        fn new(destination: IVec2, path: Vec<IVec2>) -> Self {
            Self { destination, path }
        }
    }

    fn test_board(map: &str) -> Board {
        let blueprints =
            Blueprints::from_assets_location(&("../../".to_string() + BLUEPRINTS_PATH)).unwrap();
        let board = Board {
            bp: Arc::new(blueprints.clone()),
            grid: parse_map(&blueprints, &map).unwrap().grid,
            players: vec![Player {
                id: PlayerId::new(0),
                ..Default::default()
            }],
            day: 0,
            current_player_turn: PlayerId::new(0),
            player_turn_order: vec![PlayerId::new(0)],
            fog: HashMap::new(),
            fog_base: FogTile::Visible,
        };
        board
    }

    #[test]
    fn easy() {
        let map = "--- --- --- ---";
        let mut board = test_board(map);
        {
            let tile = board.grid.get_at_mut(&v!(0, 0));
            let _ = tile.unit.insert(Unit {
                blueprint_id: UnitId(0),
                owner: PlayerId::new(0),
                ..Default::default()
            });
        }
        let reachables = ActTravel::get_reachable(
            &board.get_pos_target(&UnitPos::top(v!(0, 0))).unwrap(),
            &board,
        );
        let mut expected = Vec::new();
        expected.push(Reachable {
            destination: v!(0, 0),
            path: vec![],
        });
        expected.push(Reachable {
            destination: v!(1, 0),
            path: vec![v!(0, 0)],
        });
        expected.push(Reachable {
            destination: v!(2, 0),
            path: vec![v!(0, 0), v!(1, 0)],
        });
        expected.push(Reachable {
            destination: v!(3, 0),
            path: vec![v!(0, 0), v!(1, 0), v!(2, 0)],
        });
        for reach in reachables {
            let e = expected
                .iter()
                .find(|e| e.destination == reach.destination)
                .unwrap();
            assert_eq!(reach.path, e.path, "at {}", reach.destination);
        }
    }

    #[test]
    fn hard() {
        let map = r"
()) --- =--
--- /\\ =--
=-- =-- =--";
        let mut board = test_board(map);
        {
            let tile = board.grid.get_at_mut(&v!(0, 1));
            let _ = tile.unit.insert(Unit {
                blueprint_id: UnitId(0),
                owner: PlayerId::new(0),
                ..Default::default()
            });
        }
        let reachables = ActTravel::get_reachable(
            &board.get_pos_target(&UnitPos::top(v!(0, 1))).unwrap(),
            &board,
        );
        let mut expected = Vec::new();
        // ()4 --2 --1
        // V-7 /\3 =-3
        // =-6 =-5 =-4
        expected.push(Reachable::new(v!(0, 1), vec![]));
        expected.push(Reachable::new(v!(0, 0), vec![v!(0, 1)]));
        expected.push(Reachable::new(v!(1, 0), vec![v!(0, 1), v!(0, 0)]));
        expected.push(Reachable::new(v!(1, 1), vec![v!(0, 1)]));
        expected.push(Reachable::new(v!(0, 2), vec![v!(0, 1)]));
        expected.push(Reachable::new(v!(1, 2), vec![v!(0, 1), v!(0, 2)]));
        expected.push(Reachable::new(v!(2, 2), vec![v!(0, 1), v!(0, 2), v!(1, 2)]));
        expected.push(Reachable::new(
            v!(2, 1),
            vec![v!(0, 1), v!(0, 2), v!(1, 2), v!(2, 2)],
        ));
        expected.push(Reachable::new(
            v!(2, 0),
            vec![v!(0, 1), v!(0, 2), v!(1, 2), v!(2, 2), v!(2, 1)],
        ));
        dbg!(&reachables);
        for reach in reachables {
            let e = expected
                .iter()
                .find(|e| e.destination == reach.destination)
                .unwrap();
            assert_eq!(reach.path, e.path, "at {}", reach.destination);
        }
    }
}
