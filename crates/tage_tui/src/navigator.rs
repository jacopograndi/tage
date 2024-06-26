use std::collections::HashSet;

use tage_core::{
    actions::{player_action::Pre, trade::ActTrade, train::ActTrain, travel::ActTravel},
    prelude::*,
};
use tracing::warn;

use crate::GameInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Select {
    Tile(IVec2),
    Menu(MenuChoice),
    Confirm,
}

impl Select {
    fn tile(pos: IVec2) -> Self {
        Self::Tile(pos)
    }

    fn tile_target(target: UnitTarget) -> Self {
        Self::Tile(target.at)
    }
}

#[derive(Clone, Debug)]
pub struct Path {
    pub path: Vec<Select>,
    pub action: PlayerAction,
}

impl Path {
    pub fn from_action(
        at: IVec2,
        player_action: PlayerAction,
        all_actions: &Vec<PlayerAction>,
    ) -> Path {
        let mut selects = vec![];
        match player_action.clone() {
            PlayerAction::Unit {
                target,
                destination,
                action,
                pickup,
                ..
            } => {
                selects.push(Select::tile_target(target.clone()));

                if all_actions.iter().any(|oth| match oth {
                    PlayerAction::Building {
                        target: UnitTarget { at, .. },
                        ..
                    } if *at == target.at => true,
                    _ => false,
                }) {
                    selects.push(Select::Menu(MenuChoice::TileStack(TileStackTarget::Top)));
                }

                if target.at != destination {
                    selects.push(Select::Menu(MenuChoice::Move));
                    selects.push(Select::tile(destination));
                }
                match action {
                    UnitAction::Attack(target) => {
                        selects.push(Select::Menu(MenuChoice::Attack));
                        selects.push(Select::tile_target(target));
                        selects.push(Select::Confirm);
                    }
                    UnitAction::Build(id, area) => {
                        selects.push(Select::Menu(MenuChoice::Build));
                        selects.push(Select::Menu(MenuChoice::BuildId(id)));
                        if area.iter().count() > 1 {
                            selects.push(Select::Menu(MenuChoice::BuildArea(area)));
                        }
                    }
                    UnitAction::Heal(target) => {
                        selects.push(Select::Menu(MenuChoice::Heal));
                        selects.push(Select::tile_target(target));
                    }
                    UnitAction::Convert(target) => {
                        selects.push(Select::Menu(MenuChoice::Convert));
                        selects.push(Select::tile_target(target));
                    }
                    UnitAction::Relic => match pickup {
                        Some(Collectable::Relic) => {
                            selects.push(Select::Menu(MenuChoice::RelicPickup))
                        }
                        _ => selects.push(Select::Menu(MenuChoice::RelicDropoff)),
                    },
                    UnitAction::Merge(target) => {
                        selects.push(Select::Menu(MenuChoice::Merge));
                        selects.push(Select::tile_target(target));
                    }
                    UnitAction::Repair(_) => {
                        selects.push(Select::Menu(MenuChoice::Repair));
                    }
                    UnitAction::Power(power_id, _targets) => {
                        selects.push(Select::Menu(MenuChoice::Power));
                        selects.push(Select::Menu(MenuChoice::PowerId(power_id)));
                        selects.push(Select::Confirm);
                    }
                    UnitAction::Done => selects.push(Select::Menu(MenuChoice::Done)),
                }
            }
            PlayerAction::Building { target, action } => {
                selects.push(Select::tile_target(target.clone()));

                if all_actions.iter().any(|oth| match oth {
                    PlayerAction::Unit {
                        target: UnitTarget { at, .. },
                        ..
                    } if *at == target.at => true,
                    _ => false,
                }) {
                    selects.push(Select::Menu(MenuChoice::TileStack(TileStackTarget::Bottom)));
                }

                match action {
                    BuildingAction::Train(id) => {
                        selects.push(Select::Menu(MenuChoice::Train));
                        selects.push(Select::Menu(MenuChoice::TrainId(
                            id,
                            target.unit.blueprint_id,
                        )));
                    }
                    BuildingAction::Trade(res) => {
                        selects.push(Select::Menu(MenuChoice::Trade));
                        selects.push(Select::Menu(MenuChoice::TradeResource(res)));
                    }
                    BuildingAction::AgeUp => {
                        selects.push(Select::Menu(MenuChoice::AgeUp));
                        selects.push(Select::Confirm);
                    }
                    BuildingAction::Done => selects.push(Select::Menu(MenuChoice::Done)),
                }
            }
            PlayerAction::Research(id) => {
                selects.push(Select::tile(at));
                selects.push(Select::Menu(MenuChoice::Research));
                selects.push(Select::Menu(MenuChoice::ResearchId(id)));
            }
            PlayerAction::PassTurn => {
                selects.push(Select::tile(at));
                selects.push(Select::Menu(MenuChoice::EndDay));
                selects.push(Select::Confirm);
            }
        }
        Path {
            path: selects,
            action: player_action,
        }
    }
}

