use std::collections::HashMap;

use crossterm::event::KeyCode;
use ratatui::{layout::Constraint, prelude::*, widgets::*};
use tage_core::{actions::end_turn::calculate_production, prelude::*};

use crate::*;

#[derive(Debug, Clone, Default)]
pub struct PauseMenuState {
    choices: Vec<String>,
    cursor: i32,
    settings: Option<MenuSettings>,
    load: Option<MenuLoad>,
    save: Option<String>,
    review: Option<MenuEmpireReview>,
    library: Option<MenuLibrary>,
    log: Option<i32>,
}

impl PauseMenuState {
    pub fn new() -> Self {
        PauseMenuState {
            choices: [
                "Return to Game",
                "Empire Review",
                "Library",
                "Turn Actions",
                "Save Game",
                "Load Game",
                "Settings",
                "Exit to Main Menu",
                "Quit Game",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            ..Default::default()
        }
    }

    pub fn input(
        mut self,
        input: MenuInput,
        is_running: &mut bool,
        ui_state: &mut InterfaceState,
        game_state: &mut Option<GameState>,
        bp: &Blueprints,
        net: &mut Net,
    ) -> Option<Self> {
        if let Some(settings) = self.settings {
            self.settings = settings.input(input.clone(), ui_state);
            return Some(self);
        }

        if let Some(load) = self.load {
            self.load = load.input(input.clone(), bp, game_state);
            if let Some(ref load) = self.load {
                if load.loaded {
                    return None;
                }
            }
            return Some(self);
        }

        if let Some(review) = self.review {
            self.review = review.input(input, game_state);
            return Some(self);
        }

        if let Some(library) = self.library {
            self.library = library.input(input, bp);
            return Some(self);
        }

        if let Some(offset) = self.log {
            let max =
                (game_state.as_ref().map_or(0, |gs| gs.turn_timeline.len()) as i32 - 1).max(0);
            self.log = Some((offset + input.acc.y).clamp(0, max));
            if input.select || input.back || input.quit {
                self.log = None;
            }
            return Some(self);
        }

        if let Some(mut save) = self.save {
            if input.back || input.quit {
                self.save = None;
                return Some(self);
            }

            match input.keycode {
                Some(KeyCode::Enter) => {
                    if let Some(game_state) = game_state.as_ref() {
                        let mut path = get_data_dir_sub("saves").unwrap_or_default();
                        path.push(format!("{}.ron", save));
                        tracing::trace!("saving to {:?}", path);
                        let _ = game_state.board.save(path.to_str().unwrap());
                        //todo: report failure
                        return None;
                    }
                }
                Some(KeyCode::Backspace) => {
                    save.pop();
                }
                Some(KeyCode::Char(c)) => {
                    if ALLOWED_PATH_CHARS.contains(&c) && ui_state.member_profile.name.len() < 32 {
                        save.push(c)
                    }
                }
                _ => {}
            }
            self.save = Some(save);
            return Some(self);
        }

        if input.back || input.quit {
            return None;
        }

        self.cursor = (self.cursor + input.acc.y).clamp(0, self.choices.len() as i32 - 1);
        if input.select {
            match self.cursor {
                0 => None,
                1 => {
                    self.review = Some(MenuEmpireReview::new());
                    Some(self)
                }
                2 => {
                    self.library = Some(MenuLibrary::new());
                    Some(self)
                }
                3 => {
                    self.log = Some(0);
                    Some(self)
                }
                4 => {
                    self.save = Some(String::new());
                    Some(self)
                }
                5 => {
                    self.load = Some(MenuLoad::new());
                    Some(self)
                }
                6 => {
                    self.settings = Some(MenuSettings::new(ui_state));
                    Some(self)
                }
                7 => {
                    ui_state.reset();
                    ui_state.main_menu = Some(MenuState::Home(MenuHome::new()));
                    *game_state = None;
                    net.close_connection();
                    None
                }
                8 => {
                    net.close_connection();
                    *is_running = false;
                    None
                }
                _ => Some(self),
            }
        } else {
            Some(self)
        }
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        game_state: &GameState,
        bp: &Blueprints,
        net: &Net,
        settings: &Settings,
    ) {
        use Constraint::*;

        if let Some(ref settings_menu) = self.settings {
            settings_menu.render(frame, frame.size(), settings);
            return;
        }

        if let Some(ref load) = self.load {
            load.render(frame, frame.size(), bp);
            return;
        }

        if let Some(ref review) = self.review {
            review.render(frame, frame.size(), game_state, bp);
            return;
        }

        if let Some(ref library) = self.library {
            library.render(frame, frame.size(), bp);
            return;
        }

        if let Some(ref offset) = self.log {
            let [_, inner] = Layout::vertical([Fill(1), Max(10)]).areas(frame.size());
            let inner = bordered(frame, inner);
            if game_state.turn_timeline.is_empty() {
                frame.render_widget(
                    Paragraph::new(format!("No actions have been taken in this turn"))
                        .alignment(Alignment::Center),
                    inner,
                );
            } else {
                let mut state = TableState::new().with_selected(*offset as usize);
                frame.render_stateful_widget(
                    Table::new(
                        game_state.turn_timeline.iter().enumerate().map(|(i, act)| {
                            Row::new(vec![
                                Cell::new(Line::from(format!("{:4}", i))),
                                Cell::new(Line::from(act.view(game_state.board.bp()))),
                            ])
                        }),
                        [Max(6), Fill(1)],
                    )
                    .highlight_style(Style::default().bg(Color::DarkGray)),
                    inner,
                    &mut state,
                );
            }
            return;
        }

        if let Some(ref save) = self.save {
            let inner = popup(frame, frame.size(), layout::Size::new(60, 6));
            let [header, _, body] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(inner);
            frame.render_widget(
                Paragraph::new(format!(
                    "Save Game (path: {})",
                    get_data_dir_sub("saves")
                        .unwrap_or_default()
                        .to_str()
                        .unwrap(),
                ))
                .alignment(Alignment::Center),
                header,
            );
            frame.render_widget(
                Paragraph::new(format!("{}", save)).alignment(Alignment::Center),
                body,
            );
            return;
        }

        let inner = popup(frame, frame.size(), layout::Size::new(60, 14));

        let [header, _, body] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(inner);
        frame.render_widget(
            Paragraph::new("Pause Menu").alignment(Alignment::Center),
            header,
        );

        let [menu, _, lobby] = if net.connection.is_some() {
            Layout::horizontal([Fill(1), Length(1), Length(40)]).areas(body)
        } else {
            [body, Rect::default(), Rect::default()]
        };

        let mut state = TableState::new().with_selected(self.cursor as usize);
        frame.render_stateful_widget(
            Table::new(
                self.choices.iter().map(|c| {
                    Row::new(vec![Cell::new(
                        Line::from(c.clone()).alignment(Alignment::Center),
                    )])
                }),
                [60],
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            menu,
            &mut state,
        );

        let lobby = popup(frame, lobby, layout::Size::new(40, 18));
        let [header, lobby] = Layout::vertical([Length(1), Fill(1)]).areas(lobby);
        frame.render_widget(Paragraph::new("Lobby"), header);
        frame.render_widget(
            Table::new(
                net.get_members().iter().map(|(_id, member)| {
                    let color = Color::Rgb(member.color[0], member.color[1], member.color[2]);
                    Row::new(vec![Cell::new(
                        Line::from(member.name.clone())
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(text_color_contrast(color)).bg(color)),
                    )])
                }),
                [40],
            ),
            lobby,
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct MenuEmpireReview {
    player: Option<PlayerId>,
}

impl MenuEmpireReview {
    pub fn new() -> Self {
        MenuEmpireReview { player: None }
    }

    pub fn input(mut self, input: MenuInput, game_state: &Option<GameState>) -> Option<Self> {
        if input.select || input.back {
            return None;
        }
        if input.acc.x != 0 {
            if let Some(ref game_state) = game_state {
                let player_id = self
                    .player
                    .as_ref()
                    .unwrap_or(&game_state.board.current_player_turn);
                let index = game_state.board.player_index(player_id);
                let i = (index as i32 + input.acc.x)
                    .clamp(0, game_state.board.player_turn_order.len() as i32);
                if let Some(player_id) = game_state.board.player_turn_order.get(i as usize) {
                    self.player = Some(player_id.clone());
                }
            }
        }
        Some(self)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, game_state: &GameState, bp: &Blueprints) {
        use Constraint::*;

        let player_id = self
            .player
            .as_ref()
            .unwrap_or(&game_state.board.current_player_turn);

        frame.render_widget(Clear::default(), area);

        let [header, _, body] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(area);
        frame.render_widget(
            Paragraph::new("Empire Review").alignment(Alignment::Center),
            header,
        );

        let [player_bar, _, production_area, _, constraints_area, _, ok_area] = Layout::vertical([
            Length(1),
            Length(1),
            Fill(2),
            Length(1),
            Length(1),
            Length(1),
            Length(1),
        ])
        .areas(body);

        let players = &game_state.board.player_turn_order;

        frame.render_widget(
            Table::new(
                [Row::new(players.iter().map(|player| {
                    let player = game_state.board.get_player(player);
                    let color = Color::from_u32(player.color);
                    Cell::new(
                        Line::from(player.name.as_ref())
                            .fg(text_color_contrast(color))
                            .bg(color),
                    )
                }))],
                players.iter().map(|player| {
                    if player == player_id {
                        Fill(5)
                    } else {
                        Fill(1)
                    }
                }),
            ),
            player_bar,
        );

        let [prod_header, _, production_area] =
            Layout::vertical([Length(1), Length(1), Fill(1)]).areas(production_area);

        frame.render_widget(
            Paragraph::new(format!(
                "Production at the end of Day {}",
                game_state.board.day + 1
            ))
            .alignment(Alignment::Center),
            prod_header,
        );

        let (produced_resources, tally) = calculate_production(&game_state.board, player_id, true);
        let mut tally_grouped = HashMap::<UnitId, Resources>::new();
        for (unit_id, r) in tally {
            *tally_grouped.entry(unit_id).or_insert(Resources::new(0, 0)) += r;
        }
        let mut tally_sorted: Vec<(UnitId, Resources)> = tally_grouped.into_iter().collect();
        tally_sorted.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

        let [food_area, _, gold_area] =
            Layout::horizontal([Fill(1), Length(1), Fill(1)]).areas(production_area);

        for (res, area) in [(Resource::Food, food_area), (Resource::Gold, gold_area)] {
            let [header, total, area] =
                Layout::vertical([Length(1), Length(1), Fill(1)]).areas(area);
            frame.render_widget(
                Paragraph::new(format!("               {}", res.view())).fg(resource_color(&res)),
                header,
            );
            frame.render_widget(
                Paragraph::new(format!(
                    "Total          {}",
                    produced_resources.get_res(&res)
                ))
                .fg(resource_color(&res)),
                total,
            );

            frame.render_widget(
                Table::new(
                    tally_sorted
                        .iter()
                        .filter_map(|(unit_id, r)| {
                            (r.get_res(&res) > 0).then_some((
                                bp.get_unit(unit_id).header.name.clone(),
                                r.get_res(&res),
                            ))
                        })
                        .map(|(name, amt)| {
                            Row::new(vec![
                                Cell::new(Line::from(name)),
                                Cell::new(Line::from(amt.to_string())),
                            ])
                        }),
                    [14, 8],
                ),
                area,
            );
        }

        let count = game_state
            .board
            .get_player_units_pos(player_id)
            .filter(|(u, _)| game_state.board.unit_loc(u) == UnitLocation::Top)
            .count();
        let constraint = ((produced_resources.food + produced_resources.gold) / 100).max(7);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::from(format!("Unit train limit: {}/{} ", count, constraint)),
                Span::from("Formula: ((total food + total gold) / 100).max(7)")
                    .style(Style::default().fg(Color::DarkGray)),
            ]))
            .alignment(Alignment::Center),
            constraints_area,
        );

        frame.render_widget(
            Button {
                string: "Ok",
                pressed: true,
            },
            ok_area,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct MenuLibrary {
    topics: Vec<String>,
    cursor_topics: i32,

    blueprint_list: Vec<Id>,
    cursor_blueprints: Option<i32>,

    blueprint_id: Option<Id>,
    base_bonuses: bool,
}

impl MenuLibrary {
    pub fn new() -> Self {
        MenuLibrary {
            cursor_topics: 0,
            topics: vec![
                "Terrain",
                "Units",
                "Technologies",
                "Abilities",
                "Powers",
                "Civilization",
                "Base Bonuses",
            ]
            .iter()
            .map(|t| t.to_string())
            .collect(),
            blueprint_list: vec![],
            cursor_blueprints: None,
            blueprint_id: None,
            base_bonuses: false,
        }
    }

    pub fn input(mut self, input: MenuInput, bp: &Blueprints) -> Option<Self> {
        if self.base_bonuses {
            if input.back {
                self.blueprint_id = None;
            }
            return Some(self);
        }

        if let Some(ref _blueprint_id) = self.blueprint_id {
            if input.back {
                self.blueprint_id = None;
            }
            return Some(self);
        }

        if let Some(cursor_blueprints) = self.cursor_blueprints {
            let cursor =
                (cursor_blueprints + input.acc.y).clamp(0, self.blueprint_list.len() as i32 - 1);
            self.cursor_blueprints = Some(cursor);
            if input.back {
                self.cursor_blueprints = None;
            }
            if input.select {
                self.blueprint_id = Some(self.blueprint_list[cursor as usize].clone());
            }
            return Some(self);
        }

        if input.back {
            return None;
        }
        if input.select {
            let topic = &self.topics[self.cursor_topics as usize];
            let list: Option<Vec<Id>> = match topic.as_str() {
                "Terrain" => Some(bp.terrain.iter().map(|(id, _)| id.clone().into()).collect()),
                "Units" => Some(bp.units.iter().map(|(id, _)| id.clone().into()).collect()),
                "Technologies" => Some(bp.techs.iter().map(|(id, _)| id.clone().into()).collect()),
                "Abilities" => Some(
                    bp.abilities
                        .iter()
                        .map(|(id, _)| id.clone().into())
                        .collect(),
                ),
                "Powers" => Some(bp.powers.iter().map(|(id, _)| id.clone().into()).collect()),
                "Civilization" => Some(
                    bp.civilizations
                        .iter()
                        .map(|(id, _)| id.clone().into())
                        .collect(),
                ),
                "Base Bonuses" => {
                    self.base_bonuses = true;
                    None
                }
                _ => None,
            };
            if let Some(mut list) = list {
                if !list.is_empty() {
                    list.sort_by(|a, b| bp.get(a).get_name().cmp(&bp.get(b).get_name()));
                    self.blueprint_list = list;
                    self.cursor_blueprints = Some(0);
                }
            }
        }
        self.cursor_topics =
            (self.cursor_topics + input.acc.y).clamp(0, self.topics.len() as i32 - 1);
        Some(self)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, bp: &Blueprints) {
        use Constraint::*;

        if self.base_bonuses {
            let inner = bordered(frame, area);
            frame.render_widget(
                AllBonusesWidget {
                    bonuses: &vec![],
                    unit_bonuses: &vec![],
                    battle_bonuses: &bp.base_bonuses,
                    bp,
                },
                inner,
            );
            return;
        }

        if let Some(ref blueprint_id) = self.blueprint_id {
            let inner = bordered(frame, area);
            frame.render_widget(
                BlueprintWidget {
                    blueprint: &bp.get(blueprint_id),
                    bp,
                },
                inner,
            );
            return;
        }

        if let Some(ref cursor_blueprints) = self.cursor_blueprints {
            let inner = bordered(frame, area);
            let [header, _, body] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(inner);
            frame.render_widget(
                Paragraph::new(self.topics[self.cursor_topics as usize].as_str())
                    .alignment(Alignment::Center),
                header,
            );
            let mut state = TableState::new().with_selected(*cursor_blueprints as usize);
            frame.render_stateful_widget(
                Table::new(
                    self.blueprint_list.iter().map(|id| {
                        Row::new(vec![Cell::new(
                            Line::from(bp.get(id).get_name()).alignment(Alignment::Center),
                        )])
                    }),
                    [Fill(1)],
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                body,
                &mut state,
            );
            return;
        }

        let inner = popup(
            frame,
            frame.size(),
            layout::Size::new(40, self.topics.len() as u16 + 4),
        );
        let [header, _, body] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(inner);
        frame.render_widget(
            Paragraph::new("Library").alignment(Alignment::Center),
            header,
        );
        let mut state = TableState::new().with_selected(self.cursor_topics as usize);
        frame.render_stateful_widget(
            Table::new(
                self.topics.iter().map(|topic| {
                    Row::new(vec![Cell::new(
                        Line::from(topic.as_str()).alignment(Alignment::Center),
                    )])
                }),
                [Fill(1)],
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            body,
            &mut state,
        );
    }
}

struct BlueprintWidget<'a> {
    blueprint: &'a Blueprint<'a>,
    bp: &'a Blueprints,
}

impl<'a> Widget for BlueprintWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.blueprint {
            Blueprint::Tech(tech) => BlueprintTechWidget { tech, bp: self.bp }.render(area, buf),
            Blueprint::Unit(unit) => BlueprintUnitWidget { unit, bp: self.bp }.render(area, buf),
            Blueprint::Ability(ability) => BlueprintAbilityWidget {
                ability,
                bp: self.bp,
            }
            .render(area, buf),
            Blueprint::Terrain(terrain) => BlueprintTerrainWidget { terrain }.render(area, buf),
            Blueprint::Power(power) => {
                BlueprintPowerWidget { power, bp: self.bp }.render(area, buf)
            }
            Blueprint::Civilization(civ) => {
                BlueprintCivilizationWidget { civ, bp: self.bp }.render(area, buf)
            }
        }
    }
}

pub struct BlueprintUnitWidget<'a> {
    pub unit: &'a UnitBlueprint,
    pub bp: &'a Blueprints,
}

