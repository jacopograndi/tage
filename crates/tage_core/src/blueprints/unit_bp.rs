use super::*;
use crate::is_default;
use std::ops::Add;

#[derive(
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Default,
    PartialEq,
    Eq,
    Clone,
    Hash,
    bincode::Encode,
    bincode::Decode,
)]
pub struct UnitId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UnitBlueprint {
    pub header: UnitHeader,
    pub stats: UnitStats,

    #[serde(default, skip_serializing_if = "is_default")]
    pub resources: UnitResources,

    #[serde(default, skip_serializing_if = "is_default")]
    pub upgrades_to: Option<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub train_list: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub build_list: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub build_constraints: Vec<BuildConstraint>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub defence_bonus_to_unit_on_top: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub defence_bonus_to_adjacent_buildings: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub abilities: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub required_tech: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub required_civilization: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub powers: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub unit_size: UnitSize,

    #[serde(default, skip_serializing_if = "is_default")]
    pub train_cost_bonus: Bonus,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct UnitSize {
    pub size: i32,
}

impl Default for UnitSize {
    fn default() -> Self {
        Self { size: 1 }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq, Clone)]
pub enum UnitClass {
    #[default]
    Inf,
    Bld,
    Cav,
    Sie,
    Ran,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UnitHeader {
    pub id: UnitId,
    pub name: String,
    pub glyph: String,
    pub class: UnitClass,
    pub level: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UnitStats {
    #[serde(default, skip_serializing_if = "is_default")]
    pub movement: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub attack: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub defence: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub range: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub sight: i32,
}

impl UnitStats {
    pub fn get_stat(&self, stat: &UnitStatsReflect) -> i32 {
        match stat {
            UnitStatsReflect::Attack => self.attack,
            UnitStatsReflect::Defence => self.defence,
            UnitStatsReflect::Sight => self.sight,
            UnitStatsReflect::Movement => self.movement,
            UnitStatsReflect::Range => self.range,
        }
    }

    pub fn apply(&self, bonus: Bonus) -> UnitStats {
        Self {
            movement: apply(
                self.movement,
                bonus.incr.stats.movement,
                bonus.perc.stats.movement,
            ),
            attack: apply(
                self.attack,
                bonus.incr.stats.attack,
                bonus.perc.stats.attack,
            ),
            defence: apply(
                self.defence,
                bonus.incr.stats.defence,
                bonus.perc.stats.defence,
            ),
            sight: apply(self.sight, bonus.incr.stats.sight, bonus.perc.stats.sight).max(1),
            range: apply(self.range, bonus.incr.stats.range, bonus.perc.stats.range).max(1),
        }
    }
}

impl Add for UnitStats {
    type Output = UnitStats;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            movement: self.movement + rhs.movement,
            attack: self.attack + rhs.attack,
            defence: self.defence + rhs.defence,
            range: self.range + rhs.range,
            sight: self.sight + rhs.sight,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UnitResources {
    #[serde(default, skip_serializing_if = "is_default")]
    pub produces: Resources,
    #[serde(default, skip_serializing_if = "is_default")]
    pub cost: Resources,
}

impl UnitResources {
    pub fn apply(&self, bonus: Bonus) -> UnitResources {
        Self {
            produces: self.produces.apply_produces(bonus.clone()),
            cost: self.cost.apply_cost(bonus),
        }
    }

    pub fn cost(cost: Resources) -> UnitResources {
        UnitResources {
            cost,
            ..Default::default()
        }
    }
    pub fn produces(produces: Resources) -> UnitResources {
        UnitResources {
            produces,
            ..Default::default()
        }
    }
}

impl Add for UnitResources {
    type Output = UnitResources;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            produces: self.produces + rhs.produces,
            cost: self.cost + rhs.cost,
        }
    }
}

impl Resolve for UnitBlueprint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        Self {
            upgrades_to: self.upgrades_to.resolve(res, bp),
            train_list: self.train_list.resolve(res, bp),
            build_list: self.build_list.resolve(res, bp),
            abilities: self.abilities.resolve(res, bp),
            required_tech: self.required_tech.resolve(res, bp),
            build_constraints: self
                .build_constraints
                .into_iter()
                .map(|c| c.resolve(res, bp))
                .collect(),
            powers: self
                .powers
                .into_iter()
                .map(|p| p.resolve(res, bp))
                .collect(),
            required_civilization: self
                .required_civilization
                .into_iter()
                .map(|p| p.resolve(res, bp))
                .collect(),
            ..self
        }
    }
}

impl UnitClass {
    pub fn view(&self) -> String {
        match self {
            UnitClass::Inf => "Infantry",
            UnitClass::Bld => "Building",
            UnitClass::Cav => "Cavalry",
            UnitClass::Sie => "Siege",
            UnitClass::Ran => "Ranged",
        }
        .to_string()
    }
}
