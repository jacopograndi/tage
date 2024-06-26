use crate::{is_default, prelude::*};

/// A thing that sits either the `unit` or the `building` slot in a tile.
/// It has a `blueprint_id` that points to static data such as attack and abilities.
/// This struct holds data that changes at runtime such as health and owner.
/// Buildings are units that sit in the `building` slot.
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
#[serde(default)]
pub struct Unit {
    /// The static data for this unit
    pub blueprint_id: UnitId,

    /// Stays in range (0, 100], when reaches 0 the unit dies.
    /// Damage dealt is multiplied by health unless the unit has Frenzy
    pub health: i32,

    /// Number of battles the unit has taken part
    pub veterancy: i32,

    /// Player owning this unit
    pub owner: PlayerId,

    /// Action counter, each unit has one action and then is done for the turn.
    #[serde(default, skip_serializing_if = "is_default")]
    pub done: bool,

    /// True if the unit has been moved this turn
    #[serde(default, skip_serializing_if = "is_default")]
    pub moved: bool,

    /// If true the unit is considered done.
    /// Resets at the end of day giving +50 health
    /// Buildings without a Villager on top do not finish construction
    #[serde(default, skip_serializing_if = "is_default")]
    pub in_construction: bool,

    /// Units pickup Ruins until the end of day and relics if they are Monks
    #[serde(default, skip_serializing_if = "is_default")]
    pub holding_collectable: Option<Collectable>,

    /// Stores the player attempting the conversion and the conversion strenght
    #[serde(default, skip_serializing_if = "is_default")]
    pub conversion_attempt: Option<(PlayerId, i32)>,

    /// Used to refresh Markets at the start of day with three new units
    #[serde(default, skip_serializing_if = "is_default")]
    pub train_list_override: Vec<UnitId>,

    /// Active powers that currently apply a bonus on the unit, cleared at the end of day
    #[serde(default, skip_serializing_if = "is_default")]
    pub affected_by_powers: Vec<PowerId>,

    /// Multi-tile units store pointers to all other pieces within this vec
    #[serde(default, skip_serializing_if = "is_default")]
    pub linked_units: Vec<IVec2>,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            blueprint_id: UnitId::default(),
            health: 100,
            veterancy: 0,
            done: false,
            moved: false,
            owner: PlayerId::default(),
            in_construction: false,
            holding_collectable: None,
            affected_by_powers: vec![],
            conversion_attempt: None,
            train_list_override: vec![],
            linked_units: vec![],
        }
    }
}

/// Holds both the unit and it's position.
/// Used to refer to a unit and save it's state so that it can be restore by an undo.
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
pub struct UnitTarget {
    pub unit: Unit,
    pub at: IVec2,
}

impl UnitTarget {
    pub fn new(unit: Unit, at: IVec2) -> Self {
        Self { unit, at }
    }
}

/// A unit may be in the `tile.unit` slot or in the `tile.building` slot
/// This enum maps `Top` to the unit slot and `Bot` to the building slot
#[derive(
    Copy,
    Default,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    PartialEq,
    Eq,
    bincode::Encode,
    bincode::Decode,
)]
pub enum UnitLocation {
    #[default]
    Top,
    Bot,
}

/// Completely specify the position of a unit.
#[derive(
    Copy,
    Default,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    PartialEq,
    Eq,
    bincode::Encode,
    bincode::Decode,
)]
pub struct UnitPos {
    pub xy: IVec2,
    pub loc: UnitLocation,
}

impl UnitPos {
    pub fn new(xy: IVec2, loc: UnitLocation) -> Self {
        Self { xy, loc }
    }

    pub fn at(&self, xy: IVec2) -> Self {
        Self {
            xy,
            loc: self.loc.clone(),
        }
    }

    pub fn top(xy: IVec2) -> Self {
        Self {
            xy,
            loc: UnitLocation::Top,
        }
    }

    pub fn bot(xy: IVec2) -> Self {
        Self {
            xy,
            loc: UnitLocation::Bot,
        }
    }
}
