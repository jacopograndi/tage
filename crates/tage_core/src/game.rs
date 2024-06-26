use crate::{is_default, prelude::*};
use ron::de::SpannedError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use self::travel::ActTravel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub bp: Arc<Blueprints>,
    pub grid: Grid<BoardTile>,
    pub players: Vec<Player>,
    pub day: u32,
    pub current_player_turn: PlayerId,
    pub player_turn_order: Vec<PlayerId>,
    pub fog: HashMap<PlayerId, Grid<FogTile>>,
    pub fog_base: FogTile,
}

impl Board {
    pub fn bp(&self) -> &Blueprints {
        self.bp.as_ref()
    }

    pub fn get_pos_target(&self, pos: &UnitPos) -> Option<UnitTarget> {
        self.get_unit(pos).map(|unit| UnitTarget {
            unit: unit.clone(),
            at: pos.xy,
        })
    }
    pub fn get_target_pos(&self, target: &UnitTarget) -> UnitPos {
        UnitPos {
            xy: target.at,
            loc: self.unit_loc(&target.unit),
        }
    }

    pub fn unit_loc(&self, unit: &Unit) -> UnitLocation {
        let unit_bp = self.bp.get_unit(&unit.blueprint_id);
        match unit_bp.header.class {
            UnitClass::Bld => UnitLocation::Bot,
            _ => UnitLocation::Top,
        }
    }

    pub fn get_unit(&self, pos: &UnitPos) -> Option<&Unit> {
        self.grid.get_at(&pos.xy).get_unit_loc(pos.loc)
    }
    pub fn get_unit_target(&self, target: &UnitTarget) -> Option<&Unit> {
        self.get_unit(&self.get_target_pos(target))
    }
    pub fn get_unit_mut(&mut self, pos: &UnitPos) -> Option<&mut Unit> {
        self.grid.get_at_mut(&pos.xy).get_unit_loc_mut(pos.loc)
    }

    pub fn modify_unit(&mut self, pos: &UnitPos, f: impl FnMut(&mut Unit)) {
        self.get_unit_mut(&pos).map(f);
        self.sync_linked_units(self.get_unit(&pos).cloned(), pos);
    }

    pub fn sync_linked_units(&mut self, unit: Option<Unit>, pos: &UnitPos) {
        for linked in unit
            .as_ref()
            .map_or(vec![], |unit| unit.linked_units.clone())
            .iter()
            .filter(|xy| **xy != pos.xy)
        {
            self.grid
                .get_at_mut(&linked)
                .set_unit_loc(unit.clone(), pos.loc);
        }
    }

    pub fn set_unit_at(&mut self, pos: &UnitPos, unit: Option<Unit>) {
        self.grid
            .get_at_mut(&pos.xy)
            .set_unit_loc(unit.clone(), pos.loc);
        self.sync_linked_units(unit, pos)
    }

    pub fn set_unit_target(&mut self, target: UnitTarget) {
        self.set_unit_at(&self.get_target_pos(&target), Some(target.unit))
    }

    pub fn get_player(&self, id: &PlayerId) -> &Player {
        self.players.iter().find(|p| p.id == *id).unwrap()
    }

    pub fn get_player_mut(&mut self, id: &PlayerId) -> &mut Player {
        self.players.iter_mut().find(|p| p.id == *id).unwrap()
    }

    pub fn player_index(&self, id: &PlayerId) -> usize {
        self.player_turn_order
            .iter()
            .enumerate()
            .find(|(_, p)| *p == id)
            .unwrap()
            .0
    }

    pub fn get_current_player(&self) -> &Player {
        self.get_player(&self.current_player_turn)
    }

    pub fn get_current_player_mut(&mut self) -> &mut Player {
        let player_id = self.current_player_turn.clone();
        self.get_player_mut(&player_id)
    }

    pub fn get_player_bonus(&self, player_id: &PlayerId, unit_id: Option<&UnitId>) -> Bonus {
        let bp = self.bp();
        let player = self.get_player(player_id);
        let unit_bp_opt = unit_id.map(|id| bp.get_unit(id));
        let civ_bonuses = bp
            .get_civilization(&player.civilization)
            .unit_bonuses
            .iter();
        let tech_bonuses = player
            .researched_technologies
            .iter()
            .map(|tech_id| bp.get_tech(tech_id).unit_bonuses.iter())
            .flatten();
        civ_bonuses
            .chain(tech_bonuses)
            .filter_map(|unit_bonus| {
                unit_bp_opt.map_or(Some(unit_bonus.bonus.clone()), |unit_bp| {
                    unit_bonus
                        .affects
                        .satisfied(bp, unit_bp)
                        .then_some(unit_bonus.bonus.clone())
                })
            })
            .sum()
    }

