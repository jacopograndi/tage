use crate::prelude::*;

use self::end_turn::calculate_production;

#[derive(Debug, Clone)]
pub struct ActTrain {
    pub this: UnitTarget,
    pub train_id: UnitId,
}

impl Act for ActTrain {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let tile = board.grid.get_at(&pre.at);
        let Some(ref building) = tile.building else {
            return vec![];
        };
        if let Some(_) = tile.unit {
            return vec![];
        }

        let building_bp = board.bp.get_unit(&building.blueprint_id);
        let potential_train_list: Vec<UnitId> = if building.train_list_override.len() > 0 {
            building.train_list_override.clone()
        } else {
            building_bp
                .train_list
                .iter()
                .map(|id| id.unit().clone())
                .collect()
        };

        potential_train_list
            .into_iter()
            .map(|train_id| ActTrain {
                this: pre.clone(),
                train_id,
            })
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        ActTrain::check_train_constraints(board, &self.train_id, &self.this.unit.owner)
            && self.check_train_affordable(board)
            && !self.train_limit_reached(board)
            && !self.this.unit.done
            && !self.this.unit.in_construction
            && self.this.unit.owner == board.current_player_turn
    }

    fn apply(&self, board: &mut Board) {
        let tile = board.grid.get_at_mut(&self.this.at);
        let building = tile.building.as_ref().unwrap();
        let owner = building.owner.clone();
        let building_id = building.blueprint_id.clone();
        let cost = board.bp.get_unit(&self.train_id).resources.cost.clone();

        let bonus = Self::get_bonus(board, &owner, &self.train_id, &building_id);
        let player = board.get_player_mut(&owner);
        player.resources -= cost.apply_cost(bonus);

        board.grid.get_at_mut(&self.this.at).unit = Some(Unit {
            blueprint_id: self.train_id.clone(),
            health: 50,
            done: true,
            owner,
            in_construction: true,
            ..Default::default()
        });
    }

    fn undo(&self, board: &mut Board) {
        let unit = board.grid.get_at_mut(&self.this.at).unit.take().unwrap();
        let owner = unit.owner.clone();
        let building_id = self.this.unit.blueprint_id.clone();
        let cost = board.bp.get_unit(&self.train_id).resources.cost.clone();
        let bonus = Self::get_bonus(board, &owner, &self.train_id, &building_id);
        let player = board.get_player_mut(&owner);
        player.resources += cost.apply_cost(bonus);
    }
}

impl ActTrain {
    pub fn get_bonus(
        board: &Board,
        owner: &PlayerId,
        train_id: &UnitId,
        building_id: &UnitId,
    ) -> Bonus {
        let tech_bonus = board.get_player_bonus(&owner, Some(train_id));
        let trained_from_bonus = board.get_trained_from_bonus(owner, building_id);
        let player = board.get_player(&owner);
        let power_discount = Bonus {
            incr: BonusValue {
                resources: UnitResources::cost(-player.train_discount.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        let building_bp = board.bp.as_ref().get_unit(&building_id);
        tech_bonus + power_discount + trained_from_bonus + building_bp.train_cost_bonus.clone()
    }

    pub fn check_train_constraints(board: &Board, train_id: &UnitId, player_id: &PlayerId) -> bool {
        let player = board.get_player(player_id);
        let potential_train = board.bp.get_unit(train_id);

        let level = player.level >= potential_train.header.level;

        let unlocked = potential_train
            .required_tech
            .iter()
            .all(|tech_id| player.researched_technologies.contains(tech_id.tech()));

        let civ = potential_train.required_civilization.is_empty()
            || potential_train
                .required_civilization
                .iter()
                .find(|id| id.civilization() == &player.civilization)
                .is_some();

        level && unlocked && civ
    }

    pub fn check_train_affordable(&self, board: &Board) -> bool {
        let building = &self.this.unit;
        let building_bp = board.bp.get_unit(&building.blueprint_id);
        let owner = &self.this.unit.owner;
        let player = board.get_player(owner);
        let potential_train = board.bp.get_unit(&self.train_id);

        let unit = potential_train;
        let bonus = board.get_player_bonus(&building.owner, Some(&self.train_id));
        let trained_from_bonus =
            board.get_trained_from_bonus(&building.owner, &building_bp.header.id);
        let power_discount = Bonus {
            incr: BonusValue {
                resources: UnitResources::cost(-player.train_discount.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        let affordable = player.resources.contains(&unit.resources.cost.apply_cost(
            bonus + power_discount + trained_from_bonus + building_bp.train_cost_bonus.clone(),
        ));

        affordable
    }

    fn train_limit_reached(&self, board: &Board) -> bool {
        let all_units: Vec<&Unit> = board.get_player_units(&board.current_player_turn).collect();
        let mut unit_count = 0;
        for unit in all_units.iter() {
            let unit_bp = board.bp.get_unit(&unit.blueprint_id);
            if !matches!(unit_bp.header.class, UnitClass::Bld) {
                unit_count += 1;
            }
        }
        let (production, _) = calculate_production(board, &board.current_player_turn, false);
        let constraint = ((production.food + production.gold) / 100).max(7);
        unit_count >= constraint
    }
}

impl From<ActTrain> for PlayerAction {
    fn from(value: ActTrain) -> Self {
        PlayerAction::Building {
            target: value.this.clone(),
            action: value.into(),
        }
    }
}

impl From<ActTrain> for BuildingAction {
    fn from(value: ActTrain) -> Self {
        BuildingAction::Train(value.train_id)
    }
}
