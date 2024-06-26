use crate::prelude::*;
use tracing::error;

#[derive(Debug, Clone)]
pub struct ActAttack {
    pub this: UnitTarget,
    pub target: UnitTarget,
}

impl Act for ActAttack {
    type Precondition = UnitTarget;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        let Some(unit) = board.get_unit(&UnitPos::top(pre.at)) else {
            return vec![];
        };

        let bonus = board.get_player_bonus(&unit.owner, Some(&unit.blueprint_id))
            + board.get_unit_bonus(&unit.blueprint_id)
            + board.get_power_bonus(unit);

        if !bonus.forbid_attack && !(bonus.forbid_attack_after_move && unit.moved) {
            let range = ActAttack::get_range(board, &pre);
            board
                .get_units_in_range(pre.at, range)
                .filter(|(at, target_unit)| {
                    board.grid.get_at(at).get_top_unit() == Some(target_unit)
                })
                .filter_map(|(at, target_unit)| {
                    let act = ActAttack {
                        this: pre.clone(),
                        target: UnitTarget::new(target_unit.clone(), at),
                    };
                    act.is_valid(board).then_some(act)
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn is_valid(&self, board: &Board) -> bool {
        let upos = board.get_target_pos(&self.this);
        if matches!(upos.loc, UnitLocation::Bot) {
            return false;
        }
        let Some(this_unit) = board.get_unit(&upos) else {
            return false;
        };
        let Some(target_unit) = board.get_unit(&board.get_target_pos(&self.target)) else {
            return false;
        };
        let this_player = board.get_player(&this_unit.owner);
        let target_player = board.get_player(&target_unit.owner);
        let distance = (self.this.at - self.target.at).length();
        let in_range = distance <= ActAttack::get_range(board, &self.this);
        this_player.is_hostile(target_player) && in_range
    }

    fn apply(&self, board: &mut Board) {
        let atk_pos = self.this.at;
        let def_pos = self.target.at;

        let atk_tile = board.grid.get_at(&atk_pos);
        let def_tile = board.grid.get_at(&def_pos);

        let Some(atk_unit) = atk_tile.get_top_unit() else {
            error!(target: "act.attack", "the attacker unit doesn't exist (act: {:?}, dump: {})", self, board.view());
            return;
        };
        let atk_bp = board.bp.as_ref().get_unit(&atk_unit.blueprint_id);

        let Some(def_unit) = def_tile.get_top_unit() else {
            error!(target: "act.attack", "the defender unit doesn't exist (act: {:?}, dump: {})", self, board.view());
            return;
        };
        let def_bp = board.bp.as_ref().get_unit(&def_unit.blueprint_id);

        if board.fog_base == FogTile::Visible {
            // fog can lead to invalid moves
            assert_eq!(
                *atk_unit,
                self.this.unit,
                "the saved unit isn't the same as the unit on the board (act: {:?}, dump: {})",
                self,
                board.view()
            );

            assert_eq!(
                *def_unit,
                self.target.unit,
                "the saved unit isn't the same as the unit on the board (act: {:?}, dump: {})",
                self,
                board.view()
            );
        }

        let atk_has_relic = atk_unit.holding_collectable == Some(Collectable::Relic);
        let def_has_relic = def_unit.holding_collectable == Some(Collectable::Relic);

        let distance = (atk_pos - def_pos).length();

        let (atk_bonus, def_bonus) = self.get_attack_bonuses(board);

        let outcome = if atk_bonus.attack_priority + 1 > def_bonus.attack_priority {
            ActAttack::battle(
                board.bp(),
                atk_unit,
                def_unit,
                atk_bp,
                def_bp,
                atk_bonus,
                def_bonus,
                distance,
                false,
            )
        } else {
            let BattleOutcome { atk, def, steps } = ActAttack::battle(
                board.bp(),
                def_unit,
                atk_unit,
                def_bp,
                atk_bp,
                def_bonus,
                atk_bonus,
                distance,
                false,
            );
            BattleOutcome {
                atk: def,
                def: atk,
                steps,
            }
        };

        let BattleOutcome {
            atk: atk_damaged,
            def: def_damaged,
            ..
        } = outcome;

        let atk_linked = atk_unit.linked_units.clone();
        let def_linked = def_unit.linked_units.clone();
        for linked in atk_linked.iter() {
            board
                .grid
                .get_at_mut(linked)
                .set_unit(atk_damaged.clone(), atk_bp);
        }
        for linked in def_linked.iter() {
            board
                .grid
                .get_at_mut(linked)
                .set_unit(def_damaged.clone(), def_bp);
        }

        let atk_tile_mut = board.grid.get_at_mut(&atk_pos);
        if atk_has_relic && atk_damaged.is_none() {
            atk_tile_mut.terrain.collectable = Some(Collectable::Relic)
        }
        atk_tile_mut.set_unit(atk_damaged.clone(), atk_bp);
        let def_tile_mut = board.grid.get_at_mut(&def_pos);
        if def_has_relic && def_damaged.is_none() {
            def_tile_mut.terrain.collectable = Some(Collectable::Relic)
        }
        def_tile_mut.set_unit(def_damaged.clone(), def_bp);
    }

    fn undo(&self, board: &mut Board) {
        let bp = board.bp.clone();
        let atk_bp = bp.as_ref().get_unit(&self.this.unit.blueprint_id);
        let def_bp = bp.as_ref().get_unit(&self.target.unit.blueprint_id);

        let atk_tile = board.grid.get_at_mut(&self.this.at);
        if atk_tile.unit.is_none() && self.this.unit.holding_collectable == Some(Collectable::Relic)
        {
            atk_tile.terrain.collectable = None;
        }
        atk_tile.set_unit(Some(self.this.unit.clone()), atk_bp);
        board.sync_linked_units(
            Some(self.this.unit.clone()),
            &board.get_target_pos(&self.this),
        );

        let def_tile = board.grid.get_at_mut(&self.target.at);
        if def_tile.unit.is_none()
            && self.target.unit.holding_collectable == Some(Collectable::Relic)
        {
            def_tile.terrain.collectable = None;
        }
        def_tile.set_unit(Some(self.target.unit.clone()), def_bp);
        board.sync_linked_units(
            Some(self.target.unit.clone()),
            &board.get_target_pos(&self.target),
        );
    }
}

impl ActAttack {
    pub fn get_range(board: &Board, target: &UnitTarget) -> i32 {
        let unit_bp = board.bp.get_unit(&target.unit.blueprint_id);
        let bonus: Bonus = board.get_unit_total_bonus(target);
        let range = if unit_bp.stats.range > 1 {
            (unit_bp.stats.apply(bonus).range).max(1)
        } else {
            1
        };
        assert!(range > 0, "range {}", range);
        range
    }

    /// Get the bonus for both attacker and defender engaged in a battle
    /// The battlefield is the terrain on the tile of the defender at `def_pos`
    pub fn get_attack_bonuses(&self, board: &Board) -> (Bonus, Bonus) {
        let bp = board.bp();
        let atk_tile = board.grid.get_at(&self.this.at);
        let atk_unit = atk_tile.get_top_unit().unwrap();
        let atk_bp = bp.get_unit(&atk_unit.blueprint_id);
        let atk_player = board.get_player(&atk_unit.owner);

        let def_tile = board.grid.get_at(&self.target.at);
        let def_unit = def_tile.get_top_unit().unwrap();
        let def_bp = bp.get_unit(&def_unit.blueprint_id);
        let def_player = board.get_player(&def_unit.owner);

        let battlefield_id = def_tile.terrain.blueprint_id.clone();
        let distance = (self.this.at - self.target.at).length();

        let (battle_atk_bonus, opp_def_bonus) = board.fold_battle_bonuses(
            ActAttack::get_battle_bonuses(bp, atk_player, atk_bp, atk_unit),
            atk_bp,
            Some(def_bp),
            Some(&battlefield_id),
            Some(distance),
        );
        let (battle_def_bonus, opp_atk_bonus) = board.fold_battle_bonuses(
            ActAttack::get_battle_bonuses(bp, def_player, def_bp, def_unit),
            def_bp,
            Some(atk_bp),
            Some(&battlefield_id),
            Some(distance),
        );
        let atk_bonus: Bonus =
            board.get_unit_total_bonus(&self.this) + battle_atk_bonus + opp_atk_bonus;
        let def_bonus: Bonus =
            board.get_unit_total_bonus(&self.target) + battle_def_bonus + opp_def_bonus;
        (atk_bonus, def_bonus)
    }

    fn get_battle_bonuses<'a>(
        bp: &'a Blueprints,
        this_player: &'a Player,
        this_bp: &'a UnitBlueprint,
        this_unit: &'a Unit,
    ) -> impl Iterator<Item = &'a BattleBonus> {
        let tech_battle_iter = this_player
            .researched_technologies
            .iter()
            .map(|id| bp.get_tech(id).battle_bonuses.iter())
            .flatten();
        let ability_battle_iter = this_bp
            .abilities
            .iter()
            .map(|ability_id| {
                let ability_bp = bp.get_ability(ability_id.ability());
                ability_bp.battle_bonuses.iter()
            })
            .flatten();
        let base_battle_iter = bp.base_bonuses.iter();
        let power_battle_bonus = this_unit
            .affected_by_powers
            .iter()
            .map(|power_id| &bp.get_power(power_id).battle_bonus);
        tech_battle_iter
            .chain(ability_battle_iter)
            .chain(base_battle_iter)
            .chain(power_battle_bonus)
    }

    pub fn battle(
        bp: &Blueprints,
        atk_unit: &Unit,
        def_unit: &Unit,
        atk_bp: &UnitBlueprint,
        def_bp: &UnitBlueprint,
        atk_bonus: Bonus,
        def_bonus: Bonus,
        distance: i32,
        _trace_steps: bool,
    ) -> BattleOutcome {
        let atk_frenzy = bp.unit_has_ability(&atk_unit.blueprint_id, "Frenzy");
        let def_frenzy = bp.unit_has_ability(&def_unit.blueprint_id, "Frenzy");

        let atk_stats = atk_bp.stats.apply(atk_bonus.clone());
        let def_stats = def_bp.stats.apply(def_bonus.clone());

        let mut atk_damaged = atk_unit.clone();
        let mut def_damaged = def_unit.clone();
        atk_damaged.veterancy += 1;
        if def_bp.header.class != UnitClass::Bld {
            def_damaged.veterancy += 1;
        }

        // first attack
        let atk_stats_volley = atk_stats.apply(Self::volley_bonus(bp, atk_unit));
        let def_stats_volley = def_stats.apply(Self::volley_bonus(bp, def_unit));
        ActAttack::damage(
            &atk_unit,
            &mut def_damaged,
            &atk_stats_volley,
            &def_stats_volley,
            atk_frenzy,
        );
        if def_damaged.health <= 0 {
            return BattleOutcome::new(Some(atk_damaged), None);
        }

        if def_bonus.forbid_counterattack
            || def_bonus.forbid_attack
            || (def_bonus.forbid_attack_after_move && def_unit.moved)
            || distance > def_stats.range
        {
            return BattleOutcome::new(Some(atk_damaged), Some(def_damaged));
        }

        // first counterattack
        let atk_stats_volley = atk_stats.apply(Self::volley_bonus(bp, &atk_damaged));
        let def_stats_volley = def_stats.apply(Self::volley_bonus(bp, &def_damaged));
        ActAttack::damage(
            &def_damaged,
            &mut atk_damaged,
            &def_stats_volley,
            &atk_stats_volley,
            def_frenzy,
        );
        if atk_damaged.health <= 0 {
            return BattleOutcome::new(None, Some(def_damaged));
        }

        if atk_bonus.can_attack_twice {
            // second attack
            let atk_stats_volley = atk_stats.apply(Self::volley_bonus(bp, &atk_damaged));
            let def_stats_volley = def_stats.apply(Self::volley_bonus(bp, &def_damaged));
            ActAttack::damage(
                &atk_damaged,
                &mut def_damaged,
                &atk_stats_volley,
                &def_stats_volley,
                atk_frenzy,
            );
            if def_damaged.health <= 0 {
                return BattleOutcome::new(None, Some(def_damaged));
            }
        }

        if def_bonus.can_attack_twice {
            // second counterattack
            let atk_stats_volley = atk_stats.apply(Self::volley_bonus(bp, &atk_damaged));
            let def_stats_volley = def_stats.apply(Self::volley_bonus(bp, &def_damaged));
            ActAttack::damage(
                &def_damaged,
                &mut atk_damaged,
                &def_stats_volley,
                &atk_stats_volley,
                def_frenzy,
            );
            if atk_damaged.health <= 0 {
                return BattleOutcome::new(None, Some(def_damaged));
            }
        }

        if bp.unit_has_ability(&atk_unit.blueprint_id, "Zeal") {
            atk_damaged.health = (atk_damaged.health + 20).min(100);
        }
        if bp.unit_has_ability(&def_unit.blueprint_id, "Zeal") {
            def_damaged.health = (def_damaged.health + 20).min(100);
        }

        BattleOutcome::new(Some(atk_damaged), Some(def_damaged))
    }

    pub fn volley_bonus(bp: &Blueprints, unit: &Unit) -> Bonus {
        let bonus = if bp.unit_has_ability(&unit.blueprint_id, "Volley") {
            if unit.health >= 50 {
                50
            } else {
                0
            }
        } else {
            0
        };
        UnitStats {
            attack: bonus,
            ..Default::default()
        }
        .into()
    }

    fn damage(
        atk: &Unit,
        def: &mut Unit,
        atk_stats: &UnitStats,
        def_stats: &UnitStats,
        frenzy: bool,
    ) {
        let a = atk_stats.attack as f64;
        let d = def_stats.defence as f64;
        let hp = if frenzy { 100 } else { atk.health };
        let dam = hp as f64 * (a / (2.0 * d));
        def.health -= dam as i32;
    }
}

impl From<ActAttack> for UnitAction {
    fn from(value: ActAttack) -> Self {
        UnitAction::Attack(value.target)
    }
}

#[allow(dead_code)]
pub struct BattleOutcome {
    pub atk: Option<Unit>,
    pub def: Option<Unit>,
    pub steps: Vec<BattleStep>,
}

impl BattleOutcome {
    fn new(atk: Option<Unit>, def: Option<Unit>) -> Self {
        Self {
            atk,
            def,
            steps: vec![],
        }
    }
}

#[allow(dead_code)]
pub struct BattleStep {
    pub atk: Option<Unit>,
    pub def: Option<Unit>,
    pub atk_stats: UnitStats,
    pub def_stats: UnitStats,
}
