use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{self};

use ron::error::SpannedError;

use crate::get_assets_dir;

use super::*;

/// Used only in development, in release builds use `get_assets_dir`
pub const BLUEPRINTS_PATH: &str = "assets/blueprints/";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Blueprints {
    pub terrain: HashMap<TerrainId, TerrainBlueprint>,
    pub units: HashMap<UnitId, UnitBlueprint>,
    pub techs: HashMap<TechId, TechBlueprint>,
    pub abilities: HashMap<AbilityId, AbilityBlueprint>,
    pub powers: HashMap<PowerId, PowerBlueprint>,
    pub base_bonuses: Vec<BattleBonus>,
    pub civilizations: HashMap<CivilizationId, CivilizationBlueprint>,
}

#[derive(Debug)]
pub enum BlueprintLoadError {
    ReadingFile {
        error: io::Error,
        path: String,
        current_dir: String,
    },
    Parsing(SpannedError),
}

impl From<SpannedError> for BlueprintLoadError {
    fn from(value: SpannedError) -> Self {
        Self::Parsing(value)
    }
}

fn read_file(path: &str) -> Result<String, BlueprintLoadError> {
    read_to_string(path).map_err(|io_err| BlueprintLoadError::ReadingFile {
        error: io_err,
        path: path.to_string(),
        current_dir: std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    })
}

impl Blueprints {
    pub fn from_assets() -> Result<Self, BlueprintLoadError> {
        Blueprints::from_assets_location(&format!("{}/blueprints", get_assets_dir()))
    }

    pub fn from_assets_location(base_path: &str) -> Result<Self, BlueprintLoadError> {
        let path = base_path.to_string() + "/terrains.ron";
        let terrain: Vec<TerrainBlueprint> = ron::from_str(&read_file(path.as_str())?)?;
        let path = base_path.to_string() + "/units.ron";
        let units: Vec<UnitBlueprint> = ron::from_str(&read_file(path.as_str())?)?;
        let path = base_path.to_string() + "/techs.ron";
        let techs: Vec<TechBlueprint> = ron::from_str(&read_file(path.as_str())?)?;
        let path = base_path.to_string() + "/abilities.ron";
        let abilities: Vec<AbilityBlueprint> = ron::from_str(&read_file(path.as_str())?)?;
        let path = base_path.to_string() + "/powers.ron";
        let powers: Vec<PowerBlueprint> = ron::from_str(&read_file(path.as_str())?)?;
        let path = base_path.to_string() + "/base_bonuses.ron";
        let base_bonuses: Vec<BattleBonus> = ron::from_str(&read_file(path.as_str())?)?;
        let path = base_path.to_string() + "/civilizations.ron";
        let civilizations: Vec<CivilizationBlueprint> = ron::from_str(&read_file(path.as_str())?)?;
        let bp = Self {
            terrain: terrain
                .into_iter()
                .map(|t| (t.header.id.clone(), t))
                .collect(),
            units: units
                .into_iter()
                .map(|t| (t.header.id.clone(), t))
                .collect(),
            techs: techs.into_iter().map(|t| (t.id.clone(), t)).collect(),
            abilities: abilities.into_iter().map(|t| (t.id.clone(), t)).collect(),
            powers: powers.into_iter().map(|t| (t.id.clone(), t)).collect(),
            base_bonuses,
            civilizations: civilizations
                .into_iter()
                .map(|t| (t.id.clone(), t))
                .collect(),
        };
        let res = &ResolveInto::Id;
        Ok(Self {
            units: bp
                .units
                .clone()
                .into_iter()
                .map(|(id, t)| (id, t.resolve(res, &bp)))
                .collect(),
            techs: bp
                .techs
                .clone()
                .into_iter()
                .map(|(id, t)| (id, t.resolve(res, &bp)))
                .collect(),
            abilities: bp
                .abilities
                .clone()
                .into_iter()
                .map(|(id, t)| (id, t.resolve(res, &bp)))
                .collect(),
            powers: bp
                .powers
                .clone()
                .into_iter()
                .map(|(id, t)| (id, t.resolve(res, &bp)))
                .collect(),
            base_bonuses: bp
                .base_bonuses
                .clone()
                .into_iter()
                .map(|b| b.resolve(res, &bp))
                .collect(),
            civilizations: bp
                .civilizations
                .clone()
                .into_iter()
                .map(|(id, t)| (id, t.resolve(res, &bp)))
                .collect(),
            ..bp
        })
    }