    pub fn get_trained_from_bonus(&self, player_id: &PlayerId, building_id: &UnitId) -> Bonus {
        let bp = self.bp();
        let player = self.get_player(player_id);
        player
            .researched_technologies
            .iter()
            .map(|t| {
                bp.get_tech(t)
                    .trained_from_bonus
                    .iter()
                    .filter_map(|(trainer_id, tf_bonus)| {
                        (trainer_id.unit() == building_id).then_some(tf_bonus.clone())
                    })
                    .sum()
            })
            .sum()
    }

    pub fn get_unit_bonus(&self, unit_id: &UnitId) -> Bonus {
        let unit_bp = self.bp.get_unit(unit_id);
        let bonus = unit_bp
            .abilities
            .iter()
            .map(|ability_id| {
                let ability_bp = self.bp.get_ability(ability_id.ability());
                ability_bp.unit_bonuses.iter().cloned().sum()
            })
            .sum();
        bonus
    }

    pub fn fold_battle_bonuses<'a>(
        &self,
        bonuses: impl Iterator<Item = &'a BattleBonus>,
        this_bp: &UnitBlueprint,
        opponent_bp: Option<&UnitBlueprint>,
        battlefield_id: Option<&TerrainId>,
        distance: Option<i32>,
    ) -> (Bonus, Bonus) {
        let (mut this_bonus, mut opponent_bonus) = (Bonus::default(), Bonus::default());
        for (target, bonus) in bonuses.filter_map(|battle_bonus| {
            let this_check = battle_bonus.require_this.satisfied(self.bp(), this_bp);
            let opponent_check = if let Some(opp) = opponent_bp {
                battle_bonus.require_opponent.satisfied(self.bp(), opp)
            } else {
                true
            };
            let terrain_check = if let Some(terrain) = battlefield_id {
                battle_bonus
                    .require_terrain
                    .iter()
                    .any(|id| id.terrain() == terrain)
                    || battle_bonus.require_terrain.is_empty()
            } else {
                true
            };
            let distance_check = if let Some(distance) = distance {
                battle_bonus
                    .require_distance
                    .iter()
                    .all(|(comp, val)| comp.compare(distance, *val))
            } else {
                true
            };
            (this_check && opponent_check && terrain_check && distance_check)
                .then(|| (&battle_bonus.target, battle_bonus.bonus.clone()))
        }) {
            match target {
                BattleBonusTarget::This => this_bonus = this_bonus + bonus,
                BattleBonusTarget::Opponent => opponent_bonus = opponent_bonus + bonus,
            }
        }
        (this_bonus, opponent_bonus)
    }

    /// Get the bonus for a unit not in combat
    /// Considers terrain bonuses as if the unit is defending
    pub fn get_unit_total_bonus(&self, target: &UnitTarget) -> Bonus {
        let unit = &target.unit;
        self.get_player_bonus(&unit.owner, Some(&unit.blueprint_id))
            + self.get_unit_bonus(&unit.blueprint_id)
            + self.get_terrain_bonus(&self.grid.get_at(&target.at).terrain.blueprint_id)
            + self.get_building_bonus(&target)
            + self.get_veterancy_bonus(unit)
            + self.get_power_bonus(unit)
    }

    pub fn get_power_bonus(&self, unit: &Unit) -> Bonus {
        unit.affected_by_powers
            .iter()
            .map(|power_id| {
                let power = self.bp.get_power(power_id);
                if power
                    .unit_bonus
                    .affects
                    .satisfied(self.bp(), self.bp.get_unit(&unit.blueprint_id))
                {
                    power.unit_bonus.bonus.clone() + power.bonus.clone()
                } else {
                    power.bonus.clone()
                }
            })
            .sum()
    }

    pub fn get_veterancy_bonus(&self, unit: &Unit) -> Bonus {
        let veterancy_threshold = if self
            .bp
            .unit_has_ability(&unit.blueprint_id, "Seasoned Veteran")
        {
            2
        } else {
            3
        };
        let bonus = match unit.veterancy / veterancy_threshold {
            0 => 0,
            1 => 15,
            2 => 30,
            _ => 45,
        };
        UnitStats {
            attack: bonus,
            ..Default::default()
        }
        .into()
    }

    pub fn get_building_bonus(&self, target: &UnitTarget) -> Bonus {
        let unit_bp = self.bp.get_unit(&target.unit.blueprint_id);
        let defence = match unit_bp.header.class {
            UnitClass::Bld => {
                // sum all ajdacency bonuses
                self.grid
                    .get_adjacent(&target.at)
                    .into_iter()
                    .map(|(_, tile)| {
                        if let Some(ref building) = tile.building {
                            self.bp
                                .get_unit(&building.blueprint_id)
                                .defence_bonus_to_adjacent_buildings
                        } else {
                            0
                        }
                    })
                    .sum()
            }
            _ => {
                // on top building bonus
                if let Some(ref building) = self.grid.get_at(&target.at).building {
                    self.bp
                        .get_unit(&building.blueprint_id)
                        .defence_bonus_to_unit_on_top
                } else {
                    0
                }
            }
        };
        UnitStats {
            defence,
            ..Default::default()
        }
        .into()
    }

    pub fn get_terrain_bonus(&self, terrain_id: &TerrainId) -> Bonus {
        let terrain = self.bp.get_terrain(terrain_id);
        Bonus {
            incr: BonusValue {
                stats: UnitStats {
                    range: terrain.stats.range_bonus,
                    sight: terrain.stats.sight_bonus,
                    ..Default::default()
                },
                ..Default::default()
            },
            perc: BonusValue {
                stats: UnitStats {
                    defence: terrain.stats.defence_bonus,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn get_power_bonus_battle(
        &self,
        unit: &Unit,
        opp: &Unit,
        battlefield: &TerrainId,
        distance: i32,
    ) -> Bonus {
        self.fold_battle_bonuses(
            unit.affected_by_powers
                .iter()
                .map(|power_id| &self.bp.get_power(power_id).battle_bonus),
            self.bp.get_unit(&unit.blueprint_id),
            Some(self.bp.get_unit(&opp.blueprint_id)),
            Some(battlefield),
            Some(distance),
        )
        .0 + unit
            .affected_by_powers
            .iter()
            .map(|power_id| self.bp.get_power(power_id).bonus.clone())
            .sum()
    }

    pub fn get_units<'a>(&'a self) -> impl Iterator<Item = &'a Unit> {
        iter_area(self.grid.size)
            .map(|xy| self.grid.get_at(&xy).get_units())
            .flatten()
    }

    pub fn get_units_pos<'a>(&'a self) -> impl Iterator<Item = (&'a Unit, IVec2)> {
        iter_area(self.grid.size)
            .map(|xy| {
                self.grid
                    .get_at(&xy)
                    .get_units()
                    .into_iter()
                    .map(move |u| (u, xy))
            })
            .flatten()
    }

    pub fn get_player_units<'a>(&'a self, id: &'a PlayerId) -> impl Iterator<Item = &'a Unit> {
        self.get_units().filter(|unit| unit.owner == *id)
    }

    pub fn get_hostile_units<'a>(&'a self, id: &'a PlayerId) -> impl Iterator<Item = &'a Unit> {
        let player = self.get_player(id);
        self.get_units()
            .filter(|unit| player.is_hostile(self.get_player(&unit.owner)))
    }

    pub fn get_player_units_pos<'a>(
        &'a self,
        id: &'a PlayerId,
    ) -> impl Iterator<Item = (&'a Unit, IVec2)> {
        self.get_units_pos().filter(|(unit, _)| unit.owner == *id)
    }

    pub fn get_units_in_range<'a>(
        &'a self,
        from: IVec2,
        range: i32,
    ) -> impl Iterator<Item = (IVec2, &'a Unit)> {
        iter_area(IVec2::splat(range * 2 + 1))
            .filter_map(move |xy| {
                let target = from + xy - IVec2::splat(range);
                let length = (xy - IVec2::splat(range)).length();
                (length > 0 && length <= range && self.grid.contains(&target)).then(move || {
                    self.grid
                        .get_at(&target)
                        .get_units()
                        .into_iter()
                        .map(move |unit| (target.clone(), unit))
                })
            })
            .flatten()
    }

    pub fn get_visible_from(&self, upos: &UnitPos) -> Vec<IVec2> {
        //todo: buildings
        let from = &upos.xy;
        let Some(unit) = self.get_unit(upos) else {
            return vec![];
        };
        let unit_bp = self.bp.units.get(&unit.blueprint_id).unwrap();
        let bonus = self.get_unit_total_bonus(&UnitTarget::new(unit.clone(), *from));
        let points = (unit_bp.stats.apply(bonus).sight as i32).max(0);
        let mut frontier: Vec<(i32, IVec2)> = vec![(points, *from)];
        let mut visited: Vec<IVec2> = vec![*from];
        const MAX_SIGHT_ITER: i32 = 10000;
        let mut i = -1;
        while !frontier.is_empty() {
            i += 1;
            if i >= MAX_SIGHT_ITER {
                //eprintln!("out of travel iterations");
                break;
            }
            let index = frontier
                .iter()
                .enumerate()
                .max_by_key(|(_, kv)| kv.0)
                .map(|(j, _)| j)
                .unwrap();
            let (points, pos) = frontier.remove(index);
            if points <= 0 {
                continue;
            }
            for (dir, look_tile) in self.grid.get_adjacent(&pos) {
                let look = pos + *dir;
                if visited.contains(&look) {
                    continue;
                }
                let look_blueprint = self.bp.get_terrain(&look_tile.terrain.blueprint_id);
                let look_points = look_blueprint.stats.sight_cost;
                if look_points > 0 {
                    frontier.push((points - look_points, look));
                }
            }
            if visited.contains(&pos) {
                continue;
            }
            visited.push(pos);
        }
        visited
    }

    pub fn get_winners(&self) -> Option<Vec<PlayerId>> {
        // Conquest: the only alliance left wins
        let mut alive: Vec<(PlayerId, Option<TeamId>)> = self
            .players
            .iter()
            .filter_map(|player| {
                self.get_player_units(&player.id)
                    .next()
                    .map(|_| (player.id.clone(), player.team.clone()))
            })
            .collect();
        let mut winners = vec![];
        while let Some((player, team)) = alive.pop() {
            if team == None && winners.is_empty() {
                winners.push((player, team))
            } else {
                if winners.iter().all(|(_, team_win)| {
                    *team_win == team
                         // team None can win only if it's alone
                         && team != None
                }) {
                    winners.push((player, team));
                } else {
                    return None;
                }
            }
        }
        if !winners.is_empty() && !alive.is_empty() {
            None
        } else {
            Some(winners.into_iter().map(|(player, _)| player).collect())
        }
    }

    pub fn load(bp: &Blueprints, path: &str) -> Result<Board, ()> {
        let raw = std::fs::read_to_string(path).map_err(|_| ())?;
        let board_view: BoardView = ron::from_str(raw.as_str()).map_err(|_| ())?;
        Ok(board_view.to(bp))
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let config = ron::ser::PrettyConfig::default()
            .escape_strings(false)
            .compact_arrays(true)
            .depth_limit(2);
        let string =
            ron::ser::to_string_pretty(&BoardView::from(self.bp(), self), config.clone()).unwrap();
        std::fs::write(path, string)
    }

    pub fn view(&self) -> String {
        let config = ron::ser::PrettyConfig::default()
            .compact_arrays(true)
            .depth_limit(2);
        ron::ser::to_string_pretty(&BoardView::from(self.bp(), self), config.clone()).unwrap()
    }

    pub fn init_fog(&mut self) {
        for player in self.players.iter() {
            self.fog.insert(
                player.id.clone(),
                Grid::fill(self.grid.size, self.fog_base.clone()),
            );
        }
    }

    pub fn refresh_fog(&mut self) {
        if self.fog_base == FogTile::Visible {
            return;
        }

        let mut new_fog = HashMap::new();
        for player in self.players.iter() {
            new_fog.insert(
                player.id.clone(),
                Grid::fill(self.grid.size, self.fog_base.clone()),
            );
        }

        for xy in iter_area(self.grid.size) {
            for loc in [UnitLocation::Top, UnitLocation::Bot].into_iter() {
                let upos = UnitPos::new(xy, loc);
                if let Some(unit) = self.get_unit(&upos) {
                    let visible = self.get_visible_from(&upos);
                    if let Some(player_fog) = new_fog.get_mut(&unit.owner) {
                        for v in visible {
                            player_fog.set_at(&v, FogTile::Visible);
                        }
                    }
                }
            }
        }

        for (id, old_player_fog) in self.fog.iter_mut() {
            let new_player_fog = new_fog.get(&id).unwrap();
            for xy in iter_area(self.grid.size) {
                let new = new_player_fog.get_at(&xy);
                let old = old_player_fog.get_at(&xy);
                let out = match new {
                    FogTile::Hidden => match old {
                        FogTile::Visible => match self.fog_base {
                            FogTile::Visible => FogTile::Visible,
                            FogTile::Explored => FogTile::Explored,
                            FogTile::Hidden => FogTile::Explored,
                        },
                        FogTile::Explored => FogTile::Explored,
                        FogTile::Hidden => FogTile::Hidden,
                    },
                    FogTile::Visible => FogTile::Visible,
                    FogTile::Explored => FogTile::Explored,
                };
                old_player_fog.set_at(&xy, out);
            }
        }
    }

    pub fn strip_fog(&self, player_id: &PlayerId) -> Board {
        let player = self.get_player(player_id);

        //todo: config to not use allied fog.
        let fog: HashMap<PlayerId, Grid<FogTile>> = self
            .fog
            .iter()
            .filter(|(id, _)| !player.is_hostile(self.get_player(id)))
            .map(|(id, grid)| (id.clone(), grid.clone()))
            .collect();

        let mut stripped = Board {
            fog: fog.clone(),
            ..self.clone()
        };

        for xy in iter_area(self.grid.size) {
            if fog
                .iter()
                .any(|(_, grid)| grid.get_at(&xy) != &FogTile::Visible)
            {
                let tile = stripped.grid.get_at_mut(&xy);
                tile.unit = None;
                tile.building = None;
            }
        }

        stripped
    }

    pub fn fog_bonk(&self, player_action: PlayerAction) -> PlayerAction {
        match &player_action {
            PlayerAction::Unit {
                target,
                destination,
                path,
                ..
            } => {
                let act = ActTravel {
                    this: target.clone(),
                    destination: destination.clone(),
                    path: path.clone(),
                };
                if let Some(reachable) = act.has_bonked(self) {
                    PlayerAction::Unit {
                        target: target.clone(),
                        destination: reachable.destination,
                        pickup: None,
                        action: UnitAction::Done,
                        path: reachable.path,
                    }
                } else {
                    player_action
                }
            }
            _ => player_action,
        }
    }

    pub fn can_undo(&self) -> bool {
        self.fog_base == FogTile::Visible
    }
}

