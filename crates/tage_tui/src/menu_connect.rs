use crate::*;

#[derive(Debug, Clone)]
pub struct MenuConnect {
    choices: Vec<String>,
    cursor: i32,
    addr: String,
    error: Option<String>,
}

impl MenuConnect {
    pub fn new() -> Self {
        Self {
            choices: ["Back", "Host", "Join Address", "Edit Profile"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            cursor: 0,
            addr: format!("127.0.0.1:{}", PORT),
            error: None,
        }
    }

    pub fn input(
        &self,
        input: MenuInput,
        ui_state: &mut InterfaceState,
        net: &mut Net,
    ) -> MenuState {
        let mut next = self.clone();
        next.cursor = (self.cursor + input.acc.y).clamp(0, self.choices.len() as i32 - 1);

        if self.cursor == 2 {
            match input.keycode {
                Some(KeyCode::Backspace) => {
                    next.addr.pop();
                }
                Some(KeyCode::Char(c)) => {
                    if ALLOWED_IP_CHARS.contains(&c) && next.addr.len() < 20 {
                        next.addr.push(c);
                    }
                }
                _ => {}
            }
        }

        if input.select {
            if next.error.is_some() {
                next.error = None;
                return MenuState::Connect(next);
            }

            match self.cursor {
                0 => MenuState::Home(MenuHome::new()),
                1 => {
                    net.open_server(ui_state.member_profile.clone());
                    MenuState::Lobby(MenuLobby::new(ui_state, net))
                }
                2 => match self.addr.parse() {
                    Ok(addr) => {
                        net.open_client(ui_state.member_profile.clone(), addr);
                        MenuState::Lobby(MenuLobby::new(ui_state, net))
                    }
                    Err(e) => {
                        let help = format!(
                            "The ip address is in the form num.num.num.num:port
For example 192.168.1.1:{}
The port used by the game is {}",
                            PORT, PORT
                        );
                        next.error =
                            Some(format!("Ip {}: {}\n\n{}", self.addr, e.to_string(), help));
                        MenuState::Connect(next)
                    }
                },
                3 => MenuState::Profile(MenuProfile::new()),
                _ => unreachable!(),
            }
        } else {
            MenuState::Connect(next)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, member: &Member) {
        use Constraint::*;

        frame.render_widget(PanelWidget::new(DECOR_2), area);

        let center = popup(frame, area, Size::new(44, 10));

        let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(center);
        frame.render_widget(
            Paragraph::new("Online").alignment(Alignment::Center),
            topbar,
        );

        let [center, profile] = Layout::vertical([Fill(1), Length(1)]).areas(rest);

        let mut state = TableState::new().with_selected(
            (0..4)
                .contains(&self.cursor)
                .then_some(self.cursor as usize),
        );
        frame.render_stateful_widget(
            Table::new(
                self.choices
                    .iter()
                    .take(4)
                    .zip(["", "", self.addr.as_str(), ""].into_iter())
                    .map(|(a, b)| {
                        Row::new([Cell::new(Line::from(a.as_str())), Cell::new(Line::from(b))])
                    }),
                [20, 22],
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
            center,
            &mut state,
        );

        frame.render_widget(MemberWidget { member }, profile);

        if let Some(error) = &self.error {
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