impl<'a> Widget for BlueprintUnitWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        //todo: refactor
        UnitBlueprintWidget {
            blueprints: self.bp,
            id: &self.unit.header.id,
        }
        .render(area, buf)
    }
}

pub struct BlueprintCivilizationWidget<'a> {
    pub civ: &'a CivilizationBlueprint,
    pub bp: &'a Blueprints,
}

impl<'a> Widget for BlueprintCivilizationWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let [name, _, heroes, _, discounts, rest] = Layout::vertical([
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Fill(1),
        ])
        .areas(area);
        Paragraph::new(format!("#{}#", self.civ.name)).render(name, buf);
        Paragraph::new(format!(
            "Heroes:         {}",
            self.civ
                .heroes
                .iter()
                .map(|id| self.bp.get(id).get_name())
                .fold(String::new(), |acc, n| acc + n + "  ")
        ))
        .render(heroes, buf);

        let [head, body] = Layout::horizontal([Length(15), Fill(1)]).areas(discounts);
        Paragraph::new("Tech Discount").render(head, buf);
        ResourcesWidget::new(&self.civ.tech_discount).render(body, buf);

        AllBonusesWidget::new(&vec![], &self.civ.unit_bonuses, &vec![], self.bp).render(rest, buf)
    }
}

pub struct BlueprintPowerWidget<'a> {
    pub power: &'a PowerBlueprint,
    pub bp: &'a Blueprints,
}