/// A tile in the map, identified by an IVec2 into the `board.grid`.
/// Has a terrain, a building slot, a unit slot and remembers the initial spawn points.
#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[serde(default)]
pub struct BoardTile {
    pub terrain: TerrainTile,

    #[serde(default, skip_serializing_if = "is_default")]
    pub building: Option<Unit>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub unit: Option<Unit>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub spawn_point: Option<PlayerId>,
}

impl BoardTile {
    pub fn get_unit_loc(&self, loc: UnitLocation) -> Option<&Unit> {
        match loc {
            UnitLocation::Top => self.unit.as_ref(),
            UnitLocation::Bot => self.building.as_ref(),
        }
    }

    pub fn get_unit_loc_mut(&mut self, loc: UnitLocation) -> Option<&mut Unit> {
        match loc {
            UnitLocation::Top => self.unit.as_mut(),
            UnitLocation::Bot => self.building.as_mut(),
        }
    }

    pub fn set_unit_loc(&mut self, unit: Option<Unit>, loc: UnitLocation) {
        match loc {
            UnitLocation::Top => self.unit = unit,
            UnitLocation::Bot => self.building = unit,
        }
    }

    pub fn get_top_unit(&self) -> Option<&Unit> {
        match (&self.unit, &self.building) {
            (Some(unit), _) => Some(unit),
            (None, Some(building)) => Some(building),
            (None, None) => None,
        }
    }