pub trait Picker {
    fn selected(&self) -> Option<Select>;
}

#[derive(Debug, Clone)]
pub enum UiPicker {
    Tile(UiTilePicker),
    Menu(UiMenuPicker),
    Tech(UiTechPicker),
    Confirm,
}

impl Picker for UiPicker {
    fn selected(&self) -> Option<Select> {
        match self {
            UiPicker::Tile(picker) => picker.selected(),
            UiPicker::Menu(picker) => picker.selected(),
            UiPicker::Confirm => Some(Select::Confirm),
            UiPicker::Tech(picker) => picker.selected(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileSelectionType {
    Movement,
    Attack,
    Target,
}

#[derive(Debug, Clone)]
pub struct UiTilePicker {
    pub cursor: IVec2,
    pub unit: Option<Unit>,
    pub valid_tiles: Vec<IVec2>,
    pub selection_type: Option<TileSelectionType>,
}

impl Picker for UiTilePicker {
    fn selected(&self) -> Option<Select> {
        Some(Select::Tile(self.cursor))
    }
}

#[derive(Debug, Clone)]
pub struct UiMenuPicker {
    pub cursor: usize,
    pub choices: Vec<MenuChoice>,
}

impl Picker for UiMenuPicker {
    fn selected(&self) -> Option<Select> {
        (0..self.choices.len())
            .contains(&self.cursor)
            .then(|| Select::Menu(self.choices[self.cursor].clone()))
    }
}

impl UiMenuPicker {
    pub fn open(selects: Vec<PathNode>) -> Self {
        let uniques: HashSet<MenuChoice> = selects
            .iter()
            .map(|node| match &node.select {
                Select::Menu(choice) => choice.clone(),
                _ => unreachable!(),
            })
            .collect();
        let mut choices: Vec<MenuChoice> = uniques.into_iter().collect();
        choices.sort();
        Self { cursor: 0, choices }
    }
}

#[derive(Debug, Clone)]
pub struct UiTechPicker {
    pub cursor: usize,
    pub level: usize,
    pub layout: Vec<Vec<TechId>>,
    pub choices: Vec<TechId>,
}

impl Picker for UiTechPicker {
    fn selected(&self) -> Option<Select> {
        if !(0..self.layout.len()).contains(&self.level) {
            return None;
        }
        if !(0..self.layout[self.level].len()).contains(&self.cursor) {
            return None;
        }
        let selected = self.layout[self.level][self.cursor].clone();
        if !self.choices.contains(&selected) {
            return None;
        }
        Some(Select::Menu(MenuChoice::ResearchId(selected.clone())))
    }
}

impl UiTechPicker {
    pub fn open(blueprints: &Blueprints, selects: Vec<PathNode>) -> Self {
        let uniques: HashSet<TechId> = selects
            .into_iter()
            .map(|node| match node.select {
                Select::Menu(MenuChoice::ResearchId(tech_id)) => tech_id,
                _ => unreachable!(),
            })
            .collect();
        let mut sorted_techs: Vec<(TechId, TechBlueprint)> =
            blueprints.techs.clone().into_iter().collect();
        sorted_techs.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));
        let max_level = sorted_techs
            .iter()
            .max_by(|a, b| a.1.level.cmp(&b.1.level))
            .expect("there is more than 1 tech");
        let mut layout: Vec<Vec<TechId>> = vec![vec![]; max_level.1.level as usize + 1];
        for (id, tech) in sorted_techs {
            layout[tech.level as usize].push(id);
        }
        Self {
            cursor: 0,
            level: 0,
            layout,
            choices: uniques.into_iter().collect(),
        }
    }
}

pub struct Layer {
    pub picker: UiPicker,
    pub preview_board: Board,
    pub selected_action: Option<PlayerAction>,
}

pub struct Navigator {
    pub stack: Vec<Layer>,
    pub paths: Vec<Path>,
}

pub struct PathNode {
    pub select: Select,
    pub action: PlayerAction,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NavigatorError {
    InvalidPush,
    Empty,
    Bonked(PlayerAction),
}

impl Navigator {
    pub fn open(board: &Board, picker: UiTilePicker, actions: Vec<PlayerAction>) -> Navigator {
        let paths: Vec<Path> = actions
            .iter()
            .map(|a| Path::from_action(picker.cursor, a.clone(), &actions))
            .collect();
        Navigator {
            stack: vec![Layer {
                picker: UiPicker::Tile(picker),
                preview_board: board.clone(),
                selected_action: None,
            }],
            paths,
        }
    }

    pub fn push(&mut self, picker: UiPicker, clear_board: &Board) -> Option<NavigatorError> {
        let Some(node) = self
            .star()
            .into_iter()
            .find(|node| Some(&node.select) == picker.selected().as_ref())
        else {
            return Some(NavigatorError::InvalidPush);
        };

        let mut preview_board = if let Some(top) = self.stack.iter().last() {
            top.preview_board.clone()
        } else {
            return Some(NavigatorError::Empty);
        };

        if let UiPicker::Tile(UiTilePicker {
            selection_type: Some(TileSelectionType::Movement),
            ..
        }) = picker
        {
            if let PlayerAction::Unit {
                target,
                destination,
                path,
                ..
            } = &node.action
            {
                let act = ActTravel {
                    this: target.clone(),
                    destination: destination.clone(),
                    path: path.clone(),
                };

                if let Some(reachable) = act.has_bonked(clear_board) {
                    return Some(NavigatorError::Bonked(PlayerAction::Unit {
                        target: target.clone(),
                        destination: reachable.destination,
                        pickup: None,
                        action: UnitAction::Done,
                        path: path.clone(),
                    }));
                }
                act.apply(&mut preview_board);

                preview_board.refresh_fog();
            }
        }

        self.stack.push(Layer {
            picker,
            preview_board,
            selected_action: Some(node.action),
        });
        None
    }

    pub fn star(&self) -> Vec<PathNode> {
        let picked = self.picked();
        let star: Vec<PathNode> = self
            .paths
            .iter()
            .filter_map(|p| {
                if picked.iter().eq(p.path.iter().take(picked.len())) {
                    if let Some(select) = p.path.iter().skip(picked.len()).next() {
                        Some(PathNode {
                            select: select.clone(),
                            action: p.action.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        if let Some(first) = star.first() {
            assert!(star.iter().all(|node| {
                match (&first.select, &node.select) {
                    (Select::Confirm, Select::Confirm) => true,
                    (Select::Menu(_), Select::Menu(_)) => true,
                    (Select::Tile { .. }, Select::Tile { .. }) => true,
                    _ => false,
                }
            }));
        }
        star
    }

    pub fn picked(&self) -> Vec<Select> {
        self.stack
            .iter()
            .filter_map(|layer| layer.picker.selected().clone())
            .collect()
    }
}

pub struct GameState {
    pub board: Board,
    pub blueprints: Blueprints,
    pub navigator: Option<Navigator>,
    pub current_picker: UiPicker,
    pub turn_timeline: Vec<PlayerAction>,
}

impl GameState {
    pub fn apply_input(&mut self, input: GameInput) -> Option<PlayerAction> {
        match &mut self.current_picker {
            UiPicker::Tile(picker) => {
                picker.cursor = picker.cursor + input.acc;
                picker.cursor = picker
                    .cursor
                    .clamp(IVec2::ZERO, self.board.grid.size - IVec2::ONE);
                if picker.selection_type != Some(TileSelectionType::Movement) {
                    picker.unit = self
                        .board
                        .grid
                        .get_at(&picker.cursor)
                        .get_top_unit()
                        .cloned();
                }
            }
            UiPicker::Menu(menu) => {
                if menu.choices.is_empty() {
                    return None;
                }
                menu.cursor = (menu.cursor as i32 + input.acc.y)
                    .clamp(0, menu.choices.len() as i32 - 1) as usize;
            }
            UiPicker::Tech(tech_menu) => {
                tech_menu.level = (tech_menu.level as i32 + input.acc.x)
                    .clamp(0, tech_menu.layout.len() as i32 - 1)
                    as usize;
                tech_menu.cursor = (tech_menu.cursor as i32 + input.acc.y)
                    .clamp(0, tech_menu.layout[tech_menu.level].len() as i32 - 1)
                    as usize;
            }
            UiPicker::Confirm => {}
        }

        if input.select {
            if let Some(nav) = &mut self.navigator {
                if let Some(err) = nav.push(self.current_picker.clone(), &self.board) {
                    warn!("error while selecting {:?}", err);
                    match err {
                        NavigatorError::Bonked(action) => self.apply_bonk(action),
                        _ => {}
                    }
                } else {
                    return self.open_picker();
                }
            } else {
                return self.open_navigator();
            }
        }

        if input.back {
            if self.board.can_undo() {
                if let Some(nav) = &mut self.navigator {
                    if nav.stack.len() > 1 {
                        let layer = nav.stack.pop().unwrap();
                        self.current_picker = layer.picker;
                    } else {
                        self.current_picker = UiPicker::Tile(UiTilePicker {
                            cursor: self.top_tile_picker().cursor,
                            unit: None,
                            valid_tiles: vec![],
                            selection_type: None,
                        });
                        self.navigator = None;
                    }
                } else {
                    if let Some(last_action) = self.turn_timeline.pop() {
                        last_action.undo(&mut self.board)
                    }
                }
            } else {
                if let Some(nav) = &mut self.navigator {
                    if nav.stack.len() == 1 {
                        self.current_picker = UiPicker::Tile(UiTilePicker {
                            cursor: self.top_tile_picker().cursor,
                            unit: None,
                            valid_tiles: vec![],
                            selection_type: None,
                        });
                        self.navigator = None;
                    }
                }
            }
        }

        if input.next {
            if self.navigator.is_none() {
                // get closest non-done unit
                let cursor = self.top_tile_picker().cursor;
                let mut temp_board = self.board.clone();
                if let Some((_, pos)) = self
                    .board
                    .get_player_units_pos(&self.board.current_player_turn)
                    .filter(|(_, pos)| {
                        !PlayerAction::generate(&Pre::Tile(*pos), &mut temp_board).is_empty()
                    })
                    .min_by(|a, b| (a.1 - cursor).length().cmp(&(b.1 - cursor).length()))
                {
                    if let UiPicker::Tile(picker) = &mut self.current_picker {
                        picker.cursor = pos
                    }
                }
            }
        }

        if input.short_move {
            return self.short(MenuChoice::Move);
        }

        if input.short_attack {
            return self.short(MenuChoice::Attack);
        }

        if input.short_done {
            return self.short(MenuChoice::Done);
        }

        if input.short_build {
            return self.short(MenuChoice::Build);
        }

        if input.short_tech {
            if self.navigator.is_none() {
                if let UiPicker::Tile(picker) = &mut self.current_picker {
                    let mut stripped = self.board.strip_fog(&self.board.current_player_turn);
                    let turn_actions = PlayerAction::generate(&Pre::Global, &mut stripped);
                    picker.unit = None;
                    let nav = Navigator::open(&stripped, picker.clone(), turn_actions);
                    self.navigator = Some(nav);
                    self.open_picker();
                }
            }

            return self.short(MenuChoice::Research);
        }

        if input.short_pass {
            if self.navigator.is_none() {
                if let UiPicker::Tile(picker) = &mut self.current_picker {
                    let mut stripped = self.board.strip_fog(&self.board.current_player_turn);
                    let turn_actions = PlayerAction::generate(&Pre::Global, &mut stripped);
                    picker.unit = None;
                    let nav = Navigator::open(&stripped, picker.clone(), turn_actions);
                    self.navigator = Some(nav);
                    self.open_picker();
                }
            }

            return self.short(MenuChoice::EndDay);
        }

        None
    }

    pub fn short(&mut self, choice: MenuChoice) -> Option<PlayerAction> {
        if self.navigator.is_none() {
            self.open_navigator();
        }

        if let Some(nav) = &mut self.navigator {
            if let UiPicker::Tile(_) = &mut self.current_picker {
                if let Some(err) = nav.push(self.current_picker.clone(), &self.board) {
                    warn!("error while selecting {:?}", err);
                    match err {
                        NavigatorError::Bonked(action) => self.apply_bonk(action),
                        _ => {}
                    }
                } else {
                    self.open_picker();
                }
            }
        }

        if let Some(nav) = &mut self.navigator {
            if let UiPicker::Menu(menu) = &mut self.current_picker {
                if let Some((i, _)) = menu.choices.iter().enumerate().find(|(_, c)| c == &&choice) {
                    menu.cursor = i;
                    if let Some(err) = nav.push(self.current_picker.clone(), &self.board) {
                        warn!("error while selecting {:?}", err);
                        match err {
                            NavigatorError::Bonked(action) => self.apply_bonk(action),
                            _ => {}
                        }
                    } else {
                        return self.open_picker();
                    }
                }
            }
        }

        None
    }

    pub fn open_navigator(&mut self) -> Option<PlayerAction> {
        if let UiPicker::Tile(picker) = &mut self.current_picker {
            let mut stripped = self.board.strip_fog(&self.board.current_player_turn);
            let tile_actions = PlayerAction::generate(&Pre::Tile(picker.cursor), &mut stripped);
            if !tile_actions.is_empty() {
                let nav = Navigator::open(&stripped, picker.clone(), tile_actions);
                self.navigator = Some(nav);
                return self.open_picker();
            } else {
                let turn_actions = PlayerAction::generate(&Pre::Global, &mut stripped);
                picker.unit = None;
                let nav = Navigator::open(&stripped, picker.clone(), turn_actions);
                self.navigator = Some(nav);
                return self.open_picker();
            }
        }
        None
    }

    pub fn apply_bonk(&mut self, action: PlayerAction) {
        self.current_picker = UiPicker::Tile(UiTilePicker {
            cursor: self.top_tile_picker().cursor,
            unit: None,
            valid_tiles: vec![],
            selection_type: None,
        });
        self.navigator = None;

        action.apply(&mut self.board);
        self.board.refresh_fog();

        self.turn_timeline.push(action.clone());
    }

    pub fn open_picker(&mut self) -> Option<PlayerAction> {
        let Some(navigator) = &mut self.navigator else {
            return None;
        };

        let star = navigator.star();
        if star.is_empty()
            || (star.len() == 1
                && star
                    .first()
                    .is_some_and(|node| node.select == Select::Menu(MenuChoice::Done)))
        {
            // leaf reached: apply action and delete nav
            let top = navigator
                .stack
                .pop()
                .expect("opened nav always has a layer");
            self.current_picker = UiPicker::Tile(UiTilePicker {
                cursor: self.top_tile_picker().cursor,
                unit: None,
                valid_tiles: vec![],
                selection_type: None,
            });
            self.navigator = None;
            if let Some(action) = &top.selected_action {
                let action = self.board.fog_bonk(action.clone());
                action.apply(&mut self.board);
                self.board.refresh_fog();

                match action {
                    PlayerAction::PassTurn => self.turn_timeline.clear(),
                    _ => self.turn_timeline.push(action.clone()),
                }
                return Some(action.clone());
            } else {
                return None;
            }
        }

        let next = &star[0];
        self.current_picker = match next.select {
            Select::Tile { .. } => {
                let picked = navigator.picked();
                let selection_type = picked.last().map(|select| match select {
                    Select::Menu(MenuChoice::Attack) => TileSelectionType::Attack,
                    Select::Menu(MenuChoice::Move) => TileSelectionType::Movement,
                    _ => TileSelectionType::Target,
                });
                let unique_tiles: HashSet<IVec2> = star
                    .iter()
                    .map(|node| match node.select {
                        Select::Tile(pos) => pos,
                        _ => unreachable!(),
                    })
                    .collect();
                UiPicker::Tile(UiTilePicker {
                    cursor: self.top_tile_picker().cursor,
                    unit: None,
                    valid_tiles: unique_tiles.into_iter().collect(),
                    selection_type,
                })
            }
            Select::Menu(MenuChoice::ResearchId(_)) => {
                UiPicker::Tech(UiTechPicker::open(&self.blueprints, star))
            }
            Select::Menu(_) => UiPicker::Menu(UiMenuPicker::open(star)),
            Select::Confirm => UiPicker::Confirm,
        };

        None
    }

    pub fn top_tile_picker(&self) -> UiTilePicker {
        if let UiPicker::Tile(picker) = &self.current_picker {
            picker.clone()
        } else {
            let nav = self
                .navigator
                .as_ref()
                .expect("the base picker is always a tile picker");
            if let Some(picker) = nav
                .stack
                .iter()
                .rev()
                .find_map(|layer| match &layer.picker {
                    UiPicker::Tile(picker) => Some(picker),
                    _ => None,
                })
            {
                picker.clone()
            } else {
                unreachable!("the base picker is always a tile picker")
            }
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TileStackTarget {
    Bottom,
    Top,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum MenuChoice {
    TileStack(TileStackTarget),
    Move,
    Attack,
    Build,
    BuildId(UnitId),
    BuildArea(BuildArea),
    Heal,
    Convert,
    RelicPickup,
    RelicDropoff,
    Merge,
    Power,
    PowerId(PowerId),
    Repair,
    Train,
    TrainId(UnitId, UnitId),
    TradeResource(Resource),
    Trade,
    AgeUp,
    Done,
    Research,
    ResearchId(TechId),
    EndDay,
}

impl MenuChoice {
    pub fn priority(&self) -> u32 {
        match self {
            Self::Move => 0,
            Self::Attack => 1,
            Self::Build => 2,
            Self::BuildId(id) => 3 + id.0,
            Self::RelicPickup => 4,
            Self::RelicDropoff => 5,
            Self::Heal => 1000,
            Self::Convert => 1001,
            Self::Merge => 1002,
            Self::Power => 2000,
            Self::PowerId(id) => 2001 + id.0,
            Self::Repair => 3000,
            Self::Train => 3001,
            Self::TrainId(id, _build_id) => 3002 + id.0,
            Self::TradeResource(id) => {
                4000 + match id {
                    Resource::Food => 0,
                    Resource::Gold => 1,
                }
            }
            Self::Trade => 4002,
            Self::AgeUp => 4003,
            Self::Done => 4004,
            Self::Research => 4005,
            Self::ResearchId(id) => 4006 + id.0,
            Self::EndDay => 5000,
            Self::TileStack(target) => match target {
                TileStackTarget::Top => 10000,
                TileStackTarget::Bottom => 10001,
            },
            Self::BuildArea(_) => 0,
        }
    }

    pub fn view(&self, board: &Board) -> String {
        match self {
            MenuChoice::TileStack(target) => match target {
                TileStackTarget::Top => "Unit".to_string(),
                TileStackTarget::Bottom => "Building".to_string(),
            },
            MenuChoice::Move => "Move".to_string(),
            MenuChoice::Attack => "Attack".to_string(),
            MenuChoice::Build => "Build".to_string(),
            MenuChoice::BuildId(id) => {
                let build_bp = board.bp.get_unit(id);
                let bonus = board.get_player_bonus(&board.current_player_turn, Some(&id));
                let cost = build_bp.resources.cost.apply_cost(bonus);
                format!(
                    "Build {} (cost {}f {}g)",
                    build_bp.header.name, cost.food, cost.gold
                )
            }
            MenuChoice::BuildArea(area) => format!("{:?}", area),
            MenuChoice::Heal => "Heal".to_string(),
            MenuChoice::Convert => "Convert".to_string(),
            MenuChoice::RelicPickup => "Capture Relic".to_string(),
            MenuChoice::RelicDropoff => "Donate Relic".to_string(),
            MenuChoice::Merge => "Merge".to_string(),
            MenuChoice::Power => "Powers".to_string(),
            MenuChoice::PowerId(id) => board.bp.get_power(id).name.clone(),
            MenuChoice::Repair => "Repair".to_string(),
            MenuChoice::Train => "Train".to_string(),
            MenuChoice::TrainId(id, build_id) => {
                let unit_bp = board.bp.get_unit(id);
                let bonus = ActTrain::get_bonus(board, &board.current_player_turn, &id, &build_id);
                let cost = unit_bp.resources.cost.apply_cost(bonus);
                format!(
                    "Train {} (base cost {}f {}g)",
                    unit_bp.header.name, cost.food, cost.gold
                )
            }
            MenuChoice::TradeResource(resource) => {
                let rate = ActTrade::get_rate(&board.current_player_turn, board);
                match resource {
                    Resource::Food => format!("Trade {} Food for 100 Gold", rate),
                    Resource::Gold => format!("Trade {} Gold for 100 Food", rate),
                }
            }
            MenuChoice::Trade => "Trade".to_string(),
            MenuChoice::AgeUp => "Age Up".to_string(),
            MenuChoice::Done => "Done".to_string(),
            MenuChoice::Research => "Research".to_string(),
            MenuChoice::ResearchId(id) => {
                let tech = board.bp.get_tech(id);
                format!("Tech {}", tech.name)
            }
            MenuChoice::EndDay => "End Day".to_string(),
        }
    }
}

impl PartialOrd for MenuChoice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl Ord for MenuChoice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}