impl<'a> Widget for BlueprintPowerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let [name, descr] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        Paragraph::new(format!("#{}#", self.power.name)).render(name, buf);
        let mut description = String::new();
        if self.power.require_on_building != UnitConstraint::NoConstraint {
            description += &format!(
                "Requires that this unit to be on {}\n",
                self.power.require_on_building.view(&self.bp)
            );
        }
        if self.power.targets != PowerTargets::default() {
            description += &format!("Targets {}", self.power.targets.view());
        }
        if self.power.bonus != Bonus::default() {
            description += &format!(
                "\nThe targeted units gain:\n{}",
                self.power.bonus.view(&self.bp)
            );
        }
        if self.power.unit_bonus != UnitBonus::default() {
            description += &format!(
                "\nThe targeted units obtain:\n{}",
                &self.power.unit_bonus.view(&self.bp)
            );
        }
        if self.power.battle_bonus != BattleBonus::default() {
            description += &format!(
                "\nThe targeted units obtain:\n{}",
                &self.power.battle_bonus.view(&self.bp)
            );
        }
        if self.power.effects != PowerBlueprint::default().effects {
            description += &format!("\n");
            for effect in self.power.effects.iter() {
                description += &(format!("Effect: {}", effect.view() + "\n"));
            }
        }
        Paragraph::new(
            description
                .split("\n")
                .map(|s| Line::raw(s))
                .collect::<Vec<Line>>(),
        )
        .wrap(Wrap { trim: false })
        .render(descr, buf);
    }
}

