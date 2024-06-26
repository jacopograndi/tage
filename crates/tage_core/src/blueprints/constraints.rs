use super::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub enum UnitConstraint {
    #[default]
    NoConstraint,
    Unit(IdName),
    Class(UnitClass),
    Level(i32),
    Civilization(IdName),
    AnyCivilization,
    Stat(UnitStatsReflect, Compare, i32),
    Or(Vec<UnitConstraint>),
    And(Vec<UnitConstraint>),
    Not(Box<UnitConstraint>),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum BuildConstraint {
    // optimized distance = 1
    IsAdjacentTo(IdName),

    // optimized distance = 2 && x < 2 && y < 2
    IsDiagonalTo(IdName),

    // general purpose distance
    DistanceFrom(IdName, Compare, i32),

    NumberOf(IdName, Compare, i32),
    OnTerrain(IdName),
    OnlyOnFoodResource,
    OnlyOnGoldResource,
    NotOnFoodResource,
    NotOnGoldResource,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Compare {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
}

impl Compare {
    pub fn compare(&self, a: i32, b: i32) -> bool {
        match self {
            Compare::GreaterThan => a > b,
            Compare::LessThan => a < b,
            Compare::Equal => a == b,
            Compare::NotEqual => a != b,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum UnitStatsReflect {
    Attack,
    Defence,
    Sight,
    Movement,
    Range,
}

impl UnitStatsReflect {
    pub fn iter<'a>() -> impl Iterator<Item = &'a UnitStatsReflect> {
        [
            Self::Attack,
            Self::Defence,
            Self::Sight,
            Self::Movement,
            Self::Range,
        ]
        .iter()
    }
}

impl Resolve for UnitConstraint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        match self {
            UnitConstraint::Unit(id) => UnitConstraint::Unit(id.resolve(res, bp)),
            UnitConstraint::Or(list) => {
                UnitConstraint::Or(list.into_iter().map(|e| e.resolve(res, bp)).collect())
            }
            UnitConstraint::And(list) => {
                UnitConstraint::And(list.into_iter().map(|e| e.resolve(res, bp)).collect())
            }
            _ => self,
        }
    }
}

impl UnitConstraint {
    pub fn satisfied(&self, bp: &Blueprints, unit: &UnitBlueprint) -> bool {
        match self {
            UnitConstraint::NoConstraint => true,
            UnitConstraint::Unit(id) => unit.header.id == id.id().into(),
            UnitConstraint::Class(class) => unit.header.class == *class,
            UnitConstraint::Level(lv) => unit.header.level == *lv,
            UnitConstraint::Civilization(id) => unit.required_civilization.contains(&id),
            UnitConstraint::AnyCivilization => !unit.required_civilization.is_empty(),
            UnitConstraint::Stat(stat, cmp, val) => cmp.compare(unit.stats.get_stat(&stat), *val),
            UnitConstraint::Or(list) => list
                .iter()
                .fold(false, |acc, c| acc || c.satisfied(bp, unit)),
            UnitConstraint::And(list) => list
                .iter()
                .fold(true, |acc, c| acc && c.satisfied(bp, unit)),
            UnitConstraint::Not(constr) => !constr.satisfied(bp, unit),
        }
    }
}

impl Resolve for BuildConstraint {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        match self {
            Self::IsAdjacentTo(id) => Self::IsAdjacentTo(id.resolve(res, bp)),
            Self::IsDiagonalTo(id) => Self::IsDiagonalTo(id.resolve(res, bp)),
            Self::DistanceFrom(id, comp, val) => Self::DistanceFrom(id.resolve(res, bp), comp, val),
            Self::OnTerrain(id) => Self::OnTerrain(id.resolve(res, bp)),
            Self::NumberOf(id, comp, val) => Self::NumberOf(id.resolve(res, bp), comp, val),
            _ => self,
        }
    }
}

impl UnitConstraint {
    pub fn view(&self, bp: &Blueprints) -> String {
        match self {
            UnitConstraint::NoConstraint => format!("No requirements"),
            UnitConstraint::Unit(id) => {
                format!("{}", bp.get_unit(id.unit()).header.name)
            }
            UnitConstraint::Class(class) => {
                format!("{}", class.view())
            }
            UnitConstraint::Level(level) => {
                format!("{} unit", view_level(*level))
            }
            UnitConstraint::Civilization(id) => {
                format!(
                    "unit of civilization {}",
                    bp.get_civilization(id.civilization()).name
                )
            }
            UnitConstraint::AnyCivilization => {
                format!("civilization specific unit")
            }
            UnitConstraint::Stat(stat, comp, val) => {
                format!("unit with {} {} {}", stat.view(), comp.view(), val)
            }
            UnitConstraint::Or(list) => {
                let mut s = String::new();
                for (i, constr) in list.iter().map(|e| e.view(bp)).enumerate() {
                    if (1..list.len()).contains(&i) {
                        s += " or ";
                    }
                    s += &constr;
                }
                s
            }
            UnitConstraint::And(list) => {
                let mut s = String::new();
                for (i, constr) in list.iter().map(|e| e.view(bp)).enumerate() {
                    if (1..list.len()).contains(&i) {
                        s += " and ";
                    }
                    s += &constr;
                }
                s
            }
            UnitConstraint::Not(constr) => format!("No {}", constr.view(bp)),
        }
    }
}

impl BuildConstraint {
    pub fn view(&self, bp: &Blueprints) -> String {
        match self {
            BuildConstraint::IsAdjacentTo(unit_id) => format!(
                "Must be adjacent to your {}",
                bp.get_unit(unit_id.unit()).header.name
            ),
            BuildConstraint::IsDiagonalTo(unit_id) => format!(
                "Must be diagonal to your {}",
                bp.get_unit(unit_id.unit()).header.name
            ),
            BuildConstraint::DistanceFrom(unit_id, comp, val) => format!(
                "Distance from {} must be {} {}",
                bp.get_unit(unit_id.unit()).header.name,
                comp.view(),
                val
            ),
            BuildConstraint::OnTerrain(terrain_id) => format!(
                "Must be on {}",
                bp.get_terrain(terrain_id.terrain()).header.name
            ),
            BuildConstraint::NumberOf(unit_id, comp, val) => format!(
                "Number of {} must be {} {}",
                bp.get_unit(unit_id.unit()).header.name,
                comp.view(),
                val
            ),
            BuildConstraint::OnlyOnFoodResource => format!("Must be on a food resource"),
            BuildConstraint::OnlyOnGoldResource => format!("Must be on a gold resource"),
            BuildConstraint::NotOnFoodResource => format!("Must not be on a food resource"),
            BuildConstraint::NotOnGoldResource => format!("Must not be on a food resource"),
        }
        .to_string()
    }
}

impl UnitStatsReflect {
    pub fn view(&self) -> String {
        match self {
            UnitStatsReflect::Attack => "Attack",
            UnitStatsReflect::Defence => "Defence",
            UnitStatsReflect::Sight => "Sight",
            UnitStatsReflect::Movement => "Movement",
            UnitStatsReflect::Range => "Range",
        }
        .to_string()
    }
}

impl Compare {
    pub fn view(&self) -> String {
        match self {
            Compare::GreaterThan => ">",
            Compare::LessThan => "<",
            Compare::Equal => "=",
            Compare::NotEqual => "not equal",
        }
        .to_string()
    }
}
