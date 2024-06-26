use crate::prelude::*;

pub struct ActPickup {
    pub this: UnitTarget,
    pub pickup: Option<Collectable>,
}

impl Act for ActPickup {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let act = ActPickup {
            this: pre.clone(),
            pickup: ActPickup::get_collectible_at(board, pre.at),
        };
        act.is_valid(board).then_some(act).into_iter().collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        self.pickup == ActPickup::get_collectible_at(board, self.this.at)
            && self.this.unit.holding_collectable == None
    }

    fn apply(&self, board: &mut Board) {
        if self.pickup != None {
            let tile = board.grid.get_at_mut(&self.this.at);
            let mut production = Resources::new(0, 0);
            if let Some(collectible) = tile.terrain.collectable.take() {
                match collectible {
                    Collectable::BonusFood => production.food += 100,
                    Collectable::BonusGold => production.gold += 100,
                    Collectable::Ruins => {
                        tile.unit
                            .as_mut()
                            .map(|u| u.holding_collectable = Some(collectible));
                    }
                    Collectable::Relic => {}
                }
            }
            let player = board.get_current_player_mut();
            player.resources += production;
        }
    }

    fn undo(&self, board: &mut Board) {
        let mut production = Resources::default();
        let tile = board.grid.get_at_mut(&self.this.at);
        tile.unit.as_mut().map(|unit| {
            if let Some(collectible) = unit.holding_collectable.take().or(self.pickup.clone()) {
                match collectible {
                    Collectable::BonusFood => production.food += 100,
                    Collectable::BonusGold => production.gold += 100,
                    _ => {}
                }
                tile.terrain.collectable = Some(collectible);
            }
        });
        let player = board.get_current_player_mut();
        player.resources -= production;
    }
}

impl ActPickup {
    fn get_collectible_at(board: &Board, at: IVec2) -> Option<Collectable> {
        board
            .grid
            .get_at(&at)
            .terrain
            .collectable
            .clone()
            .filter(|collectable| *collectable != Collectable::Relic)
    }
}
