use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActHeal {
    pub this: UnitTarget,
    pub target: UnitTarget,
}

impl Act for ActHeal {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        board
            .grid
            .get_adjacent(&pre.at)
            .iter()
            .filter_map(|(dir, tile)| {
                tile.unit.as_ref().and_then(|target_unit| {
                    Some(ActHeal {
                        this: pre.clone(),
                        target: UnitTarget {
                            unit: target_unit.clone(),
                            at: pre.at + **dir,
                        },
                    })
                })
            })
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        if !board
            .bp
            .unit_has_ability(&self.this.unit.blueprint_id, "Heal")
        {
            return false;
        }
        let distance = (self.target.at - self.this.at).length();
        self.target.unit.owner == self.this.unit.owner && distance == 1
    }

    fn apply(&self, board: &mut Board) {
        let target_terrain = &board.grid.get_at(&self.target.at).terrain.blueprint_id;
        let bonus = board
            .get_player_bonus(&self.this.unit.owner, Some(&self.this.unit.blueprint_id))
            + board.get_power_bonus_battle(
                &self.this.unit,
                &self.target.unit,
                target_terrain,
                (self.this.at - self.target.at).length(),
            );
        let heal_strenght = 20 + bonus.heal * 10;
        board.modify_unit(&board.get_target_pos(&self.target), |unit| {
            unit.health = (unit.health + heal_strenght).min(100)
        });
    }

    fn undo(&self, board: &mut Board) {
        board.set_unit_target(self.target.clone());
    }
}

impl From<ActHeal> for UnitAction {
    fn from(value: ActHeal) -> Self {
        UnitAction::Heal(value.target)
    }
}
