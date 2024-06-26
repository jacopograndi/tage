use crate::*;

pub static TITLE: &'static str = r#"
  d8                                        
  88                                        
MM88MMM ,adPPYYba,  ,adPPYb,d8  ,adPPYba,   
  88    ""     `Y8 a8"    `Y88 a8P_____88   
  88    ,adPPPPP88 8b       88 8PP"""""""   
  88,   88,    ,88 "8a,   ,d88 "8b,   ,aa   
  "Y888 `"8bbdP"Y8  `"YbbdP"Y8  `"Ybbd8"'   
                    aa,    ,88              
                     "Y8bbdP"               
"#;

pub static ALLOWED_NAME_CHARS: [char; 88] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '.', '-', '@', '%', '*', '!', '?', '|', ':', ';', ',', '<', '>', '(',
    ')', '[', ']', '{', '}', '+', '#', '$', '^', '&', '/', '\\',
];

pub static ALLOWED_PATH_CHARS: [char; 82] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '.', '-', '@', '%', '*', '!', '?', '|', ';', ',', '(', ')', '[', ']',
    '{', '}', '+', '#', '^', '&',
];

pub static ALLOWED_IP_CHARS: [char; 12] =
    ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ':'];

#[derive(Debug, Clone)]
pub enum MenuState {
    Home(MenuHome),
    Lobby(MenuLobby),
    Profile(MenuProfile),
    Connect(MenuConnect),
    Settings(MenuSettings),
    Load(MenuLoad),
    Close,
    Play(MapSettings),
}

impl MenuState {
    pub fn input(
        self,
        input: MenuInput,
        is_running: &mut bool,
        interface_state: &mut InterfaceState,
        game_state: &mut Option<GameState>,
        bp: &Blueprints,
        net: &mut Net,
    ) -> Option<Self> {
        match self {
            MenuState::Home(home) => Some(home.input(input, interface_state, net)),
            MenuState::Lobby(lobby) => Some(lobby.input(input, bp, net)),
            MenuState::Profile(profile) => Some(profile.input(input, interface_state)),
            MenuState::Connect(connect) => Some(connect.input(input, interface_state, net)),
            MenuState::Settings(settings) => {
                if let Some(settings) = settings.input(input, interface_state) {
                    Some(MenuState::Settings(settings))
                } else {
                    Some(MenuState::Home(MenuHome::new()))
                }
            }
            MenuState::Close => {
                *is_running = false;
                Some(self.clone())
            }
            MenuState::Load(load) => {
                if let Some(load) = load.input(input, bp, game_state) {
                    if load.loaded {
                        None
                    } else {
                        Some(MenuState::Load(load))
                    }
                } else {
                    Some(MenuState::Home(MenuHome::new()))
                }
            }
            MenuState::Play(map_settings) => {
                let _ = game_state.insert(setup_gamestate(map_settings.clone(), bp).unwrap());
                interface_state.main_menu = None;
                None
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, bp: &Blueprints, net: &Net, ui_state: &InterfaceState) {
        match self {
            MenuState::Home(home) => home.render(frame, frame.size(), ui_state),
            MenuState::Lobby(lobby) => lobby.render(frame, frame.size(), bp, net),
            MenuState::Profile(profile) => {
                profile.render(frame, frame.size(), &ui_state.member_profile)
            }
            MenuState::Connect(connect) => {
                connect.render(frame, frame.size(), &ui_state.member_profile)
            }
            MenuState::Settings(settings_menu) => {
                settings_menu.render(frame, frame.size(), &ui_state.settings)
            }
            MenuState::Load(load) => load.render(frame, frame.size(), bp),
            MenuState::Close => {}
            MenuState::Play(_) => {}
        };
    }
}

#[derive(Debug, Clone)]
pub struct MenuHome {
    choices: Vec<String>,
    cursor: i32,
}

impl MenuHome {
    pub fn new() -> Self {
        Self {
            choices: [
                "Singleplayer",
                "Multiplayer",
                "Load Save",
                "Settings",
                "Close",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            cursor: 0,
        }
    }

    pub fn input(&self, input: MenuInput, ui_state: &mut InterfaceState, net: &Net) -> MenuState {
        let mut next = self.clone();
        next.cursor = (self.cursor + input.acc.y).clamp(0, self.choices.len() as i32 - 1);
        if input.select {
            match self.cursor {
                0 => {
                    if let Some(member) = Member::from_disk() {
                        ui_state.member_profile = member;
                    }
                    MenuState::Lobby(MenuLobby::new(ui_state, net))
                }
                1 => {
                    if let Some(member) = Member::from_disk() {
                        ui_state.member_profile = member;
                        MenuState::Connect(MenuConnect::new())
                    } else {
                        MenuState::Profile(MenuProfile::new())
                    }
                }
                2 => MenuState::Load(MenuLoad::new()),
                3 => MenuState::Settings(MenuSettings::new(ui_state)),
                4 => MenuState::Close,
                _ => unreachable!(),
            }
        } else {
            MenuState::Home(next)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ui_state: &InterfaceState) {
        use Constraint::*;

        render_background(frame, area, ui_state);

        let center = popup(frame, area, Size::new(56, 18));

        let [title, _, rest] = Layout::vertical([Length(10), Length(1), Fill(1)]).areas(center);

        let [_, title, _] = Layout::horizontal([Fill(1), Length(44), Fill(1)]).areas(title);
        frame.render_widget(
            Paragraph::new(TITLE).style(Style::default().fg(Color::White)),
            title,
        );

        let [_, rest, _] = Layout::horizontal([Fill(1), Length(25), Fill(1)]).areas(rest);

        let mut state = TableState::new().with_selected(self.cursor as usize);
        frame.render_stateful_widget(
            Table::new(
                self.choices.iter().map(|c| {
                    Row::new(vec![Cell::new(
                        Line::from(c.clone()).alignment(Alignment::Center),
                    )])
                }),
                [Fill(1)],
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            rest,
            &mut state,
        );
    }
}

pub fn render_background(frame: &mut Frame, area: Rect, ui_state: &InterfaceState) {
    if let Some(ref board) = ui_state.background_board {
        frame.render_widget(
            BoardWidget {
                board,
                blueprints: &board.bp(),
                cursor: board.grid.size / 2,
                movement_tiles: &vec![],
                attack_tiles: &vec![],
                target_tiles: &vec![],
                only_player_color: true,
                zoom: 3,
                show_spawns: false,
                travel_path: &vec![],
                fog_player: &PlayerId::new(0),
            },
            area,
        )
    } else {
        frame.render_widget(Clear::default(), area)
    }
}
