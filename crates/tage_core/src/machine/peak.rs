use tracing::trace;

use crate::prelude::*;

use self::boulder::Boulder;

use super::{
    eval::Eval,
    heuristics::{unit_value_heuristic, UnitValueTable},
    Machine, Ply,
};

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
pub struct Peak {
    pub variance: u32,
    pub playout_variance: u32,
    pub depth: u32,
    pub starting_branches: u32,
    pub playout_brances: u32,
}

impl Default for Peak {
    fn default() -> Self {
        Peak {
            variance: 30,
            depth: 5,
            starting_branches: 10,
            playout_brances: 1,
            playout_variance: 0,
        }
    }
}

/// Searches the game tree with `hypermax`
/// Relies on `explore` being very fast
impl Machine for Peak {
    fn turn_actions(&self, bp: &Blueprints, board: &mut Board) -> Vec<PlayerAction> {
        trace!(target: "machine.peak", "day {} player {:?}", board.day, board.current_player_turn);
        let unit_value_table = unit_value_heuristic(bp);

        let depth = self.depth * (board.players.len() as u32);
        let first_pack = Pack {
            bp,
            branches: self.starting_branches,
            unit_value_table: &unit_value_table,
        };

        let children: Vec<Ply> = self.explore(board, &first_pack);

        let playout_pack = Pack {
            branches: self.playout_brances,
            ..first_pack
        };

        let player = board.player_index(&board.current_player_turn);
        let mut max_eval = Eval::min(board);
        let mut best = vec![PlayerAction::PassTurn];
        for child in children {
            let eval = self.hypermax(child.final_board, Eval::min(board), depth, &playout_pack);
            if eval.get(player) >= max_eval.get(player) {
                trace!(target: "machine.peak", "eval child {}", eval.get(player));
                max_eval = eval;
                best = child.actions;
            }
        }

        trace!(target: "machine.peak", "best eval child {}", max_eval.get(player));

        best
    }
}

impl Peak {
    /// MaxN algorithm with speculative pruning
    /// http://urn.kb.se/resolve?urn=urn:nbn:se:uu:diva-235687
    fn hypermax(&self, mut board: Board, mut alpha: Eval, depth: u32, pack: &Pack) -> Eval {
        if depth == 0 {
            let eval = Eval::from_board(&board, pack);
            return eval.zero_sum();
        }
        if let Some(winners) = board.get_winners() {
            let mut eval = Eval::from_board(&board, pack);
            for winner in &winners {
                let player = board.player_index(winner);
                *eval.get_mut(player) = 1000000;
            }
            trace!(target: "machine-peak", "found mate in {}, {:?}, {:?}", depth, alpha, eval);
            return eval.zero_sum();
        }
        let player = board.player_index(&board.current_player_turn);
        let children: Vec<Ply> = self.explore(&mut board, pack);
        let mut max_eval = Eval::min(&board);
        for (i, child) in children.into_iter().enumerate() {
            let eval = self.hypermax(child.final_board, alpha.clone(), depth - 1, pack);
            if i == 0 {
                max_eval = eval.clone();
            }
            if alpha.get(player) < eval.get(player) {
                *alpha.get_mut(player) = eval.get(player);
                max_eval = eval;
            }
            if alpha.sum() >= 0 {
                break;
            }
        }
        max_eval
    }

    fn explore(&self, board: &mut Board, pack: &Pack) -> Vec<Ply> {
        let boulder = Boulder {
            variance: self.playout_variance,
        };
        (0..pack.branches)
            .map(|i| {
                trace!(target: "machine-peak", "exploring {}/{} branch", i, pack.branches);
                let actions = boulder.turn_actions(pack.bp, board);
                let mut final_board = board.clone();
                for action in actions.iter() {
                    action.apply(&mut final_board);
                }
                Ply {
                    final_board,
                    actions,
                }
            })
            .collect()
    }
}

/// Every explorer needs a backpack with blueprints and a number that keeps him from taking too many
/// wrong turns
pub struct Pack<'a> {
    pub bp: &'a Blueprints,
    pub branches: u32,
    pub unit_value_table: &'a UnitValueTable,
}
