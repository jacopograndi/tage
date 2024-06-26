use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActRelic {
    pub this: UnitTarget,
}

impl Act for ActRelic {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        vec![ActRelic { this: pre.clone() }]
            .into_iter()
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        let tile = board.grid.get_at(&self.this.at);

        let Some(ref unit) = tile.unit else {
            return false;
        };

        let has_heal = board.bp.unit_has_ability(&unit.blueprint_id, "Heal");
        let has_convert = board.bp.unit_has_ability(&unit.blueprint_id, "Convert");
        if !has_heal && !has_convert {
            return false;
        }

        if Some(Collectable::Relic) == tile.terrain.collectable
            && self.this.unit.holding_collectable.is_none()
        {
            return true;
        }

        if let Some(ref building) = tile.building {
            let church = board.bp().get_unit_from_name("Church");
            if building.holding_collectable.is_none()
                && Some(&building.blueprint_id) == church.as_ref()
                && self.this.unit.holding_collectable == Some(Collectable::Relic)
            {
                return true;
            }
        }
        false
    }

    fn apply(&self, board: &mut Board) {
        let tile = board.grid.get_at_mut(&self.this.at);
        match &tile.terrain.collectable {
            Some(Collectable::Relic) => {
                // terrain to unit
                tile.unit
                    .as_mut()
                    .map(|unit| unit.holding_collectable = tile.terrain.collectable.take());
            }
            _ => {
                // unit to building
                tile.unit
                    .as_mut()
                    .map(|unit| unit.holding_collectable.take())
                    .and_then(|relic| {
                        tile.building
                            .as_mut()
                            .map(|building| building.holding_collectable = relic)
                    });
            }
        }
    }

    fn undo(&self, board: &mut Board) {
        let tile = board.grid.get_at_mut(&self.this.at);
        match &tile
            .building
            .as_ref()
            .map(|b| b.holding_collectable.clone())
            .flatten()
        {
            Some(Collectable::Relic) => {
                // building to unit
                tile.building
                    .as_mut()
                    .map(|building| building.holding_collectable.take())
                    .and_then(|relic| {
                        tile.unit
                            .as_mut()
                            .map(|unit| unit.holding_collectable = relic)
                    });
            }
            _ => {
                // unit to terrain
                tile.terrain.collectable = tile
                    .unit
                    .as_mut()
                    .map(|unit| unit.holding_collectable.take())
                    .flatten();
            }
        }
    }
}

impl From<ActRelic> for UnitAction {
    fn from(_value: ActRelic) -> Self {
        UnitAction::Relic
    }
}
