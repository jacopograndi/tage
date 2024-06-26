use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActDone {
    pub this: UnitTarget,
}

impl Act for ActDone {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let act = ActDone { this: pre.clone() };
        act.is_valid(board).then_some(act).into_iter().collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        self.this.unit.owner == board.current_player_turn && !self.this.unit.done
    }

    fn apply(&self, board: &mut Board) {
        board.modify_unit(&board.get_target_pos(&self.this), |unit| unit.done = true);
    }

    fn undo(&self, board: &mut Board) {
        board.modify_unit(&board.get_target_pos(&self.this), |unit| unit.done = false);
    }
}

impl From<ActDone> for BuildingAction {
    fn from(_value: ActDone) -> Self {
        BuildingAction::Done
    }
}

impl From<ActDone> for UnitAction {
    fn from(_value: ActDone) -> Self {
        UnitAction::Done
    }
}

#[derive(Debug, Clone)]
pub struct ActNone {
    pub this: UnitTarget,
}

impl Act for ActNone {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, _: &mut Board) -> Vec<Self> {
        vec![ActNone { this: pre.clone() }]
    }

    fn is_valid(&self, _: &Board) -> bool {
        true
    }

    fn apply(&self, _: &mut Board) {}
    fn undo(&self, _: &mut Board) {}
}

impl From<ActNone> for UnitAction {
    fn from(_value: ActNone) -> Self {
        UnitAction::Done
    }
}
