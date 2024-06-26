use crate::prelude::*;

use self::{
    attack::ActAttack,
    build::ActBuild,
    convert::ActConvert,
    done::{ActDone, ActNone},
    end_turn::ActEndTurn,
    heal::ActHeal,
    merge::ActMerge,
    pickup::ActPickup,
    power::ActPower,
    relic::ActRelic,
    repair::ActRepair,
    research::{ActAgeUp, ActResearch},
    trade::ActTrade,
    train::ActTrain,
    travel::ActTravel,
};

/// Root of the player action from which other actions are generated
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pre {
    Target(UnitPos),
    Tile(IVec2),
    Research,
    Global,
}

impl Act for PlayerAction {
    type Precondition = Pre;

    fn generate(pre: &Self::Precondition, board: &mut Board) -> Vec<Self> {
        match pre {
            Pre::Target(upos) => {
                if let Some(target) = board.get_pos_target(upos) {
                    match upos.loc {
                        UnitLocation::Top => PlayerAction::gen_unit(&target, board),
                        UnitLocation::Bot => PlayerAction::gen_building(&target, board),
                    }
                } else {
                    vec![]
                }
            }
            Pre::Tile(pos) => {
                let top = Self::generate(&Pre::Target(UnitPos::top(*pos)), board);
                let bot = Self::generate(&Pre::Target(UnitPos::bot(*pos)), board);
                top.into_iter().chain(bot.into_iter()).collect()
            }
            Pre::Research => PlayerAction::gen_research(board),
            Pre::Global => PlayerAction::gen_global(board),
        }
    }

    fn is_valid(&self, board: &Board) -> bool {
        //todo: wasteful
        let mut local_board = board.clone();
        match self {
            PlayerAction::Unit { target, .. } => {
                Self::generate(&Pre::Target(board.get_target_pos(target)), &mut local_board)
                    .contains(&self)
            }
            PlayerAction::Building { target, .. } => {
                Self::generate(&Pre::Target(board.get_target_pos(target)), &mut local_board)
                    .contains(&self)
            }
            PlayerAction::Research(tech_id) => ActResearch {
                tech_id: tech_id.clone(),
            }
            .is_valid(board),
            PlayerAction::PassTurn => ActEndTurn.is_valid(board),
        }
    }

    fn apply(&self, board: &mut Board) {
        self.wrap_check_invariants(board, |board| match self.clone() {
            PlayerAction::Unit {
                target: this,
                destination,
                pickup,
                action,
                path,
            } => {
                ActTravel {
                    this: this.clone(),
                    destination: destination.clone(),
                    path: path.clone(),
                }
                .apply(board);
                let moved = UnitTarget::new(
                    board.get_unit(&UnitPos::top(destination)).unwrap().clone(),
                    destination,
                );
                match action {
                    UnitAction::Attack(target) => ActAttack {
                        this: moved.clone(),
                        target,
                    }
                    .apply(board),
                    UnitAction::Build(build_id, area) => ActBuild {
                        this: moved.clone(),
                        build_id,
                        area,
                    }
                    .apply(board),
                    UnitAction::Heal(target) => ActHeal {
                        this: moved.clone(),
                        target,
                    }
                    .apply(board),
                    UnitAction::Convert(target) => ActConvert {
                        this: moved.clone(),
                        target,
                    }
                    .apply(board),
                    UnitAction::Relic => ActRelic {
                        this: moved.clone(),
                    }
                    .apply(board),
                    UnitAction::Merge(target) => ActMerge {
                        this: moved.clone(),
                        target,
                    }
                    .apply(board),
                    UnitAction::Repair(target) => ActRepair {
                        this: moved.clone(),
                        target,
                    }
                    .apply(board),
                    UnitAction::Power(power_id, targets) => ActPower {
                        this: moved.clone(),
                        power_id,
                        targets,
                    }
                    .apply(board),
                    UnitAction::Done => {}
                };
                ActPickup {
                    this: moved.clone(),
                    pickup,
                }
                .apply(board);
                ActDone {
                    this: moved.clone(),
                }
                .apply(board);
            }
            PlayerAction::Building {
                target: this,
                action,
            } => {
                match action {
                    BuildingAction::Train(train_id) => ActTrain {
                        this: this.clone(),
                        train_id,
                    }
                    .apply(board),
                    BuildingAction::Trade(resource) => ActTrade {
                        this: this.clone(),
                        resource,
                    }
                    .apply(board),
                    BuildingAction::AgeUp => ActAgeUp { this: this.clone() }.apply(board),
                    BuildingAction::Done => {}
                }
                ActDone { this: this.clone() }.apply(board);
            }
            PlayerAction::Research(tech_id) => ActResearch { tech_id }.apply(board),
            PlayerAction::PassTurn => ActEndTurn.apply(board),
        })
    }

