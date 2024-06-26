use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActPower {
    pub this: UnitTarget,
    pub targets: Vec<UnitTarget>,
    pub power_id: PowerId,
}

impl Act for ActPower {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let Some(unit) = board.get_unit(&UnitPos::top(pre.at)) else {
            return vec![];
        };
        let unit_bp = board.bp.get_unit(&unit.blueprint_id);
        unit_bp
            .powers
            .iter()
            .map(|idref| idref.power())
            .map(|power_id| {
                let targets = ActPower::get_power_targets(board.bp(), board, &pre.at, power_id);
                ActPower {
                    this: pre.clone(),
                    targets,
                    power_id: power_id.clone(),
                }
            })
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        let power = board.bp.get_power(&self.power_id);
        if power.require_on_building != UnitConstraint::NoConstraint {
            if !board
                .get_unit(&UnitPos::bot(self.this.at))
                .map(|building| {
                    let building_bp = board.bp.get_unit(&building.blueprint_id);
                    power
                        .require_on_building
                        .satisfied(board.bp(), &building_bp)
                })
                .unwrap_or(false)
            {
                return false;
            }
        }
        if self.targets.is_empty() {
            if power.effects.is_empty() {
                return false;
            }
            if power.effects.iter().all(|eff| match eff {
                PowerEffect::Heal(_) => true,
                _ => false,
            }) {
                return false;
            }
            return false;
        }
        let targets = ActPower::get_power_targets(board.bp(), board, &self.this.at, &self.power_id);
        self.targets == targets
    }

    fn apply(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let power = bp.as_ref().get_power(&self.power_id);
        for effect in power.effects.iter() {
            match effect {
                PowerEffect::Heal(heal) => {
                    for target in self.targets.iter() {
                        board
                            .grid
                            .get_at_mut(&target.at)
                            .unit
                            .as_mut()
                            .map(|unit| unit.health = (unit.health + heal).min(100));
                    }
                }
                PowerEffect::ProduceResources(produces) => {
                    let player = board.get_current_player_mut();
                    player.resources += produces.clone();
                }
                PowerEffect::TechDiscount(discount) => {
                    let player = board.get_current_player_mut();
                    player.tech_discount += discount.clone();
                }
                PowerEffect::TrainDiscount(discount) => {
                    let player = board.get_current_player_mut();
                    player.train_discount += discount.clone();
                }
            }
        }

        for target in self.targets.iter() {
            let tile = board.grid.get_at_mut(&target.at);
            if let Some(unit) = tile.unit.as_mut() {
                unit.affected_by_powers.push(power.id.clone());
            }
        }
    }

    fn undo(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let power = bp.get_power(&self.power_id);
        for effect in power.effects.iter() {
            match effect {
                PowerEffect::Heal(_) => {}
                PowerEffect::ProduceResources(produces) => {
                    let player = board.get_current_player_mut();
                    player.resources -= produces.clone();
                }
                PowerEffect::TechDiscount(discount) => {
                    let player = board.get_current_player_mut();
                    player.tech_discount -= discount.clone();
                }
                PowerEffect::TrainDiscount(discount) => {
                    let player = board.get_current_player_mut();
                    player.train_discount -= discount.clone();
                }
            }
        }

        for target in self.targets.iter() {
            board.grid.get_at_mut(&target.at).unit = Some(target.unit.clone());
        }
    }
}

impl ActPower {
    fn get_power_targets(
        bp: &Blueprints,
        board: &Board,
        from: &IVec2,
        power_id: &PowerId,
    ) -> Vec<UnitTarget> {
        let power = bp.get_power(power_id);
        power
            .targets
            .location
            .iter()
            .map(|location| match location {
                PowerTargetLocation::This => vec![*from],
                PowerTargetLocation::Adjacent => board
                    .grid
                    .get_adjacent(from)
                    .iter()
                    .map(|(dir, _)| **dir + *from)
                    .collect(),
                PowerTargetLocation::Diagonal => board
                    .grid
                    .get_diagonal(from)
                    .iter()
                    .map(|(dir, _)| **dir + *from)
                    .collect(),
                PowerTargetLocation::InSight => {
                    board.get_visible_from(&UnitPos::new(*from, UnitLocation::Top))
                }
                PowerTargetLocation::All => iter_area(board.grid.size)
                    .filter_map(|xy| (!board.grid.get_at(&xy).get_units().is_empty()).then(|| xy))
                    .collect(),
            })
            .flatten()
            .filter_map(|xy| {
                board
                    .grid
                    .get_at(&xy)
                    .unit
                    .as_ref()
                    .and_then(|u| Some(UnitTarget::new(u.clone(), xy)))
            })
            .filter(|target| {
                board.grid.get_at(from).unit.as_ref().is_some_and(|unit| {
                    match power.targets.status {
                        PowerTargetStatus::Friendly => target.unit.owner == unit.owner,
                        PowerTargetStatus::Enemy => target.unit.owner != unit.owner,
                        PowerTargetStatus::Any => true,
                    }
                })
            })
            .collect()
    }
}

impl From<ActPower> for UnitAction {
    fn from(value: ActPower) -> Self {
        UnitAction::Power(value.power_id, value.targets)
    }
}
