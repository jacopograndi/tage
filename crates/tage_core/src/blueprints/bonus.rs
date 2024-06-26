use crate::is_default;
use std::{iter::Sum, ops::Add};

use super::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Bonus {
    #[serde(default, skip_serializing_if = "is_default")]
    pub incr: BonusValue,

    #[serde(default, skip_serializing_if = "is_default")]
    pub perc: BonusValue,

    #[serde(default, skip_serializing_if = "is_default")]
    pub trade: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub heal: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub convert: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub attack_priority: i32,

    #[serde(default, skip_serializing_if = "is_default")]
    pub forbid_attack: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    pub forbid_attack_after_move: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    pub forbid_counterattack: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    pub can_attack_twice: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    pub terrain_movement_cost_override: Vec<(IdName, i32)>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct BattleBonus {
    #[serde(default, skip_serializing_if = "is_default")]
    pub require_this: UnitConstraint,

    #[serde(default, skip_serializing_if = "is_default")]
    pub require_opponent: UnitConstraint,

    #[serde(default, skip_serializing_if = "is_default")]
    pub require_terrain: Vec<IdName>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub require_distance: Vec<(Compare, i32)>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub target: BattleBonusTarget,

    pub bonus: Bonus,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct UnitBonus {
    #[serde(default, skip_serializing_if = "is_default")]
    pub affects: UnitConstraint,

    #[serde(default, skip_serializing_if = "is_default")]
    pub bonus: Bonus,
}

pub fn apply(val: i32, incr: i32, perc: i32) -> i32 {
    ((val + incr) as f64 * ((perc + 100) as f64 * 0.01)).max(0.0) as i32
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct BonusValue {
    #[serde(default, skip_serializing_if = "is_default")]
    pub stats: UnitStats,

    #[serde(default, skip_serializing_if = "is_default")]
    pub resources: UnitResources,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, PartialEq, Eq, Clone, Hash)]
pub enum BattleBonusTarget {
    #[default]
    This,
    Opponent,
}

impl Add for BonusValue {
    type Output = BonusValue;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            stats: self.stats + rhs.stats,
            resources: self.resources + rhs.resources,
        }
    }
}

impl Add for Bonus {
    type Output = Bonus;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            incr: self.incr + rhs.incr,
            perc: self.perc + rhs.perc,
            trade: self.trade + rhs.trade,
            heal: self.heal + rhs.heal,
            convert: self.convert + rhs.convert,
            attack_priority: self.attack_priority + rhs.attack_priority,
            forbid_attack: self.forbid_attack || rhs.forbid_attack,
            forbid_attack_after_move: self.forbid_attack_after_move || rhs.forbid_attack_after_move,
            forbid_counterattack: self.forbid_counterattack || rhs.forbid_counterattack,
            can_attack_twice: self.can_attack_twice || rhs.can_attack_twice,
            terrain_movement_cost_override: self
                .terrain_movement_cost_override
                .into_iter()
                .chain(rhs.terrain_movement_cost_override.into_iter())
                .collect(),
        }
    }
}

impl Sum for Bonus {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Bonus::default(), |acc, b| b + acc)
    }
}

impl Resolve for Bonus {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> Self {
        Self {
            terrain_movement_cost_override: self
                .terrain_movement_cost_override
                .into_iter()
                .map(|(id, ov)| (id.resolve(res, bp), ov))
                .collect(),
            ..self
        }
    }
}

impl Resolve for BattleBonus {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> BattleBonus {
        Self {
            require_this: self.require_this.resolve(res, bp),
            require_opponent: self.require_opponent.resolve(res, bp),
            require_terrain: self.require_terrain.resolve(res, bp),
            bonus: self.bonus.resolve(res, bp),
            ..self
        }
    }
}
impl Resolve for UnitBonus {
    fn resolve(self, res: &ResolveInto, bp: &Blueprints) -> UnitBonus {
        Self {
            affects: self.affects.resolve(res, bp),
            bonus: self.bonus.resolve(res, bp),
        }
    }
}

pub fn view_incr(val: i32) -> String {
    format!("{}{}", if val.is_positive() { "+" } else { "" }, val)
}

