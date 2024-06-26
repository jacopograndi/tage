use super::*;
use crate::is_default;

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
pub struct PowerId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct PowerBlueprint {
    pub id: PowerId,
    pub name: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub require_on_building: UnitConstraint,

    #[serde(default, skip_serializing_if = "is_default")]
    pub targets: PowerTargets,

    #[serde(default, skip_serializing_if = "is_default")]
    pub bonus: Bonus,

    #[serde(default, skip_serializing_if = "is_default")]
    pub unit_bonus: UnitBonus,

    #[serde(default, skip_serializing_if = "is_default")]
    pub battle_bonus: BattleBonus,

    #[serde(default, skip_serializing_if = "is_default")]
    pub effects: Vec<PowerEffect>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct PowerTargets {
    pub status: PowerTargetStatus,
    pub location: Vec<PowerTargetLocation>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub enum PowerTargetStatus {
    Friendly,
    Enemy,
    #[default]
    Any,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PowerTargetLocation {
    This,
    Adjacent,
    Diagonal,
    InSight,
    All,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PowerEffect {
    Heal(i32),
    ProduceResources(Resources),
    TechDiscount(Resources),
    TrainDiscount(Resources),
}

impl Resolve for PowerBlueprint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        Self {
            require_on_building: self.require_on_building.resolve(res, bp),
            battle_bonus: self.battle_bonus.resolve(res, bp),
            ..self
        }
    }
}

impl PowerTargets {
    pub fn view(&self) -> String {
        let mut s = String::new();
        for (i, location) in self.location.iter().enumerate() {
            if (1..self.location.len()).contains(&i) {
                s += " and ";
            }
            let status = match self.status {
                PowerTargetStatus::Friendly => format!(" friendly "),
                PowerTargetStatus::Enemy => format!(" hostile "),
                PowerTargetStatus::Any => format!(" "),
            };
            match location {
                PowerTargetLocation::This => s = format!("{s}this{status}unit"),
                PowerTargetLocation::Adjacent => s = format!("{s}adjacent{status}units"),
                PowerTargetLocation::Diagonal => s = format!("{s}diagonal{status}units"),
                PowerTargetLocation::InSight => {
                    s = format!(
                        "{s}{}units in sight",
                        status.chars().skip(1).collect::<String>()
                    )
                }
                PowerTargetLocation::All => s = format!("all{status}units"),
            };
        }
        s + "\n"
    }
}

impl PowerEffect {
    pub fn view(&self) -> String {
        match self {
            PowerEffect::Heal(val) => format!("Heal {}\n", val),
            PowerEffect::ProduceResources(resources) => {
                let mut s = String::new();
                for res in Resource::iter() {
                    let val = resources.get_res(res);
                    if val != Resources::default().get_res(res) {
                        s += &format!("Gain {} {}\n", val, res.view(),);
                    }
                }
                s
            }
            PowerEffect::TechDiscount(resources) => {
                let mut s = String::new();
                for res in Resource::iter() {
                    let val = resources.get_res(res);
                    if val != Resources::default().get_res(res) {
                        s += &format!("Research discount: {} {}\n", val, res.view(),);
                    }
                }
                s
            }
            PowerEffect::TrainDiscount(resources) => {
                let mut s = String::new();
                for res in Resource::iter() {
                    let val = resources.get_res(res);
                    if val != Resources::default().get_res(res) {
                        s += &format!("Unit train discount: {} {}\n", val, res.view(),);
                    }
                }
                s
            }
        }
    }
}
