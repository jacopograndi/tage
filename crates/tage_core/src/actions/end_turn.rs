use crate::prelude::*;
use end_turn::build::ActBuild;
use rand::{thread_rng, Rng};
use tracing::trace;

use self::{research::ActResearch, train::ActTrain};

#[derive(Debug, Clone)]
pub struct ActEndTurn;

impl Act for ActEndTurn {
    type Precondition = ();

    fn generate(_: &Self::Precondition, _: &mut Board) -> Vec<Self> {
        vec![ActEndTurn]
    }

    fn is_valid(&self, _: &Board) -> bool {
        true
    }

    fn apply(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let mut rng = thread_rng();

        // conversions and ruin rng
        for xy in iter_area(board.grid.size) {
            let tile = board.grid.get_at_mut(&xy);
            if let Some(unit) = &mut tile.unit {
                if unit.owner != board.current_player_turn {
                    continue;
                }
                if let Some((player, conversion_strenght)) = unit.conversion_attempt.take() {
                    if match conversion_strenght {
                        -1 => rng.gen_bool(0.1),
                        0 => rng.gen_bool(0.25),
                        1 => rng.gen_bool(0.33),
                        _ => rng.gen_bool(0.50),
                    } {
                        unit.owner = player;
                    }
                }
                if Some(Collectable::Ruins) == unit.holding_collectable {
                    unit.holding_collectable = None;
                    match rng.gen_range(0..=4) {
                        0 => {
                            board.grid.get_at_mut(&xy).unit = None;
                        }
                        1 => {
                            let player = board.get_current_player_mut();
                            player.resources.food += 200;
                        }
                        2 => {
                            let player = board.get_current_player_mut();
                            player.resources.gold += 200;
                        }
                        3 => {
                            if let Some(ActResearch { tech_id }) =
                                ActResearch::generate(&(), board).first()
                            {
                                let player = board.get_current_player_mut();
                                player.researched_technologies.push(tech_id.clone());
                            }
                        }
                        4 => {
                            let target =
                                board.grid.get_adjacent(&xy).iter().find_map(|(dir, t)| {
                                    let terrain_bp = bp.get_terrain(&t.terrain.blueprint_id);
                                    (t.get_top_unit() == None && terrain_bp.stats.move_cost < 5)
                                        .then_some(xy + **dir)
                                });
                            if let Some(pos) = target {
                                let mut unit_bp =
                                    bp.get_unit(&bp.get_unit_from_name("Militia").unwrap());
                                let player = board.get_player(&board.current_player_turn);
                                for _ in 0..player.level {
                                    unit_bp =
                                        bp.get_unit(unit_bp.upgrades_to.clone().unwrap().unit())
                                }
                                board.grid.get_at_mut(&pos).unit = Some(Unit {
                                    blueprint_id: unit_bp.header.id.clone(),
                                    owner: player.id.clone(),
                                    ..Default::default()
                                });
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        // advance to next player
        if Some(&board.current_player_turn) == board.player_turn_order.last() {
            board.day += 1;
            board.current_player_turn = board.player_turn_order[0].clone();
        } else {
            let current_turn = board
                .player_turn_order
                .iter()
                .position(|pl| pl == &board.current_player_turn)
                .expect("every player must be in the turn order");
            board.current_player_turn = board.player_turn_order[current_turn + 1].clone();
        }

        let player = board.get_player_mut(&board.current_player_turn.clone());
        player.train_discount = Resources::default();
        player.tech_discount = Resources::default();

        if let Some(queued) = player.research_queued.take() {
            match queued {
                QueuedResearch::Tech(id) => player.researched_technologies.push(id),
                QueuedResearch::AgeUp => {
                    player.level += 1;
                    upgrade_units(bp.as_ref(), board);
                }
            }
        }

        // unit update
        let market_list = get_market_units(bp.as_ref(), board);
        for xy in iter_area(board.grid.size) {
            let unit_build_list = board
                .get_unit(&UnitPos::top(xy))
                .map_or(vec![], |u| bp.get_unit(&u.blueprint_id).build_list.clone());
            let building_exists = board.get_unit(&UnitPos::bot(xy)).is_some();
            let current_player = board.current_player_turn.clone();
            board.modify_unit(&UnitPos::top(xy), |unit| {
                if unit.owner == current_player {
                    unit.done = false;
                    unit.moved = false;
                    if unit.in_construction {
                        unit.in_construction = false;
                        unit.health = (unit.health + 50).min(100);
                    } else if building_exists {
                        unit.health = (unit.health + 20).min(100);
                    }
                    unit.affected_by_powers.clear()
                }
            });
            let mut check_capture = false;
            board.modify_unit(&UnitPos::bot(xy), |building| {
                if building.owner == current_player {
                    building.done = false;
                    if building.in_construction {
                        if unit_build_list
                            .contains(&IdName::Id(Id::Unit(building.blueprint_id.clone())))
                        {
                            building.in_construction = false;
                            building.health = (building.health + 50).min(100);
                            check_capture = true;
                        }
                    }
                    let market = bp.get_unit_from_name("Market");
                    if market == Some(building.blueprint_id.clone()) {
                        building.train_list_override.clear();
                        let mut list = market_list.clone();
                        if list.len() >= 3 {
                            let override_list = (0..3)
                                .map(|_| {
                                    let pick = rng.clone().gen_range(0..list.len());
                                    list.swap_remove(pick)
                                })
                                .collect();
                            building.train_list_override = override_list;
                        }
                    }
                }
            });
            if check_capture {
                capture(board, xy);
            }
        }

        // production
        let (production, _) = calculate_production(board, &board.current_player_turn, false);
        let player = board.get_player_mut(&board.current_player_turn.clone());
        player.resources += production;
        player.train_discount = Resources::default();
        player.tech_discount = Resources::default();
    }

    fn undo(&self, _: &mut Board) {
        // end turn does not support undo
    }
}

fn get_market_units(bp: &Blueprints, board: &Board) -> Vec<UnitId> {
    let player = board.get_player(&board.current_player_turn);
    let Some(market_id) = bp.get_unit_from_name("Market") else {
        return vec![];
    };

    let market = bp.get_unit(&market_id);
    let mut list: Vec<UnitId> = market
        .train_list
        .iter()
        .map(|idref| idref.unit())
        .filter_map(|id| {
            let constraints = ActTrain::check_train_constraints(board, id, &player.id);
            let level = bp.get_unit(id).header.level == player.level;
            (constraints && level).then_some(id.clone())
        })
        .collect();
    list.sort_by(|a, b| a.0.cmp(&b.0));
    list.dedup_by(|a, b| a == b);
    if let Some(scout_cav) = bp.get_unit_from_name("Scout Cavalry") {
        if !list.contains(&scout_cav) {
            list.push(scout_cav);
        }
    }
    list
}

fn upgrade_units(bp: &Blueprints, board: &mut Board) {
    for xy in iter_area(board.grid.size) {
        let tile = board.grid.get_at_mut(&xy);
        if let Some(unit) = &mut tile.unit {
            let owner_level = board
                .players
                .iter()
                .find(|player| player.id == unit.owner)
                .unwrap()
                .level;
            if unit.owner == board.current_player_turn {
                let unit_bp = bp.get_unit(&unit.blueprint_id);
                if let Some(upgrade) = &unit_bp.upgrades_to {
                    if bp.get_unit(upgrade.unit()).header.level <= owner_level {
                        unit.blueprint_id = upgrade.unit().clone();
                    }
                }
            }
        }
    }
}

pub fn calculate_production(
    board: &Board,
    player_id: &PlayerId,
    tally_details: bool,
) -> (Resources, Vec<(UnitId, Resources)>) {
    let church = board.bp.get_unit_from_name("Church");
    let mut production = Resources::default();
    let mut tally = Vec::<(UnitId, Resources)>::new();
    for xy in iter_area(board.grid.size) {
        let tile = board.grid.get_at(&xy);
        for unit in tile.get_units().iter() {
            if &unit.owner != player_id {
                continue;
            }
            if board.unit_loc(unit) == UnitLocation::Bot {
                if !ActBuild::is_building_active(board, UnitPos::bot(xy)) {
                    continue;
                }
            }
            let bonus = board.get_player_bonus(player_id, Some(&unit.blueprint_id));
            let unit_bp = board.bp.get_unit(&unit.blueprint_id);
            let mut produces = unit_bp.resources.produces.clone() * (unit.health as f64 * 0.01);
            if church == Some(unit_bp.header.id.clone())
                && unit.holding_collectable == Some(Collectable::Relic)
            {
                produces.gold += 100
            }
            assert!(
                produces
                    .apply_produces(bonus.clone())
                    .contains(&Resources::new(0, 0)),
                "{:?}, {:?}, {:?}",
                unit_bp,
                produces.apply_produces(bonus.clone()),
                &bonus,
            );
            let enhanced_produces = produces.apply_produces(bonus);
            production += enhanced_produces.clone();
            if tally_details {
                tally.push((unit.blueprint_id.clone(), enhanced_produces));
            }
        }
    }
    (production, tally)
}

/// Buildings with the `Capture` ability can set the owner as their own for inactive buildings.
/// If the inactive buildings would become active if the new building was of the same owner, then
/// they are captured.
/// Capturing can propagate (in theory, it shouldn't happen with vanilla blueprints)
fn capture(board: &mut Board, xy: IVec2) {
    let Some(building) = board.get_unit(&UnitPos::bot(xy)) else {
        return;
    };
    if !board.bp.unit_has_ability(&building.blueprint_id, "Capture") {
        return;
    }

    trace!(target: "capture", "capturing from {}", xy);

    let original_owner = building.owner.clone();

    let inactive_buildings: Vec<IVec2> = board
        .get_units_in_range(xy, 12)
        .filter(|(pos, unit)| board.unit_loc(unit) == UnitLocation::Bot && pos != &xy)
        .filter(|(pos, _)| !ActBuild::is_building_active(board, UnitPos::bot(*pos)))
        .map(|(pos, _)| pos)
        .collect();
    let mut activated_buildings = vec![];
    for player in board.players.clone().iter() {
        let building = board.get_unit_mut(&UnitPos::bot(xy)).unwrap();
        building.owner = player.id.clone();

        for pos in inactive_buildings.iter() {
            if !activated_buildings.contains(pos)
                && ActBuild::is_building_active(board, UnitPos::bot(*pos))
            {
                activated_buildings.push(*pos);
            }
        }
    }

    let building = board.get_unit_mut(&UnitPos::bot(xy)).unwrap();
    building.owner = original_owner.clone();

    for pos in activated_buildings.iter() {
        let building = board.get_unit_mut(&UnitPos::bot(*pos)).unwrap();
        building.owner = original_owner.clone();
    }

    trace!(target: "capture", "captured {:?}", activated_buildings);

    // propagation
    for pos in activated_buildings.into_iter() {
        capture(board, pos);
    }
}
