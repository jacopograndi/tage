mod distance_travel_map;
mod eval;
mod heuristics;
mod weighted;

pub mod boulder;
//pub mod bruteforce;
pub mod peak;

use boulder::*;
use peak::*;

use crate::prelude::*;

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
pub enum MachineOpponent {
    Boulder(Boulder),
    Peak(Peak),
    WeakBoulder,
    AverageBoulder,
    StrongBoulder,
    WeakPeak,
    AveragePeak,
    StrongPeak,
}

trait Machine {
    fn turn_actions(&self, bp: &Blueprints, board: &mut Board) -> Vec<PlayerAction>;
}

pub fn get_machine_turn(
    bp: &Blueprints,
    board: &mut Board,
    machine: &MachineOpponent,
) -> Vec<PlayerAction> {
    match machine {
        MachineOpponent::Boulder(boulder) => boulder.turn_actions(bp, board),
        MachineOpponent::Peak(peak) => peak.turn_actions(bp, board),
        MachineOpponent::WeakBoulder => Boulder { variance: 100 }.turn_actions(bp, board),
        MachineOpponent::AverageBoulder => Boulder { variance: 40 }.turn_actions(bp, board),
        MachineOpponent::StrongBoulder => Boulder { variance: 10 }.turn_actions(bp, board),
        MachineOpponent::WeakPeak => Peak {
            variance: 50,
            playout_variance: 0,
            depth: 1,
            starting_branches: 10,
            playout_brances: 1,
        }
        .turn_actions(bp, board),
        MachineOpponent::AveragePeak => Peak {
            variance: 25,
            playout_variance: 10,
            depth: 3,
            starting_branches: 20,
            playout_brances: 1,
        }
        .turn_actions(bp, board),
        MachineOpponent::StrongPeak => Peak {
            variance: 40,
            playout_variance: 10,
            depth: 6,
            starting_branches: 10,
            playout_brances: 1,
        }
        .turn_actions(bp, board),
    }
}

/// A node in the game tree with the branch that it took to get there
#[derive(Clone, Debug)]
struct Ply {
    final_board: Board,
    actions: Vec<PlayerAction>,
}