    pub fn get_units(&self) -> Vec<&Unit> {
        match (&self.unit, &self.building) {
            (Some(unit), Some(building)) => vec![unit, building],
            (Some(unit), None) => vec![unit],
            (None, Some(building)) => vec![building],
            (None, None) => vec![],
        }
    }

    pub fn get_unit_by_class(&self, class: &UnitClass) -> Option<&Unit> {
        match class {
            UnitClass::Bld => self.building.as_ref(),
            _ => self.unit.as_ref(),
        }
    }

    pub fn set_unit(&mut self, unit: Option<Unit>, unit_bp: &UnitBlueprint) {
        match unit_bp.header.class {
            UnitClass::Bld => self.building = unit,
            _ => self.unit = unit,
        }
    }

    pub fn get_movement_cost(&self, bp: &Blueprints) -> i32 {
        if self.terrain.has_road {
            1
        } else {
            bp.get_terrain(&self.terrain.blueprint_id).stats.move_cost
        }
    }
}

/// A tile in the map, identified by an IVec2 into the `board.grid`.
/// Has a terrain, a building slot, a unit slot and remembers the initial spawn points.
#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum FogTile {
    /// The tile is sighted and shows units and buildings
    #[default]
    Visible,

    /// The tile has been sighted in the past, currently no unit has sight of it.
    /// Shows only terrain data
    Explored,

    /// The tile has never been sighted, shows no data.
    Hidden,
}

