use std::ops::Deref;

use crate::{blueprints::*, prelude::MachineOpponent};

#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct PlayerId(u32);

#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct TeamId(u32);

impl TeamId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl PlayerId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
    pub fn view(&self) -> String {
        format!("{}", self.0)
    }
}

impl Deref for PlayerId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum QueuedResearch {
    Tech(TechId),
    AgeUp,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub struct Player {
    pub id: PlayerId,
    pub resources: Resources,
    pub color: u32,
    pub symbol: String,
    pub level: i32,
    pub researched_technologies: Vec<TechId>,
    pub research_queued: Option<QueuedResearch>,
    pub tech_discount: Resources,
    pub train_discount: Resources,
    pub civilization: CivilizationId,
    pub team: Option<TeamId>,
    pub controller: Controller,
    pub name: String,
}

impl Player {
    pub fn get_resource(&self, resource: &Resource) -> i32 {
        match resource {
            Resource::Food => self.resources.food,
            Resource::Gold => self.resources.gold,
        }
    }

    pub fn get_resource_mut(&mut self, resource: &Resource) -> &mut i32 {
        match resource {
            Resource::Food => &mut self.resources.food,
            Resource::Gold => &mut self.resources.gold,
        }
    }

    pub fn get_age_up_tech_count(level: i32) -> i32 {
        match level {
            0 => 3,
            1 => 7,
            2 => 11,
            _ => 0,
        }
    }

    pub fn get_researched_of_level<'a>(
        &'a self,
        level: i32,
        bp: &'a Blueprints,
    ) -> impl Iterator<Item = &'a TechId> {
        self.researched_technologies
            .iter()
            .filter(move |id| bp.get_tech(id).level == level)
    }

    pub fn can_age_up(&self, bp: &Blueprints) -> bool {
        self.level < 3
            && self.get_researched_of_level(self.level, bp).count() as i32
                >= Self::get_age_up_tech_count(self.level)
    }

    pub fn get_age_up_cost(&self) -> Resources {
        let val = (self.level + 1) * 500;
        Resources {
            food: val,
            gold: val,
        }
    }

    pub fn is_hostile(&self, oth: &Player) -> bool {
        match (&self.team, &oth.team) {
            (Some(s), Some(t)) => s != t,
            _ => self.id != oth.id,
        }
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Controller {
    #[default]
    Human,
    Machine(MachineOpponent),
    Remote(u64),
}