    fn undo(&self, board: &mut Board) {
        self.wrap_check_invariants(board, |board| match self.clone() {
            PlayerAction::Unit {
                target: this,
                destination,
                pickup,
                action,
                path,
            } => {
                let moved = UnitTarget::new(this.unit.clone(), destination);
                ActDone {
                    this: moved.clone(),
                }
                .undo(board);
                ActPickup {
                    this: moved.clone(),
                    pickup,
                }
                .undo(board);
                match action {
                    UnitAction::Attack(target) => ActAttack {
                        this: moved.clone(),
                        target,
                    }
                    .undo(board),
                    UnitAction::Build(build_id, area) => ActBuild {
                        this: moved.clone(),
                        build_id,
                        area,
                    }
                    .undo(board),
                    UnitAction::Heal(target) => ActHeal {
                        this: moved.clone(),
                        target,
                    }
                    .undo(board),
                    UnitAction::Convert(target) => ActConvert {
                        this: moved.clone(),
                        target,
                    }
                    .undo(board),
                    UnitAction::Relic => ActRelic {
                        this: moved.clone(),
                    }
                    .undo(board),
                    UnitAction::Merge(target) => ActMerge {
                        this: moved.clone(),
                        target,
                    }
                    .undo(board),
                    UnitAction::Repair(target) => ActRepair {
                        this: moved.clone(),
                        target,
                    }
                    .undo(board),
                    UnitAction::Power(power_id, targets) => ActPower {
                        this: moved.clone(),
                        power_id,
                        targets,
                    }
                    .undo(board),
                    UnitAction::Done => ActDone {
                        this: moved.clone(),
                    }
                    .undo(board),
                };
                ActTravel {
                    this: this.clone(),
                    destination: destination.clone(),
                    path: path.clone(),
                }
                .undo(board);
            }
            PlayerAction::Building {
                target: this,
                action,
            } => {
                ActDone { this: this.clone() }.undo(board);
                match action {
                    BuildingAction::Train(train_id) => ActTrain {
                        this: this.clone(),
                        train_id,
                    }
                    .undo(board),
                    BuildingAction::Trade(resource) => ActTrade {
                        this: this.clone(),
                        resource,
                    }
                    .undo(board),
                    BuildingAction::AgeUp => ActAgeUp { this: this.clone() }.undo(board),
                    BuildingAction::Done => {}
                }
            }
            PlayerAction::Research(tech_id) => ActResearch { tech_id }.undo(board),
            PlayerAction::PassTurn => ActEndTurn.undo(board),
        })
    }
}