/// Terrain variable data
#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
#[serde(default)]
pub struct TerrainTile {
    pub blueprint_id: TerrainId,

    /// Roads override the movement cost to 1
    // todo: make it an overriding ability of the terrain bp
    #[serde(default, skip_serializing_if = "is_default")]
    pub has_road: bool,

    /// Resources are either food (build a mill) or gold (build a mine)
    /// The resource is set only at map loading, it never changes
    #[serde(default, skip_serializing_if = "is_default")]
    pub resource: Option<Resource>,

    /// Collectables can be picked up by units and dropped on death
    #[serde(default, skip_serializing_if = "is_default")]
    pub collectable: Option<Collectable>,
}

#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Collectable {
    #[default]
    BonusFood,
    BonusGold,
    Ruins,
    Relic,
}

impl ToString for Collectable {
    fn to_string(&self) -> String {
        match self {
            Collectable::BonusFood => "|",
            Collectable::BonusGold => "!",
            Collectable::Ruins => "?",
            Collectable::Relic => "r",
        }
        .to_string()
    }
}

/// Rect type for defining a building space
#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct BuildArea {
    pub min: IVec2,
    pub max: IVec2,
}

impl BuildArea {
    pub fn from_pos(pos: &IVec2) -> BuildArea {
        BuildArea {
            min: *pos,
            max: *pos + IVec2::ONE,
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = IVec2> + 'a {
        iter_area(self.max - self.min).map(|off| off + self.min)
    }

    pub fn contains(&self, pos: &IVec2) -> bool {
        (self.min.x..self.max.x).contains(&pos.x) && (self.min.y..self.max.y).contains(&pos.y)
    }
}

/// Helper type to not have to write a serializer for serde
#[derive(
    Default, Clone, Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
pub struct BoardView {
    day: u32,
    players: Vec<Player>,
    player_turn_order: Vec<PlayerId>,
    current_player_turn: PlayerId,
    units: HashMap<(IVec2, UnitLocation), Unit>,
    grid: String,

