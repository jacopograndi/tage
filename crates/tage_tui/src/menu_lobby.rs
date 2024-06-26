use std::{fs, sync::Arc};

use tage_core::prelude::*;

use crate::*;

#[derive(Debug, Clone)]
pub enum LobbySection {
    Back,
    AddPlayer,
    PlayerList {
        choices: Vec<MapPlayerSettings>,
        cursor: i32,
        column: i32,
    },
    SelectMap,
    WithHero,
    Fog,
    Start,
}

#[derive(Debug, Clone)]
pub struct LobbySelectMap {
    pub choices: Vec<String>,
    pub cursor: i32,
    pub maps: Vec<Board>,
}

impl LobbySelectMap {
    pub fn new(bp: &Blueprints, players: Vec<Player>, path: &str) -> Self {
        let paths = fs::read_dir("./assets/maps/").unwrap();

        let mut choices: Vec<(String, Board)> = paths
            .filter_map(|path| {
                let path = path.unwrap().path().to_str().unwrap().to_string();
                load_map(
                    bp,
                    &MapSettings {
                        path: path.clone(),
                        players: vec![],
                        place_hero: true,
                        fog_base: FogTile::Visible,
                    },
                )
                .ok()
                .map(|grid| {
                    let board = Board {
                        bp: Arc::new(bp.clone()),
                        grid,
                        players: players.clone(),
                        day: 0,
                        current_player_turn: PlayerId::new(0),
                        player_turn_order: vec![],
                        fog: HashMap::new(),
                        fog_base: FogTile::Visible,
                    };
                    (path, board)
                })
            })
            .collect();

        choices.sort_by(|a, b| a.0.cmp(&b.0));

        let cursor = choices
            .iter()
            .enumerate()
            .find(|(_, (c, _))| c == path)
            .map(|(i, _)| i as i32)
            .unwrap_or(0);

        Self {
            choices: choices.iter().map(|(c, _)| c.clone()).collect(),
            cursor,
            maps: choices.into_iter().map(|(_, board)| board).collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LobbySelectTeam {
    pub choices: Vec<Option<TeamId>>,
    pub cursor: i32,
}

#[derive(Debug, Clone, Default)]
pub struct LobbySelectColor {
    pub color: [u8; 3],
    pub cursor: i32,
}

#[derive(Debug, Clone, Default)]
pub struct LobbySelectCivilization {
    pub choices: Vec<CivilizationId>,
    pub cursor: i32,
}

#[derive(Debug, Clone, Default)]
pub struct LobbySelectController {
    pub choices: Vec<Controller>,
    pub cursor: i32,
}

#[derive(Debug, Clone, Default)]
pub struct MenuLobby {
    sections: Vec<LobbySection>,
    map_settings: MapSettings,
    cursor: i32,
    select_map: Option<LobbySelectMap>,
    chosen_map: Option<Board>,
    select_team: Option<LobbySelectTeam>,
    select_name: Option<String>,
    select_civilization: Option<LobbySelectCivilization>,
    select_symbol: Option<String>,
    select_color: Option<LobbySelectColor>,
    select_controller: Option<LobbySelectController>,
    select_hero: bool,
    select_fog_base: FogTile,
}

// a bit long
impl MenuLobby {
    pub fn new(ui: &InterfaceState, net: &Net) -> Self {
        let choices = if net.connection.is_some() {
            vec![]
        } else {
            vec![MapPlayerSettings {
                name: ui.member_profile.name.clone(),
                symbol: ui.member_profile.symbol.clone(),
                color: color_to_u32(ui.member_profile.color),
                civilization: ui.member_profile.civilization.clone(),
                ..Default::default()
            }]
        };
        Self {
            sections: vec![
                LobbySection::Back,
                LobbySection::AddPlayer,
                LobbySection::PlayerList {
                    choices,
                    column: 0,
                    cursor: 0,
                },
                LobbySection::SelectMap,
                LobbySection::WithHero,
                LobbySection::Fog,
                LobbySection::Start,
            ],
            map_settings: MapSettings::default(),
            cursor: 0,
            select_hero: true,
            ..Default::default()
        }
    }

    fn get_player_list(&mut self) -> &mut Vec<MapPlayerSettings> {
        let Some(LobbySection::PlayerList { choices, .. }) = self
            .sections
            .iter_mut()
            .find(|sect| matches!(sect, LobbySection::PlayerList { .. }))
        else {
            unreachable!();
        };
        choices
    }

    fn get_selected_player(&mut self) -> &mut MapPlayerSettings {
        let Some(LobbySection::PlayerList {
            choices, cursor, ..
        }) = self
            .sections
            .iter_mut()
            .find(|sect| matches!(sect, LobbySection::PlayerList { .. }))
        else {
            unreachable!();
        };
        &mut choices[*cursor as usize]
    }

    pub fn input(&self, input: MenuInput, bp: &Blueprints, net: &mut Net) -> MenuState {
        let mut next = self.clone();

        let mut send_member: Option<Member> = None;
        let mut send_settings = false;

        let player_list = next.get_player_list();
        let members = net.get_members();
        for (id, member) in members.iter() {
            if player_list
                .iter()
                .find(|p| {
                    if let Controller::Remote(raw) = p.controller {
                        &ClientId::from_raw(raw) == id
                    } else {
                        false
                    }
                })
                .is_none()
            {
                player_list.push(MapPlayerSettings {
                    id: PlayerId::new(player_list.len() as u32),
                    name: member.name.clone(),
                    symbol: member.symbol.clone(),
                    color: color_to_u32(member.color),
                    controller: Controller::Remote(id.raw()),
                    civilization: member.civilization.clone(),
                    ..Default::default()
                });

                send_settings = true;
            }
        }

        player_list.retain(|p| {
            if let Controller::Remote(id) = p.controller {
                send_settings = true;
                members.contains_key(&ClientId::from_raw(id))
            } else {
                true
            }
        });

        if let Some(Connection::Client(client)) = &mut net.connection {
            while let Some(message) = client.queue.pop() {
                match message {
                    ServerMessages::MapSettings { mut map_settings } => {
                        next.map_settings = map_settings.clone();
                        next.select_hero = map_settings.place_hero;
                        let player_list = next.get_player_list();
                        player_list.clear();
                        player_list.append(&mut map_settings.players);
                        if map_settings.path != "" {
                            next.chosen_map = Some(Board {
                                bp: Arc::new(bp.clone()),
                                grid: load_map(
                                    bp,
                                    &MapSettings {
                                        path: map_settings.path.clone(),
                                        place_hero: self.select_hero,
                                        players: vec![],
                                        fog_base: self.select_fog_base.clone(),
                                    },
                                )
                                .unwrap(),
                                players: player_list
                                    .iter()
                                    .map(|c| Player {
                                        id: c.id.clone(),
                                        color: c.color,
                                        ..Default::default()
                                    })
                                    .collect(),
                                day: 0,
                                current_player_turn: PlayerId::new(0),
                                player_turn_order: vec![],
                                fog: HashMap::new(),
                                fog_base: self.select_fog_base.clone(),
                            });
                        }
                    }
                    ServerMessages::ToGame => {
                        return MenuState::Play(next.map_settings.clone());
                    }
                    _ => {}
                }
            }
        }

        if let Some(Connection::Server(server)) = &mut net.connection {
            while let Some(message) = server.connection_queue.pop() {
                match message {
                    ServerConnectionMessages::Connected(_) => {
                        send_settings = true;
                    }
                    ServerConnectionMessages::Disconnected(_) => {
                        send_settings = true;
                    }
                }
            }

            while let Some((client_id, message)) = server.queue.pop() {
                match message {
                    ClientMessages::MemberChange { member } => {
                        let player_list = next.get_player_list();
                        if let Some(player_setting) = player_list.iter_mut().find(|p| {
                            if let Controller::Remote(raw) = p.controller {
                                ClientId::from_raw(raw) == client_id
                            } else {
                                false
                            }
                        }) {
                            player_setting.name = member.name;
                            player_setting.symbol = member.symbol;
                            player_setting.color = color_to_u32(member.color);
                            player_setting.civilization = member.civilization;
                        }
                        send_settings = true;
                    }
                    _ => {}
                }
            }
        }

        let current_section = &mut next.sections[next.cursor as usize];
        if let Some(select_map) = &mut next.select_map {
            select_map.cursor =
                (select_map.cursor + input.acc.y).clamp(0, select_map.choices.len() as i32 - 1);
        } else if let Some(select_team) = &mut next.select_team {
            select_team.cursor =
                (select_team.cursor + input.acc.y).clamp(0, select_team.choices.len() as i32 - 1);
        } else if let Some(select_color) = &mut next.select_color {
            select_color.cursor =
                (select_color.cursor + input.acc.y).clamp(0, select_color.color.len() as i32 - 1);
            select_color.color[select_color.cursor as usize] =
                (select_color.color[select_color.cursor as usize] as i32 + input.acc.x)
                    .clamp(0, 255) as u8;
        } else if let Some(select_name) = &mut next.select_name {
            match input.keycode {
                Some(KeyCode::Backspace) => {
                    select_name.pop();
                }
                Some(KeyCode::Char(c)) => {
                    if ALLOWED_NAME_CHARS.contains(&c) && select_name.len() < 32 {
                        select_name.push(c);
                    }
                }
                _ => {}
            }
        } else if let Some(select_symbol) = &mut next.select_symbol {
            match input.keycode {
                Some(KeyCode::Char(c)) => {
                    *select_symbol = c.to_string();
                }
                _ => {}
            }
        } else if let Some(select_controller) = &mut next.select_controller {
            select_controller.cursor = (select_controller.cursor + input.acc.y)
                .clamp(0, select_controller.choices.len() as i32 - 1);
        } else if let Some(select_civilization) = &mut next.select_civilization {
            select_civilization.cursor = (select_civilization.cursor + input.acc.y)
                .clamp(0, select_civilization.choices.len() as i32 - 1);
        } else {
            match current_section {
                LobbySection::PlayerList {
                    choices,
                    cursor,
                    column,
                } => {
                    if choices.is_empty() {
                        next.cursor =
                            (self.cursor + input.acc.y).clamp(0, self.sections.len() as i32 - 1);
                    } else {
                        *column = (*column + input.acc.x).clamp(0, 6);
                        *cursor = *cursor + input.acc.y;
                        if *cursor < 0 {
                            *cursor = 0;
                            next.cursor =
                                (self.cursor - 1).clamp(0, self.sections.len() as i32 - 1);
                        } else if *cursor >= choices.len() as i32 {
                            *cursor = (choices.len() as i32 - 1).max(0);
                            next.cursor =
                                (self.cursor + 1).clamp(0, self.sections.len() as i32 - 1);
                        }
                    }
                }
                _ => {
                    next.cursor =
                        (self.cursor + input.acc.y).clamp(0, self.sections.len() as i32 - 1);
                }
            }
        }

        let mut next_state = MenuState::Lobby(next.clone());
        if input.select {
            if let Some(_) = net.error.pop() {
                return MenuState::Home(MenuHome::new());
            }

            if let Some(select_map) = next.select_map.take() {
                let c = select_map.cursor as usize;
                if !net.is_client() {
                    next.map_settings.path = select_map.choices[c].clone();
                    next.chosen_map = Some(select_map.maps[c].clone());
                }
                next_state = MenuState::Lobby(next.clone());
            } else if let Some(select_team) = next.select_team.take() {
                let c = select_team.cursor as usize;
                next.get_selected_player().team = select_team.choices[c].clone();
                next_state = MenuState::Lobby(next.clone());
            } else if let Some(select_name) = next.select_name.take() {
                next.get_selected_player().name = select_name.clone();
                next_state = MenuState::Lobby(next.clone());
                if let Controller::Remote(id) = next.get_selected_player().controller {
                    match &mut net.connection {
                        Some(Connection::Client(client)) => {
                            let client_id = ClientId::from_raw(id);
                            if Some(client_id) == client.get_local_id() {
                                if let Some(member) = client.members.get_mut(&client_id) {
                                    member.name = select_name;
                                    send_member = Some(member.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else if let Some(select_symbol) = next.select_symbol.take() {
                next.get_selected_player().symbol = select_symbol.clone();
                next_state = MenuState::Lobby(next.clone());
                if let Controller::Remote(id) = next.get_selected_player().controller {
                    match &mut net.connection {
                        Some(Connection::Client(client)) => {
                            let client_id = ClientId::from_raw(id);
                            if Some(client_id) == client.get_local_id() {
                                if let Some(member) = client.members.get_mut(&client_id) {
                                    member.symbol = select_symbol;
                                    send_member = Some(member.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else if let Some(select_color) = next.select_color.take() {
                next.get_selected_player().color = color_to_u32(select_color.color);
                next_state = MenuState::Lobby(next.clone());
                if let Controller::Remote(id) = next.get_selected_player().controller {
                    match &mut net.connection {
                        Some(Connection::Server(server)) => {
                            if let Some(member) = server.members.get_mut(&ClientId::from_raw(id)) {
                                member.color = select_color.color;
                            }
                        }
                        Some(Connection::Client(client)) => {
                            let client_id = ClientId::from_raw(id);
                            if Some(client_id) == client.get_local_id() {
                                if let Some(member) = client.members.get_mut(&client_id) {
                                    member.color = select_color.color;
                                    send_member = Some(member.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else if let Some(select_controller) = next.select_controller.take() {
                let c = select_controller.cursor as usize;
                next.get_selected_player().controller = select_controller.choices[c].clone();
                next_state = MenuState::Lobby(next.clone());
            } else if let Some(select_civilization) = next.select_civilization.take() {
                let c = select_civilization.cursor as usize;
                next.get_selected_player().civilization = bp
                    .get_civilization(&select_civilization.choices[c])
                    .name
                    .clone();
                next_state = MenuState::Lobby(next.clone());
                if let Controller::Remote(id) = next.get_selected_player().controller {
                    match &mut net.connection {
                        Some(Connection::Client(client)) => {
                            let client_id = ClientId::from_raw(id);
                            if Some(client_id) == client.get_local_id() {
                                if let Some(member) = client.members.get_mut(&client_id) {
                                    member.civilization = bp
                                        .get_civilization(&select_civilization.choices[c])
                                        .name
                                        .clone();
                                    send_member = Some(member.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                match &mut next.sections[next.cursor as usize] {
                    LobbySection::Back => {
                        if net.connection.is_some() {
                            net.close_connection()
                        }
                        next_state = MenuState::Home(MenuHome::new())
                    }
                    LobbySection::AddPlayer if !net.is_client() => {
                        if let Some(LobbySection::PlayerList { choices, .. }) = next
                            .sections
                            .iter_mut()
                            .find(|sect| matches!(sect, LobbySection::PlayerList { .. }))
                        {
                            const COOL_COLORS: [u32; 8] = [
                                0x00880800, 0x00ff3232, 0x00ff9932, 0x00ffff5a, 0x00adff5a,
                                0x00034400, 0x00003388, 0x00519be5,
                            ];

                            // el rando cheapo
                            let rand = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .subsec_nanos() as usize;

                            let civs: Vec<CivilizationId> =
                                bp.civilizations.keys().cloned().collect();

                            choices.push(MapPlayerSettings {
                                id: PlayerId::new(choices.len() as u32),
                                civilization: bp
                                    .get_civilization(&civs[rand % bp.civilizations.len()])
                                    .name
                                    .clone(),
                                color: COOL_COLORS[rand % 8],
                                controller: Controller::Human,
                                name: "Nameless".to_string(),
                                symbol: "'".to_string(),
                                ..Default::default()
                            });
                            next_state = MenuState::Lobby(next.clone());
                        }
                    }
                    LobbySection::SelectMap => {
                        next.select_map = Some(LobbySelectMap::new(
                            bp,
                            next.get_player_list()
                                .iter()
                                .map(|c| Player {
                                    id: c.id.clone(),
                                    color: c.color,
                                    ..Default::default()
                                })
                                .collect(),
                            next.map_settings.path.as_str(),
                        ));
                        next_state = MenuState::Lobby(next.clone());
                    }
                    LobbySection::PlayerList {
                        choices,
                        cursor,
                        column,
                    } => {
                        if !choices.is_empty() {
                            let local = if let Controller::Remote(id) =
                                &choices[*cursor as usize].controller
                            {
                                Some(ClientId::from_raw(*id)) == net.get_local_id()
                            } else {
                                true
                            };
                            match column {
                                0 if !net.is_client() => {
                                    next.select_team = Some(LobbySelectTeam {
                                        choices: [None]
                                            .into_iter()
                                            .chain((0..16).map(|id| Some(TeamId::new(id))))
                                            .collect::<Vec<Option<TeamId>>>(),
                                        cursor: 0,
                                    });
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                1 if local => {
                                    next.select_name = Some(choices[*cursor as usize].name.clone());
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                2 if local => {
                                    let u = choices[*cursor as usize].color;
                                    let r = (u >> 16) as u8;
                                    let g = (u >> 8) as u8;
                                    let b = u as u8;
                                    next.select_color = Some(LobbySelectColor {
                                        color: [r, g, b],
                                        cursor: 0,
                                    });
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                3 if local => {
                                    next.select_symbol =
                                        Some(choices[*cursor as usize].symbol.clone());
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                4 if !net.is_client() => {
                                    next.select_controller = Some(LobbySelectController {
                                        choices: vec![
                                            Controller::Human,
                                            Controller::Machine(MachineOpponent::WeakBoulder),
                                            Controller::Machine(MachineOpponent::AverageBoulder),
                                            Controller::Machine(MachineOpponent::StrongBoulder),
                                            Controller::Machine(MachineOpponent::WeakPeak),
                                            Controller::Machine(MachineOpponent::AveragePeak),
                                            Controller::Machine(MachineOpponent::StrongPeak),
                                        ],
                                        cursor: 0,
                                    });
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                5 if !net.is_client() => {
                                    next.select_civilization = Some(LobbySelectCivilization {
                                        choices: bp
                                            .civilizations
                                            .iter()
                                            .map(|(id, _)| id.clone())
                                            .collect(),
                                        cursor: 0,
                                    });
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                6 if !net.is_client() => {
                                    let player_setting = choices.remove(*cursor as usize);
                                    if net.is_server() {
                                        if let Controller::Remote(id) = &player_setting.controller {
                                            net.kick(raw!(*id));
                                        }
                                    }
                                    next_state = MenuState::Lobby(next.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                    LobbySection::Start if !net.is_client() => {
                        if next.chosen_map.is_some() {
                            next.map_settings = MapSettings {
                                players: next
                                    .get_player_list()
                                    .iter()
                                    .enumerate()
                                    .map(|(i, player)| MapPlayerSettings {
                                        id: PlayerId::new(i as u32),
                                        ..player.clone()
                                    })
                                    .collect(),
                                place_hero: next.select_hero,
                                fog_base: next.select_fog_base.clone(),
                                ..next.map_settings.clone()
                            };
                            if net.is_server() {
                                net.server_send(&ServerMessages::ToGame)
                            }
                            next_state = MenuState::Play(next.map_settings.clone())
                        }
                    }
                    LobbySection::WithHero if !net.is_client() => {
                        next.select_hero = !next.select_hero;
                        next_state = MenuState::Lobby(next.clone());
                    }
                    LobbySection::Fog if !net.is_client() => {
                        next.select_fog_base = match next.select_fog_base {
                            FogTile::Visible => FogTile::Explored,
                            FogTile::Explored => FogTile::Hidden,
                            FogTile::Hidden => FogTile::Visible,
                        };
                        next_state = MenuState::Lobby(next.clone());
                    }
                    _ => {}
                }
            }

            send_settings = true;
        }

        next.map_settings = MapSettings {
            players: next
                .get_player_list()
                .iter()
                .enumerate()
                .map(|(i, player)| MapPlayerSettings {
                    id: PlayerId::new(i as u32),
                    ..player.clone()
                })
                .collect(),
            place_hero: next.select_hero,
            fog_base: next.select_fog_base.clone(),
            ..next.map_settings.clone()
        };

        if net.is_server() && send_settings {
            net.server_send(&ServerMessages::MapSettings {
                map_settings: next.map_settings,
            })
        }

        if net.is_client() {
            if let Some(member) = send_member {
                net.client_send(&ClientMessages::MemberChange { member })
            }
        }

        next_state
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, bp: &Blueprints, net: &Net) {
        use Constraint::*;

        let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        frame.render_widget(Paragraph::new("Lobby").alignment(Alignment::Center), topbar);

        let [d0, center, d1] = Layout::horizontal([Fill(1), Max(78), Fill(1)]).areas(rest);

        frame.render_widget(PanelWidget::new(DECOR_1), d0);
        frame.render_widget(PanelWidget::new(DECOR_1), d1);

        for (i, (sect_area, section)) in
            Layout::vertical(self.sections.iter().map(|section| match section {
                LobbySection::Back => Length(3),
                LobbySection::AddPlayer => Length(1),
                LobbySection::PlayerList { .. } => Fill(2),
                LobbySection::SelectMap => Fill(1),
                LobbySection::WithHero => Length(1),
                LobbySection::Fog => Length(1),
                LobbySection::Start => Length(3),
            }))
            .split(center)
            .iter()
            .zip(self.sections.iter())
            .enumerate()
        {
            let selected = if i as i32 == self.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            match section {
                LobbySection::Start => {
                    let [_, center, _] =
                        Layout::vertical([Length(1), Fill(1), Length(1)]).areas(*sect_area);
                    frame.render_widget(Paragraph::new("Start").centered().style(selected), center)
                }
                LobbySection::SelectMap => {
                    if self.map_settings.path == "" {
                        frame.render_widget(
                            Paragraph::new("Select Map").centered().style(selected),
                            *sect_area,
                        )
                    } else {
                        let [mapname, map] =
                            Layout::vertical([Length(1), Fill(1)]).areas(*sect_area);
                        frame.render_widget(
                            Paragraph::new(format!(
                                "Map: {}",
                                self.map_settings
                                    .path
                                    .split("/")
                                    .last()
                                    .unwrap()
                                    .trim_end_matches(".txt")
                            ))
                            .centered()
                            .style(selected),
                            mapname,
                        );
                        if let Some(board) = &self.chosen_map {
                            frame.render_widget(
                                BoardWidget {
                                    board,
                                    blueprints: bp,
                                    cursor: board.grid.size / 2,
                                    attack_tiles: &vec![],
                                    movement_tiles: &vec![],
                                    target_tiles: &vec![],
                                    only_player_color: false,
                                    zoom: 3,
                                    show_spawns: false,
                                    travel_path: &vec![],
                                    fog_player: &PlayerId::new(0),
                                },
                                map,
                            );
                        }
                    }
                }
                LobbySection::AddPlayer => frame.render_widget(
                    Paragraph::new("Add Player").centered().style(selected),
                    *sect_area,
                ),
                LobbySection::PlayerList {
                    choices,
                    cursor,
                    column,
                } => {
                    let header = Row::new(vec![
                        Cell::new(Line::from("Team")),
                        Cell::new(Line::from("Player")),
                        Cell::new(Line::from("Color")),
                        Cell::new(Line::from("Symbol")),
                        Cell::new(Line::from("Controller")),
                        Cell::new(Line::from("Civilization")),
                        Cell::new(Line::from("Remove")),
                    ]);
                    let rows = choices.iter().enumerate().map(|(y, player)| {
                        let values = vec![
                            (
                                match &player.team {
                                    Some(team) => format!("Team {}", team.get()),
                                    None => format!("Alone"),
                                },
                                None,
                            ),
                            (player.name.clone(), None),
                            ("      ".into(), Some(Color::from_u32(player.color))),
                            (player.symbol.clone(), None),
                            (
                                match player.controller {
                                    Controller::Human => {
                                        if net.is_client() {
                                            format!("Host hotseat")
                                        } else {
                                            format!("Local player")
                                        }
                                    }
                                    Controller::Remote(id) => {
                                        if ClientId::from_raw(id) == HOST_CLIENT_ID {
                                            format!("Host player")
                                        } else if Some(ClientId::from_raw(id)) == net.get_local_id()
                                        {
                                            format!("Local player")
                                        } else {
                                            format!("Remote player")
                                        }
                                    }
                                    Controller::Machine(MachineOpponent::WeakBoulder) => {
                                        format!("Boulder Weak")
                                    }
                                    Controller::Machine(MachineOpponent::AverageBoulder) => {
                                        format!("Boulder Average")
                                    }
                                    Controller::Machine(MachineOpponent::StrongBoulder) => {
                                        format!("Boulder Strong")
                                    }
                                    Controller::Machine(MachineOpponent::WeakPeak) => {
                                        format!("Peak Weak")
                                    }
                                    Controller::Machine(MachineOpponent::AveragePeak) => {
                                        format!("Peak Average")
                                    }
                                    Controller::Machine(MachineOpponent::StrongPeak) => {
                                        format!("Peak Strong")
                                    }
                                    Controller::Machine(MachineOpponent::Boulder(ref _boulder)) => {
                                        format!("Tuned Boulder")
                                    }
                                    Controller::Machine(MachineOpponent::Peak(ref _peak)) => {
                                        format!("Tuned Peak")
                                    }
                                },
                                None,
                            ),
                            (
                                format!("{}", bp.get_civilization(&player.civ(bp)).name),
                                None,
                            ),
                            ("Remove".into(), None),
                        ];

                        Row::new(values.into_iter().enumerate().map(|(x, (v, color))| {
                            let style = if y as i32 == *cursor && x as i32 == *column {
                                if let Some(color) = color {
                                    Style::default()
                                        .bg(color)
                                        .add_modifier(Modifier::CROSSED_OUT)
                                } else {
                                    selected
                                }
                            } else {
                                if let Some(color) = color {
                                    Style::default().bg(color)
                                } else {
                                    Style::default().bg(Color::Rgb(40, 40, 40))
                                }
                            };
                            Cell::new(Line::from(v).style(style))
                        }))
                    });

                    let mut state = TableState::new().with_selected(*cursor as usize);
                    frame.render_stateful_widget(
                        Table::new(rows, [6, 25, 6, 6, 16, 12, 6]).header(header),
                        *sect_area,
                        &mut state,
                    );
                }
                LobbySection::Back => {
                    let [_, center, _] =
                        Layout::vertical([Length(1), Fill(1), Length(1)]).areas(*sect_area);
                    frame.render_widget(Paragraph::new("Back").centered().style(selected), center)
                }
                LobbySection::WithHero => frame.render_widget(
                    Paragraph::new(if self.select_hero {
                        "Start with Heroes"
                    } else {
                        "Start without Heroes"
                    })
                    .centered()
                    .style(selected),
                    *sect_area,
                ),
                LobbySection::Fog => frame.render_widget(
                    Paragraph::new(match self.select_fog_base {
                        FogTile::Visible => "The map is explored and visible",
                        FogTile::Explored => "The map is explored (terrain always visible)",
                        FogTile::Hidden => "The map is hidden (terrain hidden)",
                    })
                    .centered()
                    .style(selected),
                    *sect_area,
                ),
            }

            if let Some(select_map) = &self.select_map {
                frame.render_widget(Clear::default(), rest);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    rest,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(rest);

                let [list, map] = Layout::horizontal([Max(30), Fill(1)]).areas(center);
                let [topbar, list] = Layout::vertical([Length(1), Fill(1)]).areas(list);
                frame.render_widget(Paragraph::new("Select Map:"), topbar);

                let rows = select_map.choices.iter().enumerate().map(|(y, path)| {
                    let style = if y as i32 == select_map.cursor {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    let name = path.split("/").last().unwrap().trim_end_matches(".txt");
                    Row::new(vec![Cell::new(Line::from(name).style(style))])
                });

                let mut state = TableState::new().with_selected(select_map.cursor as usize);
                frame.render_stateful_widget(Table::new(rows, [30]), list, &mut state);

                let chosen_map = &select_map.maps[select_map.cursor as usize];

                frame.render_widget(
                    BoardWidget {
                        board: chosen_map,
                        blueprints: bp,
                        cursor: chosen_map.grid.size / 2,
                        attack_tiles: &vec![],
                        movement_tiles: &vec![],
                        target_tiles: &vec![],
                        only_player_color: false,
                        zoom: 1,
                        show_spawns: true,
                        travel_path: &vec![],
                        fog_player: &PlayerId::new(0),
                    },
                    map,
                );
            }

            if let Some(select_team) = &self.select_team {
                let [_, center, _] = Layout::horizontal([Fill(1), Max(40), Fill(1)]).areas(rest);
                let [_, center, _] = Layout::vertical([Fill(1), Max(20), Fill(1)]).areas(center);
                frame.render_widget(Clear::default(), center);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    center,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(center);
                let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
                frame.render_widget(
                    Paragraph::new("Select Team").alignment(Alignment::Center),
                    topbar,
                );
                let rows = select_team.choices.iter().map(|opt_team| {
                    let s = match opt_team {
                        Some(team) => format!("Team {}", team.get()),
                        None => format!("Alone, all others are hostile"),
                    };
                    Row::new(vec![Cell::new(Line::from(s))])
                });
                let mut state = TableState::new().with_selected(select_team.cursor as usize);
                frame.render_stateful_widget(
                    Table::new(rows, [30])
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                    rest,
                    &mut state,
                );
            }

            if let Some(select_name) = &self.select_name {
                let [_, center, _] = Layout::horizontal([Fill(1), Max(60), Fill(1)]).areas(rest);
                let [_, center, _] = Layout::vertical([Fill(1), Max(5), Fill(1)]).areas(center);
                frame.render_widget(Clear::default(), center);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    center,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(center);
                let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
                frame.render_widget(
                    Paragraph::new("Write player's name (max 20 characters): ")
                        .alignment(Alignment::Center),
                    topbar,
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::from(select_name.clone()),
                        Span::from("|").style(Style::default().add_modifier(Modifier::SLOW_BLINK)),
                    ])),
                    rest,
                );
            }

            if let Some(select_symbol) = &self.select_symbol {
                let [_, center, _] = Layout::horizontal([Fill(1), Max(60), Fill(1)]).areas(rest);
                let [_, center, _] = Layout::vertical([Fill(1), Max(5), Fill(1)]).areas(center);
                frame.render_widget(Clear::default(), center);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    center,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(center);
                let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
                frame.render_widget(
                    Paragraph::new("Select the player symbol (one character):")
                        .alignment(Alignment::Center),
                    topbar,
                );
                frame.render_widget(Paragraph::new(select_symbol.as_str()).centered(), rest);
            }

            if let Some(select_color) = &self.select_color {
                let [_, center, _] = Layout::horizontal([Fill(1), Max(60), Fill(1)]).areas(rest);
                let [_, center, _] = Layout::vertical([Fill(1), Min(17), Fill(1)]).areas(center);
                frame.render_widget(Clear::default(), center);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    center,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(center);
                let [topbar, _, result_head, result, _, rest] =
                    Layout::vertical([Length(1), Length(1), Length(1), Max(2), Min(1), Length(9)])
                        .areas(center);
                frame.render_widget(
                    Paragraph::new("Select player's color by moving the bars left and right")
                        .alignment(Alignment::Center),
                    topbar,
                );

                let col = select_color.color;
                frame.render_widget(
                    Block::default().bg(Color::Rgb(col[0], col[1], col[2])),
                    result,
                );
                frame.render_widget(Paragraph::new("Player color:"), result_head);

                for (i, area_gauge) in
                    Layout::vertical(select_color.color.iter().map(|_| Length(3)))
                        .split(rest)
                        .iter()
                        .enumerate()
                {
                    let (text, bar_col) = match i {
                        0 => ("Red", Color::Rgb(col[0], 0, 0)),
                        1 => ("Green", Color::Rgb(0, col[1], 0)),
                        2 => ("Blue", Color::Rgb(0, 0, col[2])),
                        _ => ("", Color::Red),
                    };
                    let style = if select_color.cursor == i as i32 {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    frame.render_widget(
                        Gauge::default()
                            .block(Block::default().title(text).title_style(style))
                            .gauge_style(
                                Style::default()
                                    .bg(Color::White)
                                    .fg(bar_col)
                                    .add_modifier(Modifier::ITALIC),
                            )
                            .percent((((col[i] as f32) / 255.0) * 100.0) as u16),
                        *area_gauge,
                    );
                }
            }

            if let Some(select_civilization) = &self.select_civilization {
                let [_, center, _] = Layout::horizontal([Fill(1), Max(40), Fill(1)]).areas(rest);
                let [_, center, _] = Layout::vertical([Fill(1), Max(20), Fill(1)]).areas(center);
                frame.render_widget(Clear::default(), center);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    center,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(center);
                let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
                frame.render_widget(
                    Paragraph::new("Select Civilization").alignment(Alignment::Center),
                    topbar,
                );
                let rows = select_civilization.choices.iter().map(|id| {
                    Row::new(vec![Cell::new(Line::from(
                        bp.get_civilization(id).name.clone(),
                    ))])
                });
                let mut state =
                    TableState::new().with_selected(select_civilization.cursor as usize);
                frame.render_stateful_widget(
                    Table::new(rows, [30])
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                    rest,
                    &mut state,
                );
            }

            if let Some(select_controller) = &self.select_controller {
                let [_, center, _] = Layout::horizontal([Fill(1), Max(40), Fill(1)]).areas(rest);
                let [_, center, _] = Layout::vertical([Fill(1), Max(20), Fill(1)]).areas(center);
                frame.render_widget(Clear::default(), center);
                frame.render_widget(
                    Block::bordered().border_type(BorderType::QuadrantOutside),
                    center,
                );
                let [center] = Layout::default()
                    .constraints([Fill(1)])
                    .margin(1)
                    .areas(center);
                let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
                frame.render_widget(
                    Paragraph::new("Select Civilization").alignment(Alignment::Center),
                    topbar,
                );
                let rows = select_controller.choices.iter().map(|controller| {
                    Row::new(vec![Cell::new(Line::from(match controller {
                        Controller::Human => format!("Local"),
                        Controller::Remote(id) => format!("Remote({})", id),
                        Controller::Machine(MachineOpponent::WeakBoulder) => {
                            format!("Machine Boulder Weak")
                        }
                        Controller::Machine(MachineOpponent::AverageBoulder) => {
                            format!("Machine Boulder Average")
                        }
                        Controller::Machine(MachineOpponent::StrongBoulder) => {
                            format!("Machine Boulder Strong")
                        }
                        Controller::Machine(MachineOpponent::WeakPeak) => {
                            format!("Machine Peak Weak")
                        }
                        Controller::Machine(MachineOpponent::AveragePeak) => {
                            format!("Machine Peak Average")
                        }
                        Controller::Machine(MachineOpponent::StrongPeak) => {
                            format!("Machine Peak Strong")
                        }
                        Controller::Machine(MachineOpponent::Boulder(boulder)) => {
                            format!("Machine(Boulder({:?}))", boulder)
                        }
                        Controller::Machine(MachineOpponent::Peak(peak)) => {
                            format!("Machine(Peak({:?}))", peak)
                        }
                    }))])
                });
                let mut state = TableState::new().with_selected(select_controller.cursor as usize);
                frame.render_stateful_widget(
                    Table::new(rows, [30])
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                    rest,
                    &mut state,
                );
            }
        }

        if let Some(error) = net.error.get(0) {
            frame.render_widget(Clear, area);
            let [_, center, _] = Layout::vertical([Fill(1), Max(10), Fill(1)]).areas(rest);
            let [_, center, _] = Layout::horizontal([Fill(1), Max(70), Fill(1)]).areas(center);
            let [inner] = Layout::default()
                .constraints([Min(0)])
                .margin(1)
                .areas(center);
            let [head, inner] = Layout::vertical([Length(1), Fill(1)]).areas(inner);
            frame.render_widget(
                Block::bordered().border_type(BorderType::QuadrantOutside),
                center,
            );
            frame.render_widget(Paragraph::new("Error"), head);
            frame.render_widget(Paragraph::new(error.as_str()), inner);
        }
    }
}

impl LobbySelectColor {
    pub fn input(&self, input: MenuInput) -> LobbySelectColor {
        let mut next = self.clone();
        next.cursor = (self.cursor + input.acc.y).clamp(0, self.color.len() as i32 - 1);
        next.color[self.cursor as usize] =
            (self.color[self.cursor as usize] as i32 + input.acc.x).clamp(0, 255) as u8;
        next
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        use Constraint::*;
        let [_, center, _] = Layout::horizontal([Fill(1), Max(60), Fill(1)]).areas(area);
        let [_, center, _] = Layout::vertical([Fill(1), Min(17), Fill(1)]).areas(center);
        frame.render_widget(Clear::default(), center);
        frame.render_widget(
            Block::bordered().border_type(BorderType::QuadrantOutside),
            center,
        );
        let [center] = Layout::default()
            .constraints([Fill(1)])
            .margin(1)
            .areas(center);
        let [topbar, _, result_head, result, _, rest] =
            Layout::vertical([Length(1), Length(1), Length(1), Max(2), Min(1), Length(9)])
                .areas(center);
        frame.render_widget(
            Paragraph::new("Select player's color by moving the bars left and right")
                .alignment(Alignment::Center),
            topbar,
        );

        let col = self.color;
        frame.render_widget(
            Block::default().bg(Color::Rgb(col[0], col[1], col[2])),
            result,
        );
        frame.render_widget(Paragraph::new("Player color:"), result_head);

        for (i, area_gauge) in Layout::vertical(self.color.iter().map(|_| Length(3)))
            .split(rest)
            .iter()
            .enumerate()
        {
            let (text, bar_col) = match i {
                0 => ("Red", Color::Rgb(col[0], 0, 0)),
                1 => ("Green", Color::Rgb(0, col[1], 0)),
                2 => ("Blue", Color::Rgb(0, 0, col[2])),
                _ => ("", Color::Red),
            };
            let style = if self.cursor == i as i32 {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            frame.render_widget(
                Gauge::default()
                    .block(Block::default().title(text).title_style(style))
                    .gauge_style(
                        Style::default()
                            .bg(Color::White)
                            .fg(bar_col)
                            .add_modifier(Modifier::ITALIC),
                    )
                    .percent((((col[i] as f32) / 255.0) * 100.0) as u16),
                *area_gauge,
            );
        }
    }
}