    pub fn to_assets(&self) {
        self.to_assets_location(BLUEPRINTS_PATH)
    }

    // used for testing
    // i plan to use this later for an ingame editor/randomizer
    #[allow(dead_code)]
    pub fn to_assets_location(&self, base_path: &str) {
        let config = ron::ser::PrettyConfig::default()
            .compact_arrays(true)
            .depth_limit(1);

        let res = &ResolveInto::Name;
        let bp = Self {
            units: self
                .units
                .clone()
                .into_iter()
                .map(|t| (t.0, t.1.resolve(res, &self)))
                .collect(),
            techs: self
                .techs
                .clone()
                .into_iter()
                .map(|t| (t.0, t.1.resolve(res, &self)))
                .collect(),
            abilities: self
                .abilities
                .clone()
                .into_iter()
                .map(|t| (t.0, t.1.resolve(res, &self)))
                .collect(),
            powers: self
                .powers
                .clone()
                .into_iter()
                .map(|t| (t.0, t.1.resolve(res, &self)))
                .collect(),
            base_bonuses: self
                .base_bonuses
                .clone()
                .into_iter()
                .map(|b| b.resolve(res, &self))
                .collect(),
            civilizations: self
                .civilizations
                .clone()
                .into_iter()
                .map(|t| (t.0, t.1.resolve(res, &self)))
                .collect(),
            ..self.clone()
        };

        let mut terrains: Vec<TerrainBlueprint> =
            bp.terrain.iter().map(|(_, u)| u.clone().into()).collect();
        terrains.sort_by(|a, b| a.header.id.0.cmp(&b.header.id.0));
        let terrain_string: String = ron::ser::to_string_pretty(&terrains, config.clone()).unwrap();
        std::fs::write(
            base_path.to_string() + "/terrains_serde.ron",
            terrain_string,
        )
        .unwrap();

        let mut units: Vec<UnitBlueprint> = bp.units.iter().map(|(_, t)| t.clone()).collect();
        units.sort_by(|a, b| a.header.id.0.cmp(&b.header.id.0));
        let unit_string: String = ron::ser::to_string_pretty(&units, config.clone()).unwrap();
        std::fs::write(base_path.to_string() + "/units_serde.ron", unit_string).unwrap();

        let mut techs: Vec<TechBlueprint> = bp.techs.iter().map(|(_, t)| t.clone()).collect();
        techs.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        let techs_string: String = ron::ser::to_string_pretty(&techs, config.clone()).unwrap();
        std::fs::write(base_path.to_string() + "/techs_serde.ron", techs_string).unwrap();

        let config_abilities = ron::ser::PrettyConfig::default()
            .compact_arrays(true)
            .depth_limit(2);
        let mut abilities: Vec<AbilityBlueprint> =
            bp.abilities.iter().map(|(_, t)| t.clone()).collect();
        abilities.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        let abilities_string: String =
            ron::ser::to_string_pretty(&abilities, config_abilities.clone()).unwrap();
        std::fs::write(
            base_path.to_string() + "/abilities_serde.ron",
            abilities_string,
        )
        .unwrap();

        let base_bonuses_string: String =
            ron::ser::to_string_pretty(&self.base_bonuses, config_abilities.clone()).unwrap();
        std::fs::write(
            base_path.to_string() + "/base_bonuses_serde.ron",
            base_bonuses_string,
        )
        .unwrap();

        let mut powers: Vec<PowerBlueprint> = bp.powers.iter().map(|(_, t)| t.clone()).collect();
        powers.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        let powers_string: String =
            ron::ser::to_string_pretty(&powers, config_abilities.clone()).unwrap();
        std::fs::write(base_path.to_string() + "/powers.ron", powers_string).unwrap();
    }