    #[serde(default, skip_serializing_if = "is_default")]
    fog: HashMap<PlayerId, String>,

    #[serde(default, skip_serializing_if = "is_default")]
    fog_base: FogTile,
}

impl BoardView {
    pub fn from(bp: &Blueprints, value: &Board) -> Self {
        Self {
            grid: write_map(bp, &value.grid),
            day: value.day,
            players: value.players.clone(),
            player_turn_order: value.player_turn_order.clone(),
            current_player_turn: value.current_player_turn.clone(),
            units: value
                .get_units_pos()
                .map(|(unit, pos)| {
                    let unit_bp = bp.get_unit(&unit.blueprint_id);
                    (
                        (
                            pos,
                            match unit_bp.header.class {
                                UnitClass::Bld => UnitLocation::Bot,
                                _ => UnitLocation::Top,
                            },
                        ),
                        unit.clone(),
                    )
                })
                .collect(),
            fog: value
                .fog
                .iter()
                .map(|(player_id, grid)| (player_id.clone(), write_fog_grid(grid)))
                .collect(),
            fog_base: value.fog_base.clone(),
        }
    }

    pub fn to(self, bp: &Blueprints) -> Board {
        let mut board = Board {
            bp: Arc::new(bp.clone()),
            grid: parse_map(bp, &self.grid).unwrap().grid,
            day: self.day,
            players: self.players,
            player_turn_order: self.player_turn_order.clone(),
            current_player_turn: self.current_player_turn.clone(),
            fog: self
                .fog
                .into_iter()
                .map(|(player_id, s)| (player_id, parse_fog_grid(&s).unwrap()))
                .collect(),
            fog_base: self.fog_base,
        };
        for (unit_pos, unit) in self.units {
            let unit_bp = bp.get_unit(&unit.blueprint_id);
            board
                .grid
                .get_at_mut(&unit_pos.0)
                .set_unit(Some(unit), unit_bp);
        }
        board
    }
}

#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
#[serde(default)]
pub struct MapPlayerSettings {
    pub id: PlayerId,
    pub team: Option<TeamId>,
    pub level: i32,
    pub civilization: String,
    pub color: u32,
    pub controller: Controller,
    pub name: String,
    pub symbol: String,
}

impl MapPlayerSettings {
    pub fn civ(&self, bp: &Blueprints) -> CivilizationId {
        bp.get_civilization_from_name(&self.civilization)
            .unwrap_or(bp.civilizations.iter().next().unwrap().0.clone())
    }

