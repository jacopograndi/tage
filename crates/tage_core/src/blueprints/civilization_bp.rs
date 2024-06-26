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
pub struct CivilizationId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct CivilizationBlueprint {
    pub id: CivilizationId,
    pub name: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub unit_bonuses: Vec<UnitBonus>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub tech_discount: Resources,

    #[serde(default, skip_serializing_if = "is_default")]
    pub heroes: Vec<IdName>,
}

impl Resolve for CivilizationBlueprint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> CivilizationBlueprint {
        Self {
            unit_bonuses: self
                .unit_bonuses
                .into_iter()
                .map(|b| b.resolve(res, bp))
                .collect(),
            heroes: self
                .heroes
                .into_iter()
                .map(|b| b.resolve(res, bp))
                .collect(),
            ..self
        }
    }
}
