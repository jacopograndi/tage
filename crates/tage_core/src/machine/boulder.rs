use std::{collections::HashMap, fmt::Debug};

use boulder::{player_action::Pre, research::ActResearch};
use rand::{thread_rng, Rng};
use tracing::{trace, warn};

use crate::{
    machine::{
        distance_travel_map::*,
        heuristics::{action_value_heuristic, unit_value_heuristic},
    },
    prelude::*,
};

use super::{weighted::Weighted, Machine};

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct Boulder {
    pub variance: u32,
}

impl Default for Boulder {
    fn default() -> Self {
        Boulder { variance: 0 }
    }
}

// Constructs a movelist for the current turn without looking ahead
// Uses a couple of heuristics and random noise to select decent moves
impl Machine for Boulder {
    fn turn_actions(&self, bp: &Blueprints, board: &mut Board) -> Vec<PlayerAction> {
        trace!(target: "machine.boulder", "day {} player {:?}", board.day, board.current_player_turn);
        let mut rng = thread_rng();

        let mut stripped_board = board.strip_fog(&board.current_player_turn);
        let board = &mut stripped_board;

        let mut player_actions = vec![];

        let unit_value_table = unit_value_heuristic(bp);
        let distance_travel_map =
            DistanceTravelMap::from_board(bp, board, &board.current_player_turn);

        let mut visited: HashMap<IVec2, i32> = board
            .get_player_units_pos(&board.current_player_turn)
            .map(|(_, xy)| (xy, 2))
            .collect();

        let mut saving_goal = Resources::new(0, 0);
        if board.get_current_player().can_age_up(bp) {
            saving_goal = board.get_current_player().get_age_up_cost();
        } else {
            if let Some(act) = ActResearch::generate(&(), board).first() {
                let tech_bp = board.bp.get_tech(&act.tech_id);
                saving_goal = tech_bp.cost.clone();
            }
        }

        if board.get_current_player().level == 0 {
            saving_goal = Resources::new(500, 500);
        }

        const MAX_MACHINE_ITERATIONS: u32 = 1000;
        let mut iter = 0;
        loop {
            if iter > MAX_MACHINE_ITERATIONS {
                warn!(target: "machine.boulder", "out of iterations. dump: {}", board.view());
                break;
            }
            iter += 1;

            let mut weighted: Vec<Weighted<PlayerAction>> = vec![];

            let active_unit = visited.iter_mut().next();
            if let Some((pos, amt)) = active_unit {
                *amt -= 1;
                let tile_actions = PlayerAction::generate(&Pre::Tile(*pos), board);
                let tile_weighted = tile_actions.into_iter().map(|act| {
                    Weighted::new(
                        {
                            let distance = match &act {
                                PlayerAction::Unit {
                                    target,
                                    destination,
                                    ..
                                } => {
                                    weight_travel(
                                        &distance_travel_map,
                                        bp,
                                        &target.unit,
                                        &target.at,
                                        &destination,
                                    )
                                    .weight
                                }
                                _ => 0,
                            };
                            let action_value =
                                action_value_heuristic(bp, board, &unit_value_table, &act);
                            let random = if self.variance > 0 {
                                let v = self.variance as i32;
                                rng.gen_range(-v..=v)
                            } else {
                                0
                            };
                            let cost = action_cost(&act, board);
                            let resources = board.get_current_player().resources.clone();
                            let after = resources - cost;
                            let cost_penalty = if after.food < saving_goal.food {
                                -100
                            } else if after.gold < saving_goal.gold {
                                -100
                            } else {
                                0
                            };
                            distance.clamp(-50, 50)
                                + action_value
                                + random
                                + weight_action(&act).weight
                                + cost_penalty
                        },
                        act,
                    )
                });
                weighted.extend(tile_weighted);
            } else {
                visited.clear();
            }

            visited.retain(|_, amt| *amt > 0);

            if board.get_current_player().research_queued.is_none()
                && !board.get_current_player().can_age_up(bp)
            {
                let research_actions = PlayerAction::generate(&Pre::Research, board);
                weighted.extend(research_actions.into_iter().map(|act| {
                    Weighted::new(
                        action_value_heuristic(bp, board, &unit_value_table, &act),
                        act.clone(),
                    )
                }));
            }

            let best = weighted.iter().max_by(|a, b| a.weight.cmp(&b.weight));
            if let Some(act) = best {
                trace!(target: "machine.boulder", "{}", act.content.view(bp));
                trace!(target: "machine.boulder", "best move weight: {}, avg: {}", act.weight,
                       weighted.iter().map(|w| w.weight).sum::<i32>() as f32 / weighted.len() as f32);
                act.content.apply(board);
                player_actions.push(act.content.clone());
            } else {
                if visited.is_empty() {
                    break;
                }
            }
        }

        for act in player_actions.iter().rev() {
            act.undo(board)
        }

        player_actions.push(PlayerAction::PassTurn);

        player_actions
    }
}

