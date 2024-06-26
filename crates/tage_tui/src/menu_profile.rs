use crate::*;

#[derive(Debug, Clone)]
pub struct MenuProfile {
    choices: Vec<String>,
    cursor: i32,
    editing_name: bool,
    editing_symbol: bool,
    select_color: Option<LobbySelectColor>,
}

impl MenuProfile {
    pub fn new() -> Self {
        Self {
            choices: ["Back", "Name", "Symbol", "Color", "Done"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            cursor: 0,
            editing_name: false,
            editing_symbol: false,
            select_color: None,
        }
    }

    pub fn input(&self, input: MenuInput, ui_state: &mut InterfaceState) -> MenuState {
        let mut input = input;
        let mut next = self.clone();
        if self.editing_name {
            match input.keycode {
                Some(KeyCode::Enter) => {
                    ui_state.member_profile.to_disk();
                    next.editing_name = false;
                    input.select = false;
                }
                Some(KeyCode::Backspace) => {
                    ui_state.member_profile.name.pop();
                }
                Some(KeyCode::Char(c)) => {
                    if ALLOWED_NAME_CHARS.contains(&c) && ui_state.member_profile.name.len() < 32 {
                        ui_state.member_profile.name.push(c);
                    }
                }
                _ => {}
            }
        } else if self.editing_symbol {
            match input.keycode {
                Some(KeyCode::Enter) => {
                    ui_state.member_profile.to_disk();
                    next.editing_symbol = false;
                    input.select = false;
                }
                Some(KeyCode::Char(c)) => {
                    if ALLOWED_NAME_CHARS.contains(&c) {
                        ui_state.member_profile.symbol = c.to_string();
                        next.editing_symbol = false;
                        input.select = false;
                    }
                }
                _ => {}
            }
        } else if let Some(select_color) = &self.select_color {
            next.select_color = Some(select_color.input(input.clone()))
        } else {
            next.cursor = (self.cursor + input.acc.y).clamp(0, self.choices.len() as i32 - 1);
        }

        if input.select {
            match self.cursor {
                0 => MenuState::Home(MenuHome::new()),
                1 => {
                    next.editing_name = true;
                    MenuState::Profile(next)
                }
                2 => {
                    next.editing_symbol = true;
                    MenuState::Profile(next)
                }
                3 => {
                    if let Some(select_color) = next.select_color.take() {
                        ui_state.member_profile.color = select_color.color;
                        ui_state.member_profile.to_disk();
                    } else {
                        next.select_color = Some(LobbySelectColor {
                            color: ui_state.member_profile.color,
                            cursor: 0,
                        });
                    }
                    MenuState::Profile(next)
                }
                4 => MenuState::Connect(MenuConnect::new()),
                _ => unreachable!(),
            }
        } else {
            MenuState::Profile(next)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, member: &Member) {
        use Constraint::*;

        frame.render_widget(PanelWidget::new(DECOR_2), area);

        let center = popup(frame, area, Size::new(44, 10));

        let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
        frame.render_widget(
            Paragraph::new("Profile").alignment(Alignment::Center),
            topbar,
        );

        let [header, _, body, _, done] =
            Layout::vertical([Length(1), Fill(1), Length(3), Fill(1), Length(1)]).areas(rest);

        frame.render_widget(
            ButtonAt::new(self.choices[0].as_str(), 0, self.cursor),
            header,
        );

        let profile = [
            Cell::new(Line::from(member.name.clone())),
            Cell::new(Line::from(member.symbol.clone())),
            Cell::new(Line::from("  ").style(Style::default().bg(Color::Rgb(
                member.color[0],
                member.color[1],
                member.color[2],
            )))),
        ];

        let mut body_state = TableState::new().with_selected(
            (1..4)
                .contains(&self.cursor)
                .then_some((self.cursor - 1) as usize),
        );
        frame.render_stateful_widget(
            Table::new(
                self.choices
                    .iter()
                    .skip(1)
                    .zip(profile.into_iter())
                    .map(|(c, p)| {
                        Row::new(vec![
                            Cell::new(Line::from(c.clone()).alignment(Alignment::Left)),
                            p,
                        ])
                    }),
                [8, 34],
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            body,
            &mut body_state,
        );

        frame.render_widget(
            ButtonAt::new(self.choices[4].as_str(), 4, self.cursor),
            done,
        );

        if let Some(select_color) = &self.select_color {
            select_color.render(frame, area)
        }
    }
}
