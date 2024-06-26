use std::ops::Deref;

use super::*;

pub trait Resolve {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self;
}

pub enum ResolveInto {
    Id,
    Name,
}

#[derive(
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Eq,
    Clone,
    Hash,
    bincode::Encode,
    bincode::Decode,
)]
pub enum Id {
    Tech(TechId),
    Unit(UnitId),
    Ability(AbilityId),
    Terrain(TerrainId),
    Power(PowerId),
    Civilization(CivilizationId),
}

impl From<Id> for UnitId {
    fn from(value: Id) -> Self {
        match value {
            Id::Unit(id) => id,
            _ => panic!("id of incorrect type"),
        }
    }
}

impl From<Id> for TechId {
    fn from(value: Id) -> Self {
        match value {
            Id::Tech(id) => id,
            _ => panic!("id of incorrect type"),
        }
    }
}

impl From<Id> for AbilityId {
    fn from(value: Id) -> Self {
        match value {
            Id::Ability(id) => id,
            _ => panic!("id of incorrect type"),
        }
    }
}

impl From<Id> for TerrainId {
    fn from(value: Id) -> Self {
        match value {
            Id::Terrain(id) => id,
            _ => panic!("id of incorrect type"),
        }
    }
}

impl From<Id> for PowerId {
    fn from(value: Id) -> Self {
        match value {
            Id::Power(id) => id,
            _ => panic!("id of incorrect type"),
        }
    }
}

impl From<Id> for CivilizationId {
    fn from(value: Id) -> Self {
        match value {
            Id::Civilization(id) => id,
            _ => panic!("id of incorrect type"),
        }
    }
}

impl From<UnitId> for Id {
    fn from(value: UnitId) -> Self {
        Id::Unit(value)
    }
}

impl From<TechId> for Id {
    fn from(value: TechId) -> Self {
        Id::Tech(value)
    }
}

impl From<AbilityId> for Id {
    fn from(value: AbilityId) -> Self {
        Id::Ability(value)
    }
}

impl From<PowerId> for Id {
    fn from(value: PowerId) -> Self {
        Id::Power(value)
    }
}

impl From<TerrainId> for Id {
    fn from(value: TerrainId) -> Self {
        Id::Terrain(value)
    }
}

impl From<CivilizationId> for Id {
    fn from(value: CivilizationId) -> Self {
        Id::Civilization(value)
    }
}

pub enum Blueprint<'a> {
    Tech(&'a TechBlueprint),
    Unit(&'a UnitBlueprint),
    Ability(&'a AbilityBlueprint),
    Terrain(&'a TerrainBlueprint),
    Power(&'a PowerBlueprint),
    Civilization(&'a CivilizationBlueprint),
}

impl<'a> Blueprint<'a> {
    pub fn get_name(&self) -> &'a str {
        match self {
            Blueprint::Tech(tech) => tech.name.as_str(),
            Blueprint::Unit(unit) => unit.header.name.as_str(),
            Blueprint::Ability(ab) => ab.name.as_str(),
            Blueprint::Terrain(terrain) => terrain.header.name.as_str(),
            Blueprint::Power(power) => power.name.as_str(),
            Blueprint::Civilization(civ) => civ.name.as_str(),
        }
    }
}

// ref

#[derive(
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Eq,
    Clone,
    Hash,
    bincode::Encode,
    bincode::Decode,
)]
pub enum IdName {
    Id(Id),
    Name(String),
}

impl Deref for IdName {
    type Target = Id;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Id(id) => id,
            _ => panic!("ids have to be resolved by now"),
        }
    }
}

impl Resolve for IdName {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        match res {
            ResolveInto::Id => self.to_id(bp),
            ResolveInto::Name => self.to_name(bp),
        }
    }
}

impl From<IdName> for UnitId {
    fn from(value: IdName) -> Self {
        value.id().into()
    }
}
impl From<IdName> for TechId {
    fn from(value: IdName) -> Self {
        value.id().into()
    }
}
impl From<IdName> for AbilityId {
    fn from(value: IdName) -> Self {
        value.id().into()
    }
}
impl From<IdName> for TerrainId {
    fn from(value: IdName) -> Self {
        value.id().into()
    }
}
impl From<IdName> for PowerId {
    fn from(value: IdName) -> Self {
        value.id().into()
    }
}
impl From<IdName> for CivilizationId {
    fn from(value: IdName) -> Self {
        value.id().into()
    }
}

impl IdName {
    pub fn id(&self) -> Id {
        match self {
            IdName::Id(id) => id.clone(),
            _ => panic!("id must be resolved"),
        }
    }

    pub fn id_ref(&self) -> &Id {
        match self {
            IdName::Id(id) => id,
            _ => panic!("id must be resolved"),
        }
    }

    pub fn unit(&self) -> &UnitId {
        match self.id_ref() {
            Id::Unit(ref id) => id,
            _ => panic!(),
        }
    }
    pub fn tech(&self) -> &TechId {
        match self.id_ref() {
            Id::Tech(ref id) => id,
            _ => panic!(),
        }
    }
    pub fn terrain(&self) -> &TerrainId {
        match self.id_ref() {
            Id::Terrain(ref id) => id,
            _ => panic!(),
        }
    }
    pub fn ability(&self) -> &AbilityId {
        match self.id_ref() {
            Id::Ability(ref id) => id,
            _ => panic!(),
        }
    }
    pub fn power(&self) -> &PowerId {
        match self.id_ref() {
            Id::Power(ref id) => id,
            _ => panic!(),
        }
    }
    pub fn civilization(&self) -> &CivilizationId {
        match self.id_ref() {
            Id::Civilization(ref id) => id,
            _ => panic!(),
        }
    }

    fn to_id(self, bp: &Blueprints) -> Self {
        match self {
            IdName::Name(name) => match bp.get_from_name(&name) {
                Some(id) => Self::Id(id),
                None => panic!("nothing named {}", name),
            },
            _ => self,
        }
    }

    fn to_name(self, bp: &Blueprints) -> Self {
        match self {
            IdName::Id(id) => match bp.get(&id) {
                Blueprint::Tech(tech) => Self::Name(tech.name.clone()),
                Blueprint::Unit(unit) => Self::Name(unit.header.name.clone()),
                Blueprint::Ability(ability) => Self::Name(ability.name.clone()),
                Blueprint::Terrain(terrain) => Self::Name(terrain.header.name.clone()),
                Blueprint::Power(power) => Self::Name(power.name.clone()),
                Blueprint::Civilization(civ) => Self::Name(civ.name.clone()),
            },
            _ => self,
        }
    }
}

impl Resolve for Vec<IdName> {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        self.into_iter().map(|e| e.resolve(res, bp)).collect()
    }
}

impl Resolve for Option<IdName> {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        self.map(|e| e.resolve(res, bp))
    }
}