impl Bonus {
    pub fn view(&self, bp: &Blueprints) -> String {
        let mut s = String::new();
        let def = Bonus::default();
        for stat in UnitStatsReflect::iter() {
            let val = self.incr.stats.get_stat(stat);
            if val != def.incr.stats.get_stat(stat) {
                s += &format!(
                    "{} {}\n",
                    stat.view(),
                    format!("{}{}", if val.is_positive() { "+" } else { "" }, val)
                );
            }
        }
        for stat in UnitStatsReflect::iter() {
            let val = self.perc.stats.get_stat(stat);
            if val != def.perc.stats.get_stat(stat) {
                s += &format!(
                    "{} {}\n",
                    stat.view(),
                    format!("{}{}%", if val.is_positive() { "+" } else { "" }, val)
                );
            }
        }
        for res in Resource::iter() {
            let val = self.incr.resources.cost.get_res(res);
            if val != def.incr.resources.cost.get_res(res) {
                s += &format!(
                    "Costs {} {}\n",
                    format!("{}{}", if val.is_positive() { "+" } else { "" }, val),
                    res.view(),
                );
            }
        }
        for res in Resource::iter() {
            let val = self.incr.resources.produces.get_res(res);
            if val != def.incr.resources.produces.get_res(res) {
                s += &format!(
                    "Production {} {}\n",
                    format!("{}{}", if val.is_positive() { "+" } else { "" }, val),
                    res.view(),
                );
            }
        }
        for res in Resource::iter() {
            let val = self.perc.resources.cost.get_res(res);
            if val != def.perc.resources.cost.get_res(res) {
                s += &format!(
                    "Costs {} {}\n",
                    format!("{}{}%", if val.is_positive() { "+" } else { "" }, val),
                    res.view(),
                );
            }
        }
        for res in Resource::iter() {
            let val = self.perc.resources.produces.get_res(res);
            if val != def.perc.resources.produces.get_res(res) {
                s += &format!(
                    "Production {} {}\n",
                    format!("{}{}%", if val.is_positive() { "+" } else { "" }, val),
                    res.view(),
                );
            }
        }
        if self.trade != def.trade {
            s += &format!("Improved trade rate by {}\n", self.trade * 25);
        }
        if self.heal != def.heal {
            s += &format!("Improved heal rate by {}\n", self.heal * 10);
        }
        if self.convert != def.convert {
            s += &format!("Improved conversion chance\n");
        }
        if self.attack_priority != def.attack_priority {
            s += &format!("Alwaya attacks first\n");
        }
        if self.forbid_attack != def.forbid_attack {
            s += &format!("Cannot attack\n");
        }
        if self.forbid_attack_after_move != def.forbid_attack_after_move {
            s += &format!("Cannot attack after moving\n");
        }
        if self.forbid_counterattack != def.forbid_counterattack {
            s += &format!("Cannot counterattack\n");
        }
        if self.can_attack_twice != def.can_attack_twice {
            s += &format!("Double attack\n");
        }
        if self.terrain_movement_cost_override != def.terrain_movement_cost_override {
            for (terrain_id, val) in self.terrain_movement_cost_override.iter() {
                s += &format!(
                    "Costs {} to move through {}\n",
                    val,
                    bp.get_terrain(terrain_id.terrain()).header.name
                );
            }
        }
        s
    }
}

impl UnitBonus {
    pub fn view(&self, bp: &Blueprints) -> String {
        let mut s = String::new();
        let def = UnitBonus::default();
        if self.affects != def.affects {
            s += &format!("Affects {}\n", self.affects.view(bp));
        }
        s += &self.bonus.view(bp);
        s
    }
}

impl BattleBonus {
    pub fn view(&self, bp: &Blueprints) -> String {
        let mut s = String::new();
        let def = BattleBonus::default();
        if *self != def {
            s += &format!("During a battle:\n");
        }
        if self.require_this != def.require_this {
            s += &format!("If the unit is {}\n", self.require_this.view(bp));
        }
        if self.require_opponent != def.require_opponent {
            s += &format!("If the opponent is {}\n", self.require_opponent.view(bp));
        }
        if self.require_terrain != def.require_terrain {
            for terrain_id in self.require_terrain.iter() {
                s += &format!(
                    "If the battlefield is {}\n",
                    bp.get_terrain(terrain_id.terrain()).header.name
                );
            }
        }
        if self.require_distance != def.require_distance {
            for (comp, val) in self.require_distance.iter() {
                s += &format!(
                    "If the distance from the opponent is {} {}\n",
                    comp.view(),
                    val
                );
            }
        }
        s += &match self.target {
            BattleBonusTarget::This => format!(""),
            BattleBonusTarget::Opponent => format!("Affects the opponent\n"),
        };
        s += &self.bonus.view(bp);
        s
    }
}

impl From<UnitStats> for Bonus {
    fn from(value: UnitStats) -> Self {
        Bonus {
            perc: BonusValue {
                stats: value,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