pub struct BlueprintAbilityWidget<'a> {
    pub ability: &'a AbilityBlueprint,
    pub bp: &'a Blueprints,
}

impl<'a> Widget for BlueprintAbilityWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let [name, _, rest] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(area);
        Paragraph::new(format!("^{}^", self.ability.name)).render(name, buf);
        AllBonusesWidget::new(
            &self.ability.unit_bonuses,
            &vec![],
            &self.ability.battle_bonuses,
            self.bp,
        )
        .render(rest, buf);
    }
}

pub struct BlueprintTerrainWidget<'a> {
    pub terrain: &'a TerrainBlueprint,
}

impl<'a> Widget for BlueprintTerrainWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let [name, glyph, rest] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(area);
        Paragraph::new(format!("_{}_", self.terrain.header.name)).render(name, buf);

        let [glyph, _, tile] = Layout::horizontal([Length(9), Length(1), Length(3)]).areas(glyph);
        Paragraph::new("Glyph").render(glyph, buf);
        let color = terrain_tile_color(
            &TerrainTile {
                blueprint_id: self.terrain.header.id.clone(),
                ..Default::default()
            },
            self.terrain,
        );
        Paragraph::new(self.terrain.header.glyph.as_str())
            .bg(color)
            .fg(text_color_contrast(color))
            .render(tile, buf);

        let [a, b, c, d, e] =
            Layout::vertical([Length(1), Length(1), Length(1), Length(1), Length(1)]).areas(rest);
        Paragraph::new(format!("Move Cost     {:4}", self.terrain.stats.move_cost)).render(a, buf);
        Paragraph::new(format!("Sight Cost    {:4}", self.terrain.stats.sight_cost)).render(b, buf);
        Paragraph::new(format!(
            "Range Bonus   {:4}",
            self.terrain.stats.range_bonus
        ))
        .render(c, buf);
        Paragraph::new(format!(
            "Defence Bonus {:4}%",
            self.terrain.stats.defence_bonus
        ))
        .render(d, buf);
        Paragraph::new(format!(
            "Sight Bonus   {:4}",
            self.terrain.stats.sight_bonus
        ))
        .render(e, buf);
    }
}

