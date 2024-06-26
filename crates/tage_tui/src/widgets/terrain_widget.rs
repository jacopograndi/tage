use ratatui::{prelude::*, widgets::*};
use tage_core::prelude::*;

use crate::{resource_color, terrain_color};

#[derive(Debug, Clone, Copy)]
pub struct TerrainWidget<'a> {
    pub blueprints: &'a Blueprints,
    pub terrain: &'a TerrainTile,
}

impl<'a> TerrainWidget<'a> {
    pub const AREA: IVec2 = IVec2 { x: 27, y: 7 };
    pub const RECT: Rect = Rect {
        x: 0,
        y: 0,
        width: Self::AREA.x as u16,
        height: Self::AREA.y as u16,
    };
}

impl<'a> Widget for TerrainWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;

        Clear::default().render(area, buf);
        Block::bordered()
            .border_type(BorderType::Plain)
            .render(area, buf);
        let [area] = Layout::default()
            .constraints([Min(0)])
            .margin(1)
            .areas(area);

        let bp = self.blueprints.get_terrain(&self.terrain.blueprint_id);

        let [header, resources, collectable, stats] =
            Layout::vertical([Length(1), Length(1), Length(1), Length(2)]).areas(area);
        let [glyph, _, name] = Layout::horizontal([Length(3), Length(1), Min(0)]).areas(header);
        let [resources, road] = Layout::horizontal([Length(14), Fill(1)]).areas(resources);

        Paragraph::new(format!("{}", bp.header.glyph))
            .style(Style::default().bg(terrain_color(bp)).fg(Color::Black))
            .render(glyph, buf);
        Paragraph::new(format!("{}", bp.header.name)).render(name, buf);

        if let Some(res) = &self.terrain.resource {
            Paragraph::new(format!("Resource: {}", res.view()))
                .style(Style::default().bg(resource_color(res)).fg(Color::Black))
                .render(resources, buf);
        }
        if self.terrain.has_road {
            Paragraph::new(format!("Road ="))
                .alignment(Alignment::Right)
                .render(road, buf);
        }

        if let Some(coll) = &self.terrain.collectable {
            let text = match coll {
                Collectable::BonusFood => format!("Supplies (100 food)"),
                Collectable::BonusGold => format!("Treasure (100 gold)"),
                Collectable::Ruins => format!("Ruins"),
                Collectable::Relic => format!("Relic"),
            };
            Paragraph::new(text).render(collectable, buf);
        }

        let rows: Vec<Row> = vec![
            Row::new(vec![
                Line::from("Move"),
                Line::from("Sight"),
                Line::from("Range"),
                Line::from("Defence"),
                Line::from("View Bonus"),
            ]),
            Row::new(vec![
                Line::from(format!(
                    "{}",
                    if self.terrain.has_road {
                        1
                    } else {
                        bp.stats.move_cost
                    }
                )),
                Line::from(format!("{}", bp.stats.sight_cost)),
                Line::from(format!("{}", bp.stats.range_bonus)),
                Line::from(format!("{}%", bp.stats.defence_bonus)),
                Line::from(format!("{}", bp.stats.sight_bonus)),
            ]),
        ];
        let widths: Vec<u16> = (0..5).map(|_| 7).collect();
        let table = Table::new(rows, widths);
        Widget::render(table, stats, buf);
    }
}