enum Stance {
    Fight { range: i32 },
    Flee,
    Build,
    MonkEmpty,
    MonkRelic,
}

fn weight_action(player_action: &PlayerAction) -> Weighted<PlayerAction> {
    let w = match player_action {
        PlayerAction::Unit { action, .. } => match action {
            UnitAction::Attack(_) => 50,
            UnitAction::Build(_, _) => 20,
            UnitAction::Heal(_) => 10,
            UnitAction::Convert(_) => 10,
            UnitAction::Relic => 20,
            _ => 0,
        },
        PlayerAction::Building { .. } => 10,
        PlayerAction::Research(_) => 300,
        PlayerAction::PassTurn => -1000,
    };
    Weighted::new(w, player_action.clone())
}

pub fn weight_travel(
    map: &DistanceTravelMap,
    bp: &Blueprints,
    unit: &Unit,
    from: &IVec2,
    to: &IVec2,
) -> Weighted<IVec2> {
    let unit_bp = bp.get_unit(&unit.blueprint_id);

    let convert = bp.get_ability_from_name("Convert");
    let stance = if !unit_bp.build_list.is_empty() {
        Stance::Build
    } else if unit_bp
        .abilities
        .iter()
        .any(|id| Some(id.ability()) == convert.as_ref())
    {
        if unit.holding_collectable == Some(Collectable::Relic) {
            Stance::MonkRelic
        } else {
            Stance::MonkEmpty
        }
    } else {
        if unit.health < 25 && map.hostile_unit.get_at(to) < &-10 {
            Stance::Flee
        } else {
            Stance::Fight {
                range: unit_bp.stats.range,
            }
        }
    };

    match stance {
        Stance::Fight { range } => Weighted::new(
            (relative_distance(&map.hostile_unit, from, to) + range)
                + (relative_distance(&map.hostile_building, from, to) + range),
            *to,
        ),
        Stance::Build => Weighted::new(
            relative_distance(&map.good_towncenter_spot, from, to)
                + relative_distance(&map.unclaimed_resource, from, to) * 2
                - relative_distance(&map.hostile_unit, from, to) / 2,
            *to,
        ),
        Stance::MonkEmpty => Weighted::new(
            relative_distance(&map.friendly_unit, from, to) / 2
                + relative_distance(&map.relic, from, to)
                + relative_distance(&map.hostile_unit, from, to) / 2,
            *to,
        ),
        Stance::MonkRelic => Weighted::new(relative_distance(&map.friendly_church, from, to), *to),
        Stance::Flee => Weighted::new(
            relative_distance(&map.friendly_building, from, to)
                - relative_distance(&map.hostile_unit, from, to),
            *to,
        ),
    }
}

fn relative_distance(grid: &Grid<i32>, from: &IVec2, to: &IVec2) -> i32 {
    grid.get_at(to) - grid.get_at(from)
}

fn action_cost(player_action: &PlayerAction, board: &mut Board) -> Resources {
    let has_cost = match player_action {
        PlayerAction::Unit { action, .. } => match action {
            UnitAction::Build(_, _) => true,
            _ => false,
        },
        PlayerAction::Building { action, .. } => match action {
            BuildingAction::Train(_) => true,
            BuildingAction::Trade(_) => true,
            _ => false,
        },
        _ => false,
    };
    if has_cost {
        let before = board.get_current_player().resources.clone();
        player_action.apply(board);
        let after = board.get_current_player().resources.clone();
        player_action.undo(board);
        before - after
    } else {
        Resources::new(0, 0)
    }
}
