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
pub struct TechId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct TechBlueprint {
    pub id: TechId,
    pub name: String,
    pub cost: Resources,
    pub level: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub require: UnitConstraint,

    #[serde(default, skip_serializing_if = "is_default")]
    pub unit_bonuses: Vec<UnitBonus>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub battle_bonuses: Vec<BattleBonus>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub trained_from_bonus: Vec<(IdName, Bonus)>,
}

impl Resolve for TechBlueprint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> TechBlueprint {
        Self {
            require: self.require.resolve(res, bp),
            unit_bonuses: self
                .unit_bonuses
                .into_iter()
                .map(|b| b.resolve(res, bp))
                .collect(),
            battle_bonuses: self
                .battle_bonuses
                .into_iter()
                .map(|b| b.resolve(res, bp))
                .collect(),
            trained_from_bonus: self
                .trained_from_bonus
                .into_iter()
                .map(|(id, b)| (id.resolve(res, bp), b))
                .collect(),
            ..self
        }
    }
}
