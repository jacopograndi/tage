use ratatui::{prelude::*, widgets::*};
use tage_core::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct UnitStatsWidget<'a> {
    pub stats: &'a UnitStats,
    pub bonus: &'a Option<&'a Bonus>,
    pub alignment: Alignment,
}

impl<'a> UnitStatsWidget<'a> {
    pub const AREA: IVec2 = IVec2 { x: 14, y: 5 };
    pub const RECT: Rect = Rect {
        x: 0,
        y: 0,
        width: Self::AREA.x as u16,
        height: Self::AREA.y as u16,
    };
}

impl<'a> Widget for UnitStatsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::*;

        Clear::default().render(area, buf);

        let values: Vec<Line> = UnitStatsReflect::iter()
            .map(|stat| {
                let stat_val = self.stats.get_stat(stat);
                if let Some(bonus) = self.bonus {
                    let enhanced = self.stats.apply((*bonus).clone());
                    let enha_val = enhanced.get_stat(stat);
                    Line::from(enha_val.to_string()).style(Style::default().fg(
                        match stat_val.cmp(&enha_val) {
                            std::cmp::Ordering::Greater => Color::Red,
                            std::cmp::Ordering::Equal => Color::White,
                            std::cmp::Ordering::Less => Color::Green,
                        },
                    ))
                } else {
                    Line::from(stat_val.to_string())
                }
            })
            .collect();
        let mut rows: Vec<Vec<Line>> = UnitStatsReflect::iter()
            .zip(values.clone())
            .map(|(stat, val)| {
                vec![
                    Line::from(stat.view()).alignment(self.alignment),
                    val.alignment(self.alignment),
                ]
            })
            .collect();
        let mut widths = vec![8, 5];

        let aligned_area = match self.alignment {
            Alignment::Left => {
                rows.iter_mut().for_each(|row| row.reverse());
                widths.reverse();
                area
            }
            Alignment::Right => {
                let [_, aligned] =
                    Layout::horizontal([Min(0), Length(UnitStatsWidget::RECT.width)]).areas(area);
                aligned
            }
            Alignment::Center => {
                rows = vec![
                    UnitStatsReflect::iter()
                        .map(|stat| Line::from(stat.view()).alignment(self.alignment))
                        .collect(),
                    values,
                ];
                widths = UnitStatsReflect::iter().map(|_| 7).collect();
                area
            }
        };

        let table = Table::new(rows.into_iter().map(|row| Row::new(row)), widths);
        Widget::render(table, aligned_area, buf);
    }
}
