use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActTrade {
    pub this: UnitTarget,
    pub resource: Resource,
}

impl Act for ActTrade {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        [Resource::Food, Resource::Gold]
            .into_iter()
            .map(|resource| ActTrade {
                this: pre.clone(),
                resource,
            })
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        let Some(building) = board.get_unit(&board.get_target_pos(&self.this)) else {
            return false;
        };

        if !board.bp.unit_has_ability(&building.blueprint_id, "Trade") {
            return false;
        }

        let rate = ActTrade::get_rate(&self.this.unit.owner, board);
        let player = board.get_player(&building.owner);
        player.get_resource(&self.resource) >= rate
            && !self.this.unit.done
            && !self.this.unit.in_construction
            && building.owner == board.current_player_turn
    }

    fn apply(&self, board: &mut Board) {
        let rate = ActTrade::get_rate(&self.this.unit.owner, board);
        let player = board.get_player_mut(&self.this.unit.owner);
        *player.get_resource_mut(&self.resource) -= rate;
        *player.get_resource_mut(&self.resource.other()) += 100;
    }

    fn undo(&self, board: &mut Board) {
        let rate = ActTrade::get_rate(&self.this.unit.owner, board);
        let player = board.get_player_mut(&self.this.unit.owner);
        *player.get_resource_mut(&self.resource.other()) -= 100;
        *player.get_resource_mut(&self.resource) += rate;
    }
}

impl ActTrade {
    pub fn get_rate(owner: &PlayerId, board: &Board) -> i32 {
        let trade = board.get_player_bonus(&owner, None).trade;
        let rate = (250 - trade * 25).max(100);
        rate
    }
}

impl From<ActTrade> for PlayerAction {
    fn from(value: ActTrade) -> Self {
        PlayerAction::Building {
            target: value.this.clone(),
            action: value.into(),
        }
    }
}

impl From<ActTrade> for BuildingAction {
    fn from(value: ActTrade) -> Self {
        BuildingAction::Trade(value.resource)
    }
}
