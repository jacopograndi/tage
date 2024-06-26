use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActConvert {
    pub this: UnitTarget,
    pub target: UnitTarget,
}

impl Act for ActConvert {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let Some(unit) = board.get_unit(&UnitPos::top(pre.at)) else {
            return vec![];
        };

        let bonus = board.get_player_bonus(&unit.owner, Some(&unit.blueprint_id))
            + board.get_unit_bonus(&unit.blueprint_id);

        if !bonus.forbid_attack && !(bonus.forbid_attack_after_move && unit.moved) {
            board
                .get_units_in_range(pre.at, 2)
                .filter_map(|(at, target_unit)| {
                    let act = ActConvert {
                        this: pre.clone(),
                        target: UnitTarget::new(target_unit.clone(), at),
                    };
                    act.is_valid(board).then_some(act)
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn is_valid(&self, board: &Board) -> bool {
        let upos = board.get_target_pos(&self.this);
        if matches!(upos.loc, UnitLocation::Bot) {
            return false;
        }

        let Some(this_unit) = board.get_unit(&upos) else {
            return false;
        };
        let Some(target_unit) = board.get_unit(&board.get_target_pos(&self.target)) else {
            return false;
        };

        if !board
            .bp
            .unit_has_ability(&this_unit.blueprint_id, "Convert")
        {
            return false;
        }

        let this_player = board.get_player(&this_unit.owner);
        let target_player = board.get_player(&target_unit.owner);
        let in_range = (self.this.at - self.target.at).length() <= 2;
        this_player.is_hostile(target_player) && in_range
    }

    fn apply(&self, board: &mut Board) {
        let this_unit = board.get_unit(&board.get_target_pos(&self.this)).unwrap();
        let target_unit = board.get_unit(&board.get_target_pos(&self.target)).unwrap();
        let target_terrain = &board.grid.get_at(&self.target.at).terrain.blueprint_id;
        let bonus = board.get_player_bonus(&this_unit.owner, Some(&this_unit.blueprint_id))
            + board.get_power_bonus_battle(
                &this_unit,
                &target_unit,
                target_terrain,
                (self.this.at - self.target.at).length(),
            );
        board.modify_unit(&board.get_target_pos(&self.target), |unit| {
            unit.conversion_attempt = Some((self.target.unit.owner.clone(), bonus.convert))
        });
    }

    fn undo(&self, board: &mut Board) {
        board.set_unit_target(self.target.clone());
    }
}

impl From<ActConvert> for UnitAction {
    fn from(value: ActConvert) -> Self {
        UnitAction::Convert(value.this)
    }
}
