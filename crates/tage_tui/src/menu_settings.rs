use ratatui::{prelude::*, widgets::*};

use crate::*;

#[derive(Debug, Clone)]
pub struct MenuSettings {
    menu_keybinds: MenuSettingsKeybinds,
    menu_gameplay: MenuSettingsGameplay,
    cursor: i32,
    tab: i32,
}

impl MenuSettings {
    pub fn new(ui_state: &InterfaceState) -> Self {
        Self {
            menu_keybinds: MenuSettingsKeybinds::new(&ui_state.settings.keybinds),
            menu_gameplay: MenuSettingsGameplay::new(),
            cursor: 0,
            tab: 0,
        }
    }

    pub fn input(mut self, input: MenuInput, ui_state: &mut InterfaceState) -> Option<Self> {
        match self.cursor {
            0 => {
                if input.select {
                    return None;
                } else {
                    self.cursor = (self.cursor + input.acc.y).clamp(0, 1);
                    self.tab = (self.tab + input.acc.x).clamp(0, 1);
                    if self.cursor > 0 {
                        match self.tab {
                            0 => self.menu_gameplay.cursor = 0,
                            1 => self.menu_keybinds.cursor = 0,
                            _ => {}
                        }
                    }
                }
            }
            1 => {
                let sub_cursor = match self.tab {
                    0 => {
                        self.menu_gameplay.input(input.clone(), ui_state);
                        self.menu_gameplay.cursor
                    }
                    1 => {
                        self.menu_keybinds.input(input.clone(), ui_state);
                        self.menu_keybinds.cursor
                    }
                    _ => -1,
                };
                if sub_cursor < 0 {
                    self.cursor = (self.cursor + input.acc.y).clamp(0, 1);
                }
            }
            _ => {}
        }
        Some(self)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, settings: &Settings) {
        use Constraint::*;

        frame.render_widget(Clear::default(), area);

        let [topbar, _, tabs_area, _, rest] =
            Layout::vertical([Length(1), Length(1), Length(1), Length(1), Fill(1)]).areas(area);

        frame.render_widget(
            Paragraph::new("Settings").alignment(Alignment::Center),
            topbar,
        );

        let [d0, rest, d1] = Layout::horizontal([Fill(1), Max(60), Fill(1)]).areas(rest);

        frame.render_widget(PanelWidget::new(DECOR_0), d0);
        frame.render_widget(PanelWidget::new(DECOR_0), d1);

        let [back, _, rest] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(rest);

        let tabs = ["Gameplay", "Keybinds"];
        frame.render_widget(
            Table::new(
                [Row::new(tabs.iter().enumerate().map(|(i, t)| {
                    let style = if i == self.tab as usize {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    Cell::new(Line::from(*t)).style(style)
                }))],
                tabs.iter().map(|_| Fill(1)),
            ),
            tabs_area,
        );

        frame.render_widget(
            ButtonAt {
                string: "Back",
                index: 0,
                cursor: self.cursor,
            },
            back,
        );

        match self.tab {
            0 => self.menu_gameplay.render(frame, rest, settings),
            1 => self.menu_keybinds.render(frame, rest, settings),
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuSettingsGameplay {
    cursor: i32,
}

impl MenuSettingsGameplay {
    pub fn new() -> Self {
        Self { cursor: -1 }
    }

    pub fn input(&mut self, input: MenuInput, ui_state: &mut InterfaceState) {
        self.cursor = (self.cursor + input.acc.y).clamp(-1, 0);
        if input.select {
            match self.cursor {
                0 => {
                    let speed = match ui_state.settings.machine_speed {
                        MachineSpeed::Skip => MachineSpeed::StepMoves,
                        MachineSpeed::StepMoves => MachineSpeed::StepMovesSlow,
                        MachineSpeed::StepMovesSlow => MachineSpeed::Skip,
                        MachineSpeed::StepSelects => todo!(),
                    };
                    ui_state.settings.machine_speed = speed;
                    ui_state.settings.to_disk();
                }
                _ => {}
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, settings: &Settings) {
        use Constraint::*;

        frame.render_widget(Clear::default(), area);

        let [topbar, _, rest] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(area);
        frame.render_widget(
            Paragraph::new("Settings").alignment(Alignment::Center),
            topbar,
        );

        let mut state = TableState::new().with_selected(
            (0..=1)
                .contains(&self.cursor)
                .then_some(self.cursor as usize),
        );
        frame.render_stateful_widget(
            Table::new(
                [Row::new([
                    Cell::from("Machine action speed"),
                    Cell::from(match settings.machine_speed {
                        MachineSpeed::Skip => format!("Skip"),
                        MachineSpeed::StepMoves => format!("Step"),
                        MachineSpeed::StepMovesSlow => format!("Step slowly"),
                        MachineSpeed::StepSelects => format!("Step Detailed"),
                    }),
                ])],
                [Fill(1), Fill(1)],
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            rest,
            &mut state,
        );
    }
}

#[derive(Debug, Clone)]
pub struct MenuSettingsKeybinds {
    choices: Vec<String>,
    cursor: i32,
    cursor_keybinds: i32,
    modifying_keybinds: bool,
    dirty_keybinds: bool,

    /// local state, allows editing and then applying the whole state
    keybinds: Keybinds,
}

impl MenuSettingsKeybinds {
    pub fn new(keybinds: &Keybinds) -> Self {
        Self {
            cursor: -1,
            choices: Keybinds::fields_to_string(),
            modifying_keybinds: false,
            cursor_keybinds: 0,
            dirty_keybinds: false,
            keybinds: keybinds.clone(),
        }
    }

    pub fn input(&mut self, input: MenuInput, ui_state: &mut InterfaceState) {
        if self.modifying_keybinds {
            if let Some(keycode) = &input.keycode {
                *self
                    .keybinds
                    .get_key_from_string(self.choices[self.cursor_keybinds as usize].as_str()) =
                    keycode.clone();
                self.dirty_keybinds = true;
                self.modifying_keybinds = false;
            }
        } else {
            if self.cursor == 1 {
                self.cursor_keybinds = self.cursor_keybinds + input.acc.y;
                if self.cursor_keybinds < 0 {
                    self.cursor_keybinds = 0;
                    self.cursor = (self.cursor - 1).clamp(-1, 3);
                } else if self.cursor_keybinds >= self.choices.len() as i32 {
                    self.cursor_keybinds = self.choices.len() as i32 - 1;
                    self.cursor = (self.cursor + 1).clamp(-1, 3);
                }
            } else {
                self.cursor = (self.cursor + input.acc.y).clamp(-1, 3);
            }
        }

        if input.select {
            match self.cursor {
                0 => {
                    self.dirty_keybinds = false;
                    ui_state.settings.keybinds = self.keybinds.clone();
                    ui_state.settings.to_disk();
                }
                1 => {
                    self.modifying_keybinds = true;
                }
                2 => {
                    self.dirty_keybinds = false;
                    self.keybinds = Keybinds::default();
                    ui_state.settings.keybinds = self.keybinds.clone();
                    ui_state.settings.to_disk();
                }
                3 => {
                    self.dirty_keybinds = false;
                    self.keybinds = Keybinds::vim();
                    ui_state.settings.keybinds = self.keybinds.clone();
                    ui_state.settings.to_disk();
                }
                _ => {}
            }
        }

        if input.back && !self.modifying_keybinds {
            self.cursor = -1
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, _settings: &Settings) {
        use Constraint::*;

        frame.render_widget(Clear::default(), area);

        let [apply, _, head, rest, description, _, reset, vim] = Layout::vertical([
            Length(1),
            Length(1),
            Length(1),
            Fill(1),
            Length(3),
            Length(1),
            Length(1),
            Length(1),
        ])
        .areas(area);

        frame.render_widget(
            ButtonAt {
                string: if self.dirty_keybinds {
                    "Apply changes (Unapplied changes pending)"
                } else {
                    "Apply changes"
                },
                index: 0,
                cursor: self.cursor,
            },
            apply,
        );

        frame.render_widget(
            Paragraph::new("Modify key bindings:").alignment(Alignment::Center),
            head,
        );

        let selected = if self.cursor == 1 {
            if self.modifying_keybinds {
                Style::default().add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().add_modifier(Modifier::REVERSED)
            }
        } else {
            Style::default()
        };
        let mut state = TableState::new().with_selected(self.cursor_keybinds as usize);
        frame.render_stateful_widget(
            Table::new(
                self.choices.iter().map(|c| {
                    let name = self.keybinds.clone().get_name_from_field(c.as_str());
                    let key = self.keybinds.clone().get_key_from_string(c).clone();
                    Row::new(vec![
                        Cell::new(Line::from(name)),
                        Cell::new(Line::from(format!("{:?}", key))),
                    ])
                }),
                [Fill(2), Fill(1)],
            )
            .highlight_style(selected),
            rest,
            &mut state,
        );

        if self.cursor == 1 {
            if let Some(text) = Keybinds::fields_to_description()
                .get(self.choices[self.cursor_keybinds as usize].as_str())
            {
                frame.render_widget(
                    Paragraph::new(format!("Keybind description: {}", text))
                        .wrap(Wrap { trim: true }),
                    description,
                );
            }
        }

        frame.render_widget(
            ButtonAt {
                string: "Set keybinds to wasd mode",
                index: 2,
                cursor: self.cursor,
            },
            reset,
        );

        frame.render_widget(
            ButtonAt {
                string: "Set keybinds to vim mode",
                index: 3,
                cursor: self.cursor,
            },
            vim,
        );
    }
}