pub struct BlueprintTechWidget<'a> {
    pub tech: &'a TechBlueprint,
    pub bp: &'a Blueprints,
}

impl<'a> Widget for BlueprintTechWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let [name, level, _, cost, _, require, other] = Layout::vertical([
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Fill(1),
        ])
        .areas(area);
        Paragraph::new(format!("*{}*", self.tech.name)).render(name, buf);
        Paragraph::new(format!("{}", view_level(self.tech.level))).render(level, buf);
        ResourcesCostWidget::new(&self.tech.cost).render(cost, buf);
        UnitConstraintWidget::new(&self.tech.require, self.bp).render(require, buf);
        AllBonusesWidget::new(
            &vec![],
            &self.tech.unit_bonuses,
            &self.tech.battle_bonuses,
            self.bp,
        )
        .render(other, buf);
    }
}

struct ResourcesWidget<'a> {
    resources: &'a Resources,
}

impl<'a> ResourcesWidget<'a> {
    fn new(resources: &'a Resources) -> Self {
        Self { resources }
    }
}

impl<'a> Widget for ResourcesWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new(Line::from(vec![
            Span::raw(format!("{} food ", self.resources.food)).fg(resource_color(&Resource::Food)),
            Span::raw(format!("{} gold", self.resources.gold)).fg(resource_color(&Resource::Gold)),
        ]))
        .render(area, buf);
    }
}

