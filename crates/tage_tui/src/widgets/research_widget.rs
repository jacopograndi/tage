use ratatui::{layout::Alignment, prelude::*, widgets::*};
use tage_core::prelude::*;

use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct ResearchWidget<'a> {
    pub board: &'a Board,
    pub blueprints: &'a Blueprints,
    pub tech_picker: &'a UiTechPicker,
}

impl<'a> Widget for ResearchWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;

        Clear::default().render(area, buf);
        Block::bordered()
            .border_type(BorderType::QuadrantOutside)
            .render(area, buf);
        let [area] = Layout::default()
            .constraints([Min(0)])
            .margin(1)
            .areas(area);

        let [header, _, body] = Layout::vertical([Length(1), Length(1), Min(0)]).areas(area);

        Paragraph::new("Research")
            .alignment(Alignment::Center)
            .render(header, buf);

        let [left, right, details] = Layout::horizontal([Fill(1), Fill(1), Min(25)]).areas(body);

        let bp = self
            .blueprints
            .get_tech(&self.tech_picker.layout[self.tech_picker.level][self.tech_picker.cursor]);
        let [details_header, details_body] = Layout::vertical([Length(1), Min(0)]).areas(details);
        Paragraph::new("Description")
            .alignment(Alignment::Left)
            .render(details_header, buf);
        Block::bordered().render(details_body, buf);
        let [details_body] = Layout::default()
            .constraints([Min(0)])
            .margin(1)
            .areas(details_body);
        let [name, _, cost, _, require, other] = Layout::vertical([
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Length(1),
            Fill(1),
        ])
        .areas(details_body);
        Paragraph::new(bp.name.clone()).render(name, buf);
        Paragraph::new(format!(
            "Cost: {} food, {} gold",
            bp.cost.food, bp.cost.gold
        ))
        .render(cost, buf);
        Paragraph::new("Requires: ".to_string() + &bp.require.view(self.blueprints))
            .render(require, buf);
        Paragraph::new(
            bp.unit_bonuses
                .iter()
                .map(|b| b.view(self.blueprints) + "\n")
                .chain(
                    bp.battle_bonuses
                        .iter()
                        .map(|b| b.view(self.blueprints) + "\n"),
                )
                .fold(String::new(), |acc, s| acc + &s)
                .split("\n")
                .map(|s| Line::raw(s))
                .collect::<Vec<Line>>(),
        )
        .wrap(Wrap { trim: false })
        .render(other, buf);

        let clamped_level = self
            .tech_picker
            .level
            .min(self.tech_picker.layout.len() - 2);
        for (level, area) in [(clamped_level, left), (clamped_level + 1, right)] {
            let [list_header, techs] = Layout::vertical([Length(1), Fill(1)]).areas(area);

            let player = self.board.get_player(&self.board.current_player_turn);

            let researched = player
                .get_researched_of_level(level as i32, self.blueprints)
                .count();
            let required = Player::get_age_up_tech_count(level as i32);
            let age_up = if level < 3 {
                format!(" (to age up: {}/{})", researched, required)
            } else {
                format!("")
            };
            Paragraph::new(format!("{}{}", view_level(level as i32), age_up))
                .alignment(Alignment::Left)
                .render(list_header, buf);

            let rows: Vec<Row> = self.tech_picker.layout[level]
                .iter()
                .map(|id| {
                    let available = self.tech_picker.choices.contains(id);
                    let researched = player.researched_technologies.contains(&id);
                    let researching =
                        player.research_queued == Some(QueuedResearch::Tech(id.clone()));
                    let style = if researching {
                        Style::default().fg(Color::Blue)
                    } else if researched {
                        Style::default().fg(Color::Green)
                    } else if available {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    let name_cell =
                        Cell::new(self.blueprints.get_tech(id).name.clone()).style(style);
                    Row::new(vec![name_cell])
                })
                .collect();

            let mut state = TableState::new().with_selected(self.tech_picker.cursor);
            let table = Table::new(rows, [25])
                .highlight_symbol(if level == self.tech_picker.level {
                    "> "
                } else {
                    "  "
                })
                .block(Block::bordered().border_type(BorderType::Plain));
            StatefulWidget::render(table, techs, buf, &mut state);
        }
    }
}
