use ratatui::{prelude::*, widgets::*};
use tage_core::prelude::*;

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct UnitWidget<'a> {
    pub board: &'a Board,
    pub blueprints: &'a Blueprints,
    pub unit: &'a Unit,
    pub bonus: &'a Bonus,
}

impl<'a> UnitWidget<'a> {
    pub const AREA: IVec2 = IVec2 { x: 28, y: 7 };
    pub const RECT: Rect = Rect {
        x: 0,
        y: 0,
        width: UnitWidget::AREA.x as u16,
        height: UnitWidget::AREA.y as u16,
    };
}

impl<'a> Widget for UnitWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;

        Clear::default().render(area, buf);
        Block::bordered().render(area, buf);
        let [area] = Layout::default()
            .constraints([Min(0)])
            .margin(1)
            .areas(area);

        let style = Style::default().fg(Color::White);
        let owner = self.board.get_player(&self.unit.owner);
        let bp = self.blueprints.get_unit(&self.unit.blueprint_id);

        let [header, hp_class, details, body] =
            Layout::vertical([Length(1), Length(1), Length(1), Min(2)]).areas(area);
        let [glyph, _, name] = Layout::horizontal([Length(3), Length(1), Min(0)]).areas(header);
        let [health, _, class] =
            Layout::horizontal([Length(11), Fill(1), Length(8)]).areas(hp_class);
        let [veterancy, special] = Layout::horizontal([Min(14), Fill(1)]).areas(details);

        Paragraph::new(bp.header.glyph.as_str())
            .style(style.fg(Color::from_u32(owner.color)))
            .render(glyph, buf);
        Paragraph::new(bp.header.name.as_str())
            .style(style)
            .render(name, buf);
        Paragraph::new(format!("Health: {}", self.unit.health))
            .style(style.fg(match self.unit.health {
                0..=33 => Color::Rgb(140, 20, 0),
                34..=66 => Color::Rgb(200, 120, 0),
                67..=100 => Color::Rgb(29, 150, 40),
                _ => Color::Blue,
            }))
            .render(health, buf);
        Paragraph::new(bp.header.class.view())
            .style(style)
            .alignment(Alignment::Right)
            .render(class, buf);

        Paragraph::new(format!("Vetarancy: {}", self.unit.veterancy))
            .style(style)
            .render(veterancy, buf);

        UnitStatsWidget {
            stats: &bp.stats,
            bonus: &Some(self.bonus),
            alignment: Alignment::Center,
        }
        .render(body, buf);

        if self.unit.in_construction {
            Paragraph::new(format!("Making..."))
                .style(style)
                .alignment(Alignment::Right)
                .render(special, buf);
        } else if Some(Collectable::Relic) == self.unit.holding_collectable {
            Paragraph::new(format!("Holding Relic"))
                .style(style)
                .render(special, buf)
        }
    }
}
