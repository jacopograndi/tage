use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActRepair {
    pub this: UnitTarget,
    pub target: UnitTarget,
}

impl Act for ActRepair {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let Some(building) = board.get_pos_target(&UnitPos::bot(pre.at)) else {
            return vec![];
        };
        let act = ActRepair {
            this: pre.clone(),
            target: building,
        };
        if !act.is_valid(board) {
            return vec![];
        }
        vec![act]
    }

    fn is_valid(&self, board: &Board) -> bool {
        if !board
            .bp
            .unit_has_ability(&self.this.unit.blueprint_id, "Repair")
        {
            return false;
        }
        let Some(building) = &board.get_unit(&board.get_target_pos(&self.target)) else {
            return false;
        };
        let cost = self.get_cost(board);
        let player = board.get_player(&self.target.unit.owner);
        let affordable = player.resources.contains(&cost);
        building.health < 100 && affordable
    }

    fn apply(&self, board: &mut Board) {
        let cost = self.get_cost(board);
        board.get_player_mut(&self.this.unit.owner).resources -= cost;
        let pos = board.get_target_pos(&self.target);
        board.modify_unit(&pos, |building| building.health = 100)
    }

    fn undo(&self, board: &mut Board) {
        let cost = self.get_cost(board);
        board.get_player_mut(&self.this.unit.owner).resources += cost;
        let pos = board.get_target_pos(&self.target);
        board.set_unit_at(&pos, Some(self.target.unit.clone()))
    }
}

impl ActRepair {
    fn get_cost(&self, board: &Board) -> Resources {
        let bonus = board.get_player_bonus(
            &self.target.unit.owner,
            Some(&self.target.unit.blueprint_id),
        );
        let building_bp = board.bp.get_unit(&self.target.unit.blueprint_id);
        let perc: f64 = 1.0 - self.target.unit.health as f64 / 100.0;
        let cost = building_bp.resources.cost.clone() * perc;
        cost.apply_cost(bonus)
    }
}

impl From<ActRepair> for UnitAction {
    fn from(value: ActRepair) -> Self {
        UnitAction::Repair(value.target)
    }
}
