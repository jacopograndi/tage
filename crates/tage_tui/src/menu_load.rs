use std::collections::HashMap;

use tage_core::game::Board;

use crate::*;

#[derive(Debug, Clone)]
pub struct MenuLoad {
    choices: Vec<String>,
    cursor: i32,
    boards: HashMap<String, Board>,
    pub loaded: bool,
}

impl MenuLoad {
    pub fn new() -> MenuLoad {
        let mut choices = if let Some(path) = get_data_dir_sub("saves") {
            if let Ok(paths) = fs::read_dir(path) {
                paths
                    .map(|path| path.unwrap().file_name().to_str().unwrap().to_string())
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        choices.sort();
        MenuLoad {
            choices,
            cursor: 0,
            boards: HashMap::new(),
            loaded: false,
        }
    }

    pub fn input(
        mut self,
        input: MenuInput,
        bp: &Blueprints,
        game_state: &mut Option<GameState>,
    ) -> Option<Self> {
        self.cursor = (self.cursor + input.acc.y).clamp(0, self.choices.len() as i32);
        if self.cursor > 0 {
            let mut path = get_data_dir_sub("saves").unwrap();
            path.push(&self.choices[(self.cursor - 1).max(0) as usize]);
            let path = path.to_str().unwrap();
            if !self.boards.contains_key(path) {
                if let Ok(board) = Board::load(bp, path) {
                    self.boards.insert(path.to_string(), board);
                }
            }
        }
        if input.back {
            return None;
        }
        if input.select {
            if self.cursor > 0 {
                let mut path = get_data_dir_sub("saves").unwrap();
                path.push(&self.choices[(self.cursor - 1).max(0) as usize]);
                if let Some(board) = self.boards.get(path.to_str().unwrap()) {
                    *game_state = Some(GameState {
                        board: board.clone(),
                        blueprints: bp.clone(),
                        navigator: None,
                        turn_timeline: vec![],
                        current_picker: UiPicker::Tile(UiTilePicker {
                            cursor: IVec2::ZERO,
                            unit: None,
                            valid_tiles: vec![],
                            selection_type: None,
                        }),
                    });
                    self.loaded = true;
                }
                Some(self)
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, bp: &Blueprints) {
        use Constraint::*;

        frame.render_widget(Clear::default(), area);

        let [topbar, rest] = Layout::vertical([Length(1), Fill(1)]).areas(area);
        frame.render_widget(
            Paragraph::new("Load a saved game:").alignment(Alignment::Center),
            topbar,
        );

        let [list, map] = Layout::horizontal([Max(50), Fill(1)]).areas(rest);
        let [head, _, list] = Layout::vertical([Length(1), Length(1), Fill(1)]).areas(list);

        let list = bordered(frame, list);

        frame.render_widget(
            Paragraph::new("Back")
                .alignment(Alignment::Center)
                .style(if self.cursor == 0 {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                }),
            head,
        );

        let selected = if self.cursor > 0 {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let mut state = TableState::new().with_selected((self.cursor - 1).max(0) as usize);
        frame.render_stateful_widget(
            Table::new(
                self.choices.iter().map(|c| {
                    Row::new(vec![Cell::new(
                        Line::from(c.clone()).alignment(Alignment::Left),
                    )])
                }),
                [40],
            )
            .highlight_style(selected),
            list,
            &mut state,
        );

        if self.cursor > 0 {
            let mut path = get_data_dir_sub("saves").unwrap();
            path.push(&self.choices[(self.cursor - 1).max(0) as usize]);
            if let Some(board) = self.boards.get(path.to_str().unwrap()) {
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
            } else {
                frame.render_widget(PanelWidget::new(DECOR_1), map);
                let inner = popup(frame, map, Size::new(30, 5));
                frame.render_widget(Paragraph::new("Failed to load."), inner);
            }
        } else {
            frame.render_widget(PanelWidget::new(DECOR_3), map)
        }
    }
}