    pub fn get<'a>(&'a self, gen_id: &Id) -> Blueprint {
        match gen_id {
            Id::Tech(id) => Blueprint::Tech(self.get_tech(id)),
            Id::Unit(id) => Blueprint::Unit(self.get_unit(id)),
            Id::Ability(id) => Blueprint::Ability(self.get_ability(id)),
            Id::Terrain(id) => Blueprint::Terrain(self.get_terrain(id)),
            Id::Power(id) => Blueprint::Power(self.get_power(id)),
            Id::Civilization(id) => Blueprint::Civilization(self.get_civilization(id)),
        }
    }

    pub fn get_from_name(&self, name: &String) -> Option<Id> {
        self.get_tech_from_name(name)
            .map(|id| Id::Tech(id))
            .or(self.get_unit_from_name(name).map(|id| Id::Unit(id)))
            .or(self.get_ability_from_name(name).map(|id| Id::Ability(id)))
            .or(self.get_terrain_from_name(name).map(|id| Id::Terrain(id)))
            .or(self.get_power_from_name(name).map(|id| Id::Power(id)))
            .or(self
                .get_civilization_from_name(name)
                .map(|id| Id::Civilization(id)))
    }

    pub fn get_terrain<'a>(&'a self, id: &TerrainId) -> &'a TerrainBlueprint {
        match self.terrain.get(id) {
            Some(bp) => &bp,
            None => panic!("no bp for terrain {:?}", id),
        }
    }

    pub fn get_unit<'a>(&'a self, id: &UnitId) -> &'a UnitBlueprint {
        match self.units.get(id) {
            Some(bp) => &bp,
            None => panic!("no bp for unit {:?}", id),
        }
    }

    pub fn get_tech<'a>(&'a self, id: &TechId) -> &'a TechBlueprint {
        match self.techs.get(id) {
            Some(bp) => &bp,
            None => panic!("no bp for tech {:?}", id),
        }
    }

    pub fn get_ability<'a>(&'a self, id: &AbilityId) -> &'a AbilityBlueprint {
        match self.abilities.get(id) {
            Some(bp) => &bp,
            None => panic!("no bp for ability {:?}", id),
        }
    }

    pub fn get_power<'a>(&'a self, id: &PowerId) -> &'a PowerBlueprint {
        match self.powers.get(id) {
            Some(bp) => &bp,
            None => panic!("no bp for power {:?}", id),
        }
    }

    pub fn get_civilization<'a>(&'a self, id: &CivilizationId) -> &'a CivilizationBlueprint {
        match self.civilizations.get(id) {
            Some(bp) => &bp,
            None => panic!("no bp for civ {:?}", id),
        }
    }

    pub fn get_unit_from_name(&self, name: &str) -> Option<UnitId> {
        self.units
            .iter()
            .find(|(_, u)| u.header.name == *name)
            .map(|u| u.0.clone())
    }

    pub fn get_terrain_from_name(&self, name: &str) -> Option<TerrainId> {
        self.terrain
            .iter()
            .find(|(_, u)| u.header.name == *name)
            .map(|u| u.0.clone())
    }

    pub fn get_tech_from_name(&self, name: &str) -> Option<TechId> {
        self.techs
            .iter()
            .find(|(_, u)| u.name == *name)
            .map(|u| u.0.clone())
    }

    pub fn get_ability_from_name(&self, name: &str) -> Option<AbilityId> {
        self.abilities
            .iter()
            .find(|(_, t)| t.name == *name)
            .map(|u| u.0.clone())
    }

    pub fn get_power_from_name(&self, name: &str) -> Option<PowerId> {
        self.powers
            .iter()
            .find(|(_, t)| t.name == *name)
            .map(|u| u.0.clone())
    }

    pub fn get_civilization_from_name(&self, name: &str) -> Option<CivilizationId> {
        self.civilizations
            .iter()
            .find(|(_, t)| t.name == *name)
            .map(|u| u.0.clone())
    }

    pub fn unit_has_ability(&self, unit_id: &UnitId, name: &str) -> bool {
        let unit_bp = self.get_unit(unit_id);
        let ability = self.get_ability_from_name(name);
        unit_bp
            .abilities
            .iter()
            .any(|ab| Some(ab.ability()) == ability.as_ref())
    }
}
