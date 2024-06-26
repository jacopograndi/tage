use std::collections::HashMap;

use super::*;

pub fn action_value_heuristic(
    bp: &Blueprints,
    board: &mut Board,
    unit_value_table: &UnitValueTable,
    player_action: &PlayerAction,
) -> i32 {
    match player_action {
        PlayerAction::Unit {
            target: UnitTarget { unit, .. },
            action,
            destination,
            pickup,
            ..
        } => {
            let unit_bp = bp.get_unit(&unit.blueprint_id);

            let heal_value = board
                .grid
                .get_at(destination)
                .building
                .as_ref()
                .map_or(0, |_| if unit.health < 35 { 30 } else { 0 });

            let pickup_value = match pickup {
                Some(Collectable::Ruins) => 10,
                Some(Collectable::BonusFood) => 20,
                Some(Collectable::BonusGold) => 20,
                Some(Collectable::Relic) => 0,
                _ => 0,
            };
            let action_value = match action {
                UnitAction::Attack(target) => {
                    let target_bp = bp.get_unit(&target.unit.blueprint_id);
                    let previous_health_atk = unit.health;
                    let previous_health_def = target.unit.health;

                    player_action.apply(board);

                    let current_health_atk = board
                        .grid
                        .get_at(destination)
                        .get_unit_by_class(&unit_bp.header.class)
                        .map_or(0, |u| u.health);
                    let current_health_def = board
                        .grid
                        .get_at(&target.at)
                        .get_unit_by_class(&target_bp.header.class)
                        .map_or(0, |u| u.health);

                    player_action.undo(board);

                    let health_lost_atk = previous_health_atk - current_health_atk;
                    let health_lost_def = previous_health_def - current_health_def;
                    let value_lost_atk = health_lost_atk * unit_value_table.get(&unit.blueprint_id);
                    let value_lost_def =
                        health_lost_def * unit_value_table.get(&target.unit.blueprint_id);
                    100 + (value_lost_def * 5 - value_lost_atk) / 10
                }
                UnitAction::Build(id, _) => {
                    let building_bp = bp.get_unit(id);
                    let cost = &building_bp.resources.cost;
                    let produces = &building_bp.resources.produces;
                    let trains: i32 = building_bp
                        .train_list
                        .iter()
                        .map(|train_id| unit_value_table.get(train_id.unit()))
                        .sum::<i32>()
                        / 100;
                    let unlocks = (bp
                        .techs
                        .iter()
                        .filter(|(_, tech_bp)| tech_bp.require.satisfied(bp, building_bp))
                        .filter(|(tech_id, _)| {
                            !board
                                .get_player(&unit.owner)
                                .researched_technologies
                                .contains(tech_id)
                        })
                        .count()
                        * 20) as i32;
                    (produces.food + produces.gold) * 2 + trains + unlocks
                        - (cost.food + cost.gold) / 10
                }
                UnitAction::Heal(_) => 50,
                UnitAction::Convert(_) => 50,
                UnitAction::Relic => 200,
                UnitAction::Merge(_) => -10,
                UnitAction::Repair(_) => 70,
                UnitAction::Power(_, _) => 20,
                UnitAction::Done => 0,
            };
            pickup_value + heal_value + action_value
        }
        PlayerAction::Building { action, .. } => match action {
            BuildingAction::Train(id) => {
                let unit_bp = bp.get_unit(id);
                let cost = &unit_bp.resources.cost;
                let produces = &unit_bp.resources.produces;
                unit_value_table.get(&id) + (produces.food + produces.gold) * 2
                    - (cost.food + cost.gold) / 5
            }
            BuildingAction::Trade(resource) => {
                let res = &board.get_current_player().resources;
                (res.get_res(&resource) - res.get_res(&resource.other())) / 100 - 40
            }
            BuildingAction::AgeUp => 10000,
            BuildingAction::Done => 0,
        },
        _ => 0,
    }
}

pub struct UnitValueTable {
    map: HashMap<UnitId, i32>,
}

impl UnitValueTable {
    pub fn get(&self, id: &UnitId) -> i32 {
        *self.map.get(id).unwrap_or(&0)
    }
}

pub fn unit_value_heuristic(bp: &Blueprints) -> UnitValueTable {
    let mut map = HashMap::<UnitId, i32>::new();
    for (id, unit) in bp.units.iter() {
        map.insert(
            id.clone(),
            unit.stats.attack * (unit.stats.range / 2).max(1)
                + unit.stats.defence
                + unit.abilities.len() as i32 * 50
                + unit.powers.len() as i32 * 25
                + if unit.build_list.is_empty() { 0 } else { 40 }
                + match unit.header.class {
                    UnitClass::Inf => 0,
                    UnitClass::Bld => 100,
                    UnitClass::Cav => 50,
                    UnitClass::Sie => 25,
                    UnitClass::Ran => 35,
                },
        );
    }
    UnitValueTable { map }
}
