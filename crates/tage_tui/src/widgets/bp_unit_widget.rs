use ratatui::{prelude::*, widgets::*};
use tage_core::prelude::*;

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct UnitBlueprintWidget<'a> {
    pub blueprints: &'a Blueprints,
    pub id: &'a UnitId,
}

impl<'a> UnitBlueprintWidget<'a> {
    pub const WIDTH: u16 = 40;
}

impl<'a> Widget for UnitBlueprintWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let temp_area = IVec2::new(Self::WIDTH as i32, 100);
        let rect = Rect::new(0, 0, temp_area.x as u16, temp_area.y as u16);
        let mut temp_buffer = Buffer::empty(rect);

        Clear::default().render(area, buf);

        let style = Style::default().fg(Color::White);
        let bp = self.blueprints.get_unit(self.id);

        let mut current_line = 0;
        let mut line = |s: String| {
            Paragraph::new(s)
                .style(style)
                .render(Rect::new(0, current_line, 40, 1), &mut temp_buffer);
            current_line += 1;
        };
        line(format!("|{}|", bp.header.name));
        line(format!("Glyph: {}", bp.header.glyph));
        line(format!("Age: {}", view_level(bp.header.level)));
        line(format!(
            "Cost: (food: {}, gold: {})",
            bp.resources.cost.food, bp.resources.cost.gold
        ));
        if bp.resources.produces.food != 0 || bp.resources.produces.gold != 0 {
            line(format!(
                "Produces: (food: {}, gold: {})",
                bp.resources.produces.food, bp.resources.produces.gold
            ));
        }
        if let Some(upgrade) = &bp.upgrades_to {
            line(format!(
                "Upgrades to: {}",
                self.blueprints.get_unit(upgrade.unit()).header.name
            ));
        }
        if !bp.train_list.is_empty() {
            line(format!("Trains:"));
            for train in bp.build_list.iter() {
                line(format!(
                    "  {}",
                    self.blueprints.get_unit(train.unit()).header.name
                ));
            }
        }
        if !bp.build_list.is_empty() {
            line(format!("Builds:"));
            for build in bp.build_list.iter() {
                line(format!(
                    "  {}",
                    self.blueprints.get_unit(build.unit()).header.name
                ));
            }
        }
        if bp.defence_bonus_to_unit_on_top != 0 {
            line(format!(
                "Gives {}% defence to units on top",
                bp.defence_bonus_to_unit_on_top
            ))
        }
        if bp.defence_bonus_to_adjacent_buildings != 0 {
            line(format!(
                "Gives {}% defence to adjacent buildings",
                bp.defence_bonus_to_adjacent_buildings
            ))
        }
        if !bp.abilities.is_empty() {
            line(format!("Abilities:"));
            for ability in bp.abilities.iter() {
                line(format!(
                    "  {}",
                    self.blueprints.get_ability(ability.ability()).name
                ));
            }
        }
        if !bp.build_constraints.is_empty() {
            line(format!("Building constraints:"));
            for constraint in bp.build_constraints.iter() {
                let description = constraint.view(self.blueprints);
                line(format!("  {}", description));
            }
        }
        if !bp.required_tech.is_empty() {
            line(format!("Requires to have researched:"));
            for tech in bp.required_tech.iter() {
                line(format!("  {}", self.blueprints.get_tech(tech.tech()).name));
            }
        }
        if !bp.powers.is_empty() {
            line(format!("Powers:"));
            for power in bp.powers.iter() {
                line(format!(
                    "  {}",
                    self.blueprints.get_power(power.power()).name
                ));
            }
        }

        line(format!("Stats:"));

        UnitStatsWidget {
            stats: &bp.stats,
            bonus: &None,
            alignment: Alignment::Right,
        }
        .render(
            UnitStatsWidget::RECT.offset(layout::Offset {
                x: 0,
                y: current_line as i32,
            }),
            &mut temp_buffer,
        );

        let view_area = IVec2::new(area.width.into(), area.height.into());
        for xy in iter_area(view_area) {
            if rect_contains(&temp_area, &xy) {
                *buf.get_mut(area.x + xy.x as u16, area.y + xy.y as u16) =
                    temp_buffer.get(xy.x as u16, xy.y as u16).clone();
            }
        }
    }
}