impl PlayerAction {
    fn wrap_check_invariants<F: Fn(&mut Board) -> ()>(&self, board: &mut Board, f: F) {
        #[cfg(not(debug_assertions))]
        {
            // in release call the wrapped function without checking
            f(board);
        }

        #[cfg(debug_assertions)]
        if board.fog_base != FogTile::Visible {
            // fog-stripped board can lead to machines to generate invalid moves
            // so don't check the invariants then
            f(board);
        } else {
            tracing::trace!(target: "actions", "{}", self.view(board.bp()));
            let before = board.clone();

            // call the wrapped function
            f(board);

            // check some invariants to detect bugs
            // dump the board state if an invariant is broken
            assert!(
                board
                    .get_current_player()
                    .resources
                    .contains(&Resources::new(0, 0)),
                "{:?}\nbefore:{}\nboard:{}",
                self,
                before.view(),
                board.view(),
            );
            for (_, tile) in board.grid.iter() {
                if let Some(unit) = &tile.unit {
                    let unit_bp = board.bp.get_unit(&unit.blueprint_id);
                    assert_ne!(
                        unit_bp.header.class,
                        UnitClass::Bld,
                        "{:?}\nbefore:{}\nboard:{}",
                        self,
                        before.view(),
                        board.view(),
                    );
                }
                if let Some(building) = &tile.building {
                    let unit_bp = board.bp.get_unit(&building.blueprint_id);
                    assert_eq!(
                        unit_bp.header.class,
                        UnitClass::Bld,
                        "{:?}\nbefore:{}\nboard:{}",
                        self,
                        before.view(),
                        board.view(),
                    );
                    let size = unit_bp.unit_size.size;
                    let linked = if size == 1 { 0 } else { size * size };
                    assert_eq!(building.linked_units.len() as i32, linked);
                    for pos in &building.linked_units {
                        let linked_tile = board.grid.get_at(&pos);
                        assert_eq!(
                            Some(building),
                            linked_tile.building.as_ref(),
                            "{:?}\nbefore:{}\nboard:{}",
                            self,
                            before.view(),
                            board.view(),
                        );
                    }
                }
            }
        }
    }

    fn gen_unit(pre: &UnitTarget, board: &mut Board) -> Vec<PlayerAction> {
        if board.get_unit_target(pre).is_some_and(|u| u.done) {
            return vec![];
        }

        ActTravel::generate(pre, board)
            .iter()
            .map(|act_travel| {
                act_travel.apply(board);
                let to = &UnitTarget::new(
                    board
                        .get_unit(&UnitPos::top(act_travel.destination))
                        .unwrap()
                        .clone(),
                    act_travel.destination,
                );
                let p = &act_travel.path;
                let mut acts = vec![];
                acts.extend(Self::gen_unit_moved::<ActAttack>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActBuild>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActRepair>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActMerge>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActHeal>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActConvert>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActPower>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActRelic>(pre, to, board, p));
                acts.extend(Self::gen_unit_moved::<ActNone>(pre, to, board, p));
                act_travel.undo(board);
                acts
            })
            .flatten()
            .collect()
    }

    fn gen_unit_moved<A>(
        pre: &UnitTarget,
        moved: &UnitTarget,
        board: &mut Board,
        path: &Vec<IVec2>,
    ) -> impl Iterator<Item = PlayerAction>
    where
        A: Act<Precondition = UnitTarget> + Clone,
        UnitAction: From<A>,
    {
        let actions: Vec<PlayerAction> = A::generate(moved, board)
            .into_iter()
            .map(|act| {
                act.apply(board);
                let results: Vec<PlayerAction> = ActPickup::generate(moved, board)
                    .iter()
                    .map(|act_pickup| {
                        act_pickup.apply(board);
                        let results: Vec<PlayerAction> = ActDone::generate(moved, board)
                            .iter()
                            .map(|act_done| {
                                act_done.apply(board);
                                let result = PlayerAction::Unit {
                                    target: pre.clone(),
                                    destination: moved.at.clone(),
                                    pickup: act_pickup.pickup.clone(),
                                    action: act.clone().into(),
                                    path: path.clone(),
                                };
                                act_done.undo(board);
                                result
                            })
                            .collect();
                        act_pickup.undo(board);
                        results
                    })
                    .flatten()
                    .collect();
                act.undo(board);
                results
            })
            .flatten()
            .collect();
        actions.into_iter()
    }

