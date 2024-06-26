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
pub struct AbilityId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct AbilityBlueprint {
    pub id: AbilityId,
    pub name: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub battle_bonuses: Vec<BattleBonus>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub unit_bonuses: Vec<Bonus>,
}

impl Resolve for AbilityBlueprint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> AbilityBlueprint {
        Self {
            battle_bonuses: self
                .battle_bonuses
                .into_iter()
                .map(|b| b.resolve(res, bp))
                .collect(),
            unit_bonuses: self
                .unit_bonuses
                .into_iter()
                .map(|b| b.resolve(res, bp))
                .collect(),
            ..self
        }
    }
}
