use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[cfg(test)]
mod test;

pub mod player_action;

pub mod attack;
pub mod build;
pub mod convert;
pub mod done;
pub mod end_turn;
pub mod heal;
pub mod merge;
pub mod pickup;
pub mod power;
pub mod relic;
pub mod repair;
pub mod research;
pub mod trade;
pub mod train;
pub mod travel;

/// Interface for implementation of player action's logic
pub trait Act: Sized {
    type Precondition;

    /// Provides the valid actions from the board given the provided pre action
    /// The pre action is used to identify unit and building actions
    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self>;

    /// Checks if the action is valid for the provided board
    fn is_valid(&self, board: &Board) -> bool;

    //todo: runtime debug assert `is_valid`
    /// Modifies the board assuming the action `is_valid`
    /// Otherwise it causes undefined behavior
    fn apply(&self, board: &mut Board);

    //todo: track last action and enforce this constraint at compile time
    /// Reverts the board from a state with this action applied to the previous state
    /// If the board isn't in a state with this action just applied it causes undefined behavior
    fn undo(&self, board: &mut Board);
}

/// Represents every possible player action
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum PlayerAction {
    Unit {
        target: UnitTarget,
        destination: IVec2,
        pickup: Option<Collectable>,
        action: UnitAction,
        path: Vec<IVec2>,
    },
    Building {
        target: UnitTarget,
        action: BuildingAction,
    },
    Research(TechId),
    PassTurn,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum UnitAction {
    Attack(UnitTarget),
    Build(UnitId, BuildArea),
    Heal(UnitTarget),
    Convert(UnitTarget),
    Relic,
    Merge(UnitTarget),
    Repair(UnitTarget),
    Power(PowerId, Vec<UnitTarget>),
    Done,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum BuildingAction {
    Train(UnitId),
    Trade(Resource),
    AgeUp,
    Done,
}
