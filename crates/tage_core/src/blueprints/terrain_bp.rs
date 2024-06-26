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
pub struct TerrainId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct TerrainBlueprint {
    pub header: TerrainHeader,
    pub stats: TerrainStats,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct TerrainHeader {
    pub id: TerrainId,
    pub name: String,
    pub glyph: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct TerrainStats {
    pub move_cost: i32,
    pub sight_cost: i32,
    pub range_bonus: i32,
    pub defence_bonus: i32,
    pub sight_bonus: i32,
}