struct ResourcesCostWidget<'a> {
    resources: &'a Resources,
}

impl<'a> ResourcesCostWidget<'a> {
    fn new(resources: &'a Resources) -> Self {
        Self { resources }
    }
}

impl<'a> Widget for ResourcesCostWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let [head, body] = Layout::horizontal([Length(9), Fill(1)]).areas(area);
        Paragraph::new("Cost").render(head, buf);
        ResourcesWidget::new(self.resources).render(body, buf)
    }
}

struct UnitConstraintWidget<'a> {
    unit_constraint: &'a UnitConstraint,
    bp: &'a Blueprints,
}

impl<'a> UnitConstraintWidget<'a> {
    fn new(unit_constraint: &'a UnitConstraint, bp: &'a Blueprints) -> Self {
        Self {
            unit_constraint,
            bp,
        }
    }
}

impl<'a> Widget for UnitConstraintWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new("Requires ".to_string() + &self.unit_constraint.view(self.bp))
            .render(area, buf);
    }
}

struct AllBonusesWidget<'a> {
    bonuses: &'a Vec<Bonus>,
    unit_bonuses: &'a Vec<UnitBonus>,
    battle_bonuses: &'a Vec<BattleBonus>,
    bp: &'a Blueprints,
}

impl<'a> AllBonusesWidget<'a> {
    fn new(
        bonuses: &'a Vec<Bonus>,
        unit_bonuses: &'a Vec<UnitBonus>,
        battle_bonuses: &'a Vec<BattleBonus>,
        bp: &'a Blueprints,
    ) -> Self {
        Self {
            bonuses,
            unit_bonuses,
            battle_bonuses,
            bp,
        }
    }
}

impl<'a> Widget for AllBonusesWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let a = self.bonuses.iter().map(|b| b.view(self.bp) + "\n");
        let b = self.unit_bonuses.iter().map(|b| b.view(self.bp) + "\n");
        let c = self.battle_bonuses.iter().map(|b| b.view(self.bp) + "\n");
        Paragraph::new(
            a.chain(b)
                .chain(c)
                .fold(String::new(), |acc, s| acc + &s)
                .split("\n")
                .map(|s| Line::raw(s))
                .collect::<Vec<Line>>(),
        )
        .wrap(Wrap { trim: false })
        .render(area, buf);
    }
}
