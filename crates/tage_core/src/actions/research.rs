use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ActResearch {
    pub tech_id: TechId,
}

impl Act for ActResearch {
    type Precondition = ();

    fn generate(_pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        board
            .bp()
            .techs
            .iter()
            .filter_map(|(tech_id, _)| {
                Some(ActResearch {
                    tech_id: tech_id.clone(),
                })
                .filter(|act| act.is_valid(board))
            })
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        let player = board.get_current_player();
        if player
            .researched_technologies
            .iter()
            .any(|known| known == &self.tech_id)
        {
            return false;
        }

        let tech_bp = board.bp.get_tech(&self.tech_id);
        if !board.get_player_units(&player.id).any(|unit| {
            !unit.in_construction
                && tech_bp
                    .require
                    .satisfied(board.bp(), board.bp().get_unit(&unit.blueprint_id))
        }) {
            return false;
        }

        let cost = tech_bp.cost.clone() - player.tech_discount.clone();
        let affordable = player.resources.contains(&cost);
        tech_bp.level <= player.level && affordable
    }

    fn apply(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let player_id = board.current_player_turn.clone();
        let tech_bp = bp.get_tech(&self.tech_id);
        let player = board.get_player_mut(&player_id);
        player.resources -= tech_bp.cost.clone() - player.tech_discount.clone();
        if let Some(prev) = player.research_queued.take() {
            match prev {
                QueuedResearch::Tech(_) => {
                    player.resources += tech_bp.cost.clone() - player.tech_discount.clone();
                }
                QueuedResearch::AgeUp => {
                    let cost = player.get_age_up_cost();
                    player.resources += cost;
                }
            }
        }
        player.research_queued = Some(QueuedResearch::Tech(self.tech_id.clone()));
    }

    fn undo(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let player_id = board.current_player_turn.clone();
        let player = board.get_player_mut(&player_id);
        let tech_bp = bp.get_tech(&self.tech_id);
        if let Some(_queued) = player.research_queued.take() {
            player.resources += tech_bp.cost.clone() - player.tech_discount.clone();
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActAgeUp {
    pub this: UnitTarget,
}

impl Act for ActAgeUp {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        [ActAgeUp { this: pre.clone() }]
            .into_iter()
            .filter(|act| act.is_valid(board))
            .collect()
    }

    fn is_valid(&self, board: &Board) -> bool {
        let building_id = &self.this.unit.blueprint_id;
        if !board.bp.unit_has_ability(&building_id, "Age Up") {
            return false;
        }
        let player = board.get_player(&self.this.unit.owner);
        player.can_age_up(board.bp())
            && player.resources.contains(&player.get_age_up_cost())
            && player.research_queued != Some(QueuedResearch::AgeUp)
            && !self.this.unit.done
            && !self.this.unit.in_construction
            && player.id == board.current_player_turn
    }

    fn apply(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let player_id = board.current_player_turn.clone();
        let player = board.get_player_mut(&player_id);
        let cost = player.get_age_up_cost().clone();
        player.resources -= cost.clone();
        if let Some(prev) = player.research_queued.take() {
            match prev {
                QueuedResearch::Tech(id) => {
                    player.resources +=
                        bp.as_ref().get_tech(&id).cost.clone() - player.tech_discount.clone();
                }
                QueuedResearch::AgeUp => {
                    player.resources += cost - player.tech_discount.clone();
                }
            }
        }
        player.research_queued = Some(QueuedResearch::AgeUp);
    }

    fn undo(&self, board: &mut Board) {
        let player_id = board.current_player_turn.clone();
        let player = board.get_player_mut(&player_id);
        let cost = player.get_age_up_cost();
        if let Some(_queued) = player.research_queued.take() {
            player.resources += cost;
        }
    }
}

impl From<ActAgeUp> for PlayerAction {
    fn from(value: ActAgeUp) -> Self {
        PlayerAction::Building {
            target: value.this.clone(),
            action: value.into(),
        }
    }
}

impl From<ActAgeUp> for BuildingAction {
    fn from(_value: ActAgeUp) -> Self {
        BuildingAction::AgeUp
    }
}