    fn gen_building(target: &UnitTarget, board: &mut Board) -> Vec<PlayerAction> {
        let trains: Vec<PlayerAction> = ActTrain::generate(target, board)
            .into_iter()
            .map(|act| act.into())
            .collect();
        let trades: Vec<PlayerAction> = ActTrade::generate(target, board)
            .into_iter()
            .map(|act| act.into())
            .collect();
        let ageups: Vec<PlayerAction> = ActAgeUp::generate(target, board)
            .into_iter()
            .map(|act| act.into())
            .collect();
        let dones: Vec<PlayerAction> = ActDone::generate(target, board)
            .into_iter()
            .map(|act| PlayerAction::Building {
                target: target.clone(),
                action: act.into(),
            })
            .collect();

        if trades.is_empty() && trains.is_empty() && ageups.is_empty() {
            return vec![];
        }

        trades
            .into_iter()
            .chain(trains.into_iter())
            .chain(ageups.into_iter())
            .chain(dones.into_iter())
            .collect()
    }

    fn gen_research(board: &mut Board) -> Vec<PlayerAction> {
        ActResearch::generate(&(), board)
            .into_iter()
            .map(|act| PlayerAction::Research(act.tech_id))
            .collect()
    }

    fn gen_global(board: &mut Board) -> Vec<PlayerAction> {
        let mut res = Self::gen_research(board);
        res.push(PlayerAction::PassTurn);
        res
    }
}

impl PlayerAction {
    pub fn view(&self, bp: &Blueprints) -> String {
        match self {
            PlayerAction::Unit {
                target,
                destination,
                action,
                pickup,
                ..
            } => format!(
                "{} moves to {} and {}{}",
                target.view(bp),
                destination,
                match action {
                    UnitAction::Attack(t) => format!("attacks {}", t.view(bp)),
                    UnitAction::Build(build_id, _) =>
                        format!("builds a {}", bp.get_unit(build_id).header.name),
                    UnitAction::Heal(t) => format!("heals {}", t.view(bp)),
                    UnitAction::Convert(t) => format!("converts {}", t.view(bp)),
                    UnitAction::Relic => format!("uses relic action"),
                    UnitAction::Merge(t) => format!("merges with {}", t.view(bp)),
                    UnitAction::Repair(t) => format!("repairs {}", t.view(bp)),
                    UnitAction::Power(power_id, targets) => format!(
                        "uses power {} that affects {}",
                        bp.get_power(power_id).name,
                        targets.iter().fold(String::new(), |ss, t| format!(
                            "{}{} ",
                            ss,
                            t.view(bp)
                        ))
                    ),
                    UnitAction::Done => format!("does nothing"),
                },
                match pickup {
                    Some(coll) => format!(" picking up {:?}", coll),
                    _ => format!(""),
                }
            ),
            PlayerAction::Building { target, action } => format!(
                "{} {}",
                target.view(bp),
                match action {
                    BuildingAction::Train(train_id) =>
                        format!("trains a {}", bp.get_unit(train_id).header.name),
                    BuildingAction::Trade(res) => format!("trades away {:?}", res),
                    BuildingAction::AgeUp => format!("ages up"),
                    BuildingAction::Done => format!("does nothing"),
                }
            ),
            PlayerAction::Research(tech_id) => format!("Research {}", bp.get_tech(tech_id).name),
            PlayerAction::PassTurn => format!("Pass Turn"),
        }
    }
}

impl UnitTarget {
    pub fn view(&self, bp: &Blueprints) -> String {
        let unit_bp = bp.get_unit(&self.unit.blueprint_id);
        let class = match unit_bp.header.class {
            UnitClass::Bld => "Building",
            _ => "Unit",
        };
        format!(
            "{}({} at {} of {})",
            class,
            unit_bp.header.name,
            self.at,
            self.unit.owner.view()
        )
    }
}