    pub fn to_player(self, bp: &Blueprints) -> Player {
        Player {
            civilization: self.civ(bp),
            id: self.id,
            name: self.name,
            symbol: self.symbol,
            color: self.color,
            level: self.level,
            team: self.team,
            controller: self.controller,
            ..Default::default()
        }
    }
}

#[derive(
    Debug, Clone, Default, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode,
)]
#[serde(default)]
pub struct MapSettings {
    pub path: String,
    pub players: Vec<MapPlayerSettings>,
    pub place_hero: bool,
    pub fog_base: FogTile,
}

impl MapSettings {
    pub fn with_path(self, path: String) -> Self {
        Self { path, ..self }
    }

    pub fn from_string(config_str: &str) -> Result<Self, SpannedError> {
        ron::from_str(config_str)
    }
}

#[derive(Clone, Debug)]
pub enum ParseMapError {
    EmptyString,
    FileReadFailure,
    TerrainNotRecognized(String, IVec2),
}

pub struct ParseMapResult {
    pub grid: Grid<BoardTile>,
    pub spawn_points: Vec<(u32, IVec2)>,
}

const GRID_SEPARATOR: &'static str = " ";

pub fn parse_map(bp: &'_ Blueprints, map_string: &str) -> Result<ParseMapResult, ParseMapError> {
    let lines = map_string
        .trim()
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<&str>>();
    if lines.is_empty() {
        return Err(ParseMapError::EmptyString);
    }
    let row = lines[0].split(GRID_SEPARATOR).count();
    let size = IVec2::new(row as i32, lines.len() as i32);

    let mut spawn_points = vec![];
    let mut grid = Grid::default(size);
    for (y, line) in lines.iter().enumerate() {
        for (x, tile) in line.split(GRID_SEPARATOR).enumerate() {
            let pos = IVec2::new(x as i32, y as i32);
            let (resource, has_road) = match tile.chars().nth(0) {
                Some('$') => (Some(Resource::Gold), false),
                Some('+') => (Some(Resource::Food), false),
                Some('=') => (None, true),
                _ => (None, false),
            };
            let (collectable, spawn_point_num) = match tile.chars().nth(1) {
                Some('!') => (Some(Collectable::BonusGold), None),
                Some('|') => (Some(Collectable::BonusFood), None),
                Some('?') => (Some(Collectable::Ruins), None),
                Some('r') => (Some(Collectable::Relic), None),
                Some(char) if char.is_digit(16) => (None, char.to_digit(16)),
                _ => (None, None),
            };
            let Some((terrain_id, _)) = bp
                .terrain
                .iter()
                .find(|(_, terrain)| terrain.header.glyph.chars().nth(2) == tile.chars().nth(2))
            else {
                return Err(ParseMapError::TerrainNotRecognized(tile.to_string(), pos));
            };

            if let Some(num) = spawn_point_num {
                spawn_points.push((num, pos));
            }

            let terrain = TerrainTile {
                blueprint_id: terrain_id.clone(),
                resource,
                collectable,
                has_road,
            };
            let board_tile = BoardTile {
                terrain,
                building: None,
                unit: None,
                spawn_point: spawn_point_num.map(|num| PlayerId::new(num)),
            };
            grid.set_at(&pos, board_tile);
        }
    }
    Ok(ParseMapResult { grid, spawn_points })
}

