use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActMerge {
    pub this: UnitTarget,
    pub target: UnitTarget,
}

impl Act for ActMerge {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let tile = board.grid.get_at(&pre.at);
        let Some(ref unit) = tile.unit else {
            return vec![];
        };
        board
            .grid
            .get_adjacent(&pre.at)
            .iter()
            .filter_map(|(dir, tile)| {
                tile.unit.as_ref().and_then(|target_unit| {
                    Some(ActMerge {
                        this: UnitTarget::new(unit.clone(), pre.at),
                        target: UnitTarget::new(target_unit.clone(), pre.at + **dir),
                    })
                })
            })
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, _board: &Board) -> bool {
        self.this.unit.owner == self.target.unit.owner
            && self.this.unit.blueprint_id == self.target.unit.blueprint_id
            && self.target.unit.health < 100
    }

    fn apply(&self, board: &mut Board) {
        board
            .grid
            .get_at_mut(&self.this.at)
            .unit
            .take()
            .map(|mut unit| {
                unit.health = (unit.health + self.target.unit.health).min(100);
                unit.done = true;
                board.set_unit_at(&board.get_target_pos(&self.target), Some(unit));
            });
    }

    fn undo(&self, board: &mut Board) {
        board.set_unit_target(self.this.clone());
        board.set_unit_target(self.target.clone());
    }
}

impl From<ActMerge> for UnitAction {
    fn from(value: ActMerge) -> Self {
        UnitAction::Merge(value.target)
    }
}
