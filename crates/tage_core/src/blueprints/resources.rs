use super::*;
use crate::is_default;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(
    Default,
    Clone,
    Debug,
    Hash,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Resource {
    #[default]
    Food,
    Gold,
}

impl ToString for Resource {
    fn to_string(&self) -> String {
        match self {
            Resource::Food => "f",
            Resource::Gold => "g",
        }
        .to_string()
    }
}

impl Resource {
    pub fn other(&self) -> Self {
        match self {
            Self::Gold => Self::Food,
            Self::Food => Self::Gold,
        }
    }

    pub fn iter<'a>() -> impl Iterator<Item = &'a Resource> {
        [Resource::Food, Resource::Gold].iter()
    }

    pub fn view(&self) -> String {
        match self {
            Resource::Food => "food",
            Resource::Gold => "gold",
        }
        .to_string()
    }
}

#[derive(
    serde::Serialize,
    serde::Deserialize,
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    bincode::Encode,
    bincode::Decode,
)]
#[serde(default)]
pub struct Resources {
    #[serde(default, skip_serializing_if = "is_default")]
    pub food: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub gold: i32,
}

impl Resources {
    pub fn new(food: i32, gold: i32) -> Resources {
        Self { food, gold }
    }

    pub fn apply_produces(&self, bonus: Bonus) -> Resources {
        Self {
            food: apply(
                self.food,
                bonus.incr.resources.produces.food,
                bonus.perc.resources.produces.food,
            ),
            gold: apply(
                self.gold,
                bonus.incr.resources.produces.gold,
                bonus.perc.resources.produces.gold,
            ),
        }
    }

    pub fn apply_cost(&self, bonus: Bonus) -> Resources {
        Self {
            food: apply(
                self.food,
                bonus.incr.resources.cost.food,
                bonus.perc.resources.cost.food,
            ),
            gold: apply(
                self.gold,
                bonus.incr.resources.cost.gold,
                bonus.perc.resources.cost.gold,
            ),
        }
    }

    pub fn contains(&self, oth: &Resources) -> bool {
        self.food >= oth.food && self.gold >= oth.gold
    }

    pub fn get_res(&self, resource: &Resource) -> i32 {
        match resource {
            Resource::Food => self.food,
            Resource::Gold => self.gold,
        }
    }
}

impl Add for Resources {
    type Output = Resources;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            food: self.food + rhs.food,
            gold: self.gold + rhs.gold,
        }
    }
}

impl AddAssign for Resources {
    fn add_assign(&mut self, rhs: Self) {
        self.food += rhs.food;
        self.gold += rhs.gold;
    }
}

impl Sub for Resources {
    type Output = Resources;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            food: self.food - rhs.food,
            gold: self.gold - rhs.gold,
        }
    }
}

impl SubAssign for Resources {
    fn sub_assign(&mut self, rhs: Self) {
        self.food -= rhs.food;
        self.gold -= rhs.gold;
    }
}

impl Neg for Resources {
    type Output = Resources;

    fn neg(self) -> Self::Output {
        Self {
            food: -self.food,
            gold: -self.gold,
        }
    }
}

impl Mul<f64> for Resources {
    type Output = Resources;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            food: (self.food as f64 * rhs) as i32,
            gold: (self.gold as f64 * rhs) as i32,
        }
    }
}