pub fn write_map(bp: &'_ Blueprints, grid: &Grid<BoardTile>) -> String {
    let mut s = format!("\n");
    let mut line = String::new();
    for (xy, tile) in grid.iter() {
        let terrain_bp = bp.get_terrain(&tile.terrain.blueprint_id);
        let mut tile_str = format!("{}", terrain_bp.header.glyph);

        if tile.terrain.has_road {
            tile_str.replace_range(0..1, "=");
        }
        if let Some(res) = &tile.terrain.resource {
            tile_str.replace_range(
                0..1,
                match res {
                    Resource::Food => "+",
                    Resource::Gold => "$",
                },
            );
        }

        if let Some(coll) = &tile.terrain.collectable {
            tile_str.replace_range(
                1..2,
                match coll {
                    Collectable::BonusFood => "|",
                    Collectable::BonusGold => "!",
                    Collectable::Ruins => "?",
                    Collectable::Relic => "r",
                },
            );
        }
        if let Some(spawn_point) = &tile.spawn_point {
            tile_str.replace_range(1..2, &spawn_point.get().to_string())
        }

        line += &tile_str;

        if xy.x == grid.size.x - 1 {
            s += &(line.clone() + "\n");
            line.clear();
        } else {
            line += GRID_SEPARATOR;
        }
    }
    s
}

pub fn load_map(
    bp: &'_ Blueprints,
    settings: &MapSettings,
) -> Result<Grid<BoardTile>, ParseMapError> {
    let map_string =
        std::fs::read_to_string(&settings.path).map_err(|_| ParseMapError::FileReadFailure)?;

    let ParseMapResult {
        mut grid,
        mut spawn_points,
    } = parse_map(bp, map_string.as_str())?;

    spawn_points.sort_by(|a, b| a.0.cmp(&b.0).reverse());
    for player in settings.players.iter() {
        if let Some((_, spawn_point)) = spawn_points.pop() {
            grid.get_at_mut(&spawn_point).unit = Some(Unit {
                blueprint_id: bp.get_unit_from_name("Villager").unwrap(),
                owner: player.id.clone(),
                ..Default::default()
            });
            let mut unit_bp = bp.get_unit(&bp.get_unit_from_name("Militia").unwrap());
            for _ in 0..player.level {
                unit_bp = bp.get_unit(unit_bp.upgrades_to.clone().unwrap().unit());
            }
            grid.get_at_mut(&(spawn_point - IVec2::X)).unit = Some(Unit {
                blueprint_id: unit_bp.header.id.clone(),
                owner: player.id.clone(),
                ..Default::default()
            });
            let other_unit = if settings.place_hero {
                let civilization = bp.get_civilization(&player.civ(bp));
                civilization
                    .heroes
                    .iter()
                    .find_map(|id| {
                        (bp.get_unit(id.unit()).header.level == player.level).then(|| id.unit())
                    })
                    .expect("every civilization must have at least one hero per age")
                    .clone()
            } else {
                unit_bp.header.id.clone()
            };
            grid.get_at_mut(&(spawn_point + IVec2::Y)).unit = Some(Unit {
                blueprint_id: other_unit,
                owner: player.id.clone(),
                ..Default::default()
            });
        }
    }

    Ok(grid)
}

pub fn write_fog_grid(grid: &Grid<FogTile>) -> String {
    let mut s = format!("\n");
    let mut line = String::new();
    for (xy, tile) in grid.iter() {
        line += &match tile {
            FogTile::Visible => 0,
            FogTile::Explored => 1,
            FogTile::Hidden => 2,
        }
        .to_string();

        if xy.x == grid.size.x - 1 {
            s += &(line.clone() + "\n");
            line.clear();
        }
    }
    s
}

pub fn parse_fog_grid(grid_string: &str) -> Result<Grid<FogTile>, ParseMapError> {
    let lines = grid_string
        .trim()
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<&str>>();
    if lines.is_empty() {
        return Err(ParseMapError::EmptyString);
    }
    let row = lines[0].chars().count();
    let size = IVec2::new(row as i32, lines.len() as i32);

    let mut grid = Grid::default(size);
    for (y, line) in lines.iter().enumerate() {
        for (x, tile) in line.chars().enumerate() {
            let pos = IVec2::new(x as i32, y as i32);
            let val = match tile {
                '0' => FogTile::Visible,
                '1' => FogTile::Explored,
                '2' => FogTile::Hidden,
                _ => {
                    return Err(ParseMapError::TerrainNotRecognized(tile.to_string(), pos));
                }
            };
            grid.set_at(&pos, val);
        }
    }
    Ok(grid)
}
