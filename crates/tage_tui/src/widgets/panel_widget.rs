use ratatui::{prelude::*, widgets::*};

pub const DECOR_0: &'static str = r"* ";
pub const DECOR_1: &'static str = r"WM";
pub const DECOR_2: &'static str = r".";
pub const DECOR_3: &'static str = r" ` ";

pub struct PanelWidget<'a> {
    pub pattern: &'a str,
    pub style: Style,
    pub border: bool,
}

impl<'a> PanelWidget<'a> {
    pub fn new(pattern: &'a str) -> Self {
        Self {
            pattern,
            ..Default::default()
        }
    }
    pub fn with_border(self, border: bool) -> Self {
        Self { border, ..self }
    }
}

impl<'a> Default for PanelWidget<'a> {
    fn default() -> Self {
        Self {
            pattern: DECOR_0,
            style: Style::default().fg(Color::Rgb(50, 50, 50)).bg(Color::Black),
            border: true,
        }
    }
}

impl<'a> Widget for PanelWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        use Constraint::*;
        let area = if self.border {
            Clear::default().render(area, buf);
            let [area] = Layout::default()
                .constraints([Min(0)])
                .margin(1)
                .areas(area);
            area
        } else {
            area
        };

        let w = self.pattern.len() as u16;
        for y in area.y..area.y + area.height {
            for x in 0..(area.width / w) + 1 {
                let limit = (area.width as i32 - (x * w) as i32).max(0) as u16;
                if limit > 0 {
                    buf.set_stringn(area.x + x * w, y, self.pattern, limit as usize, self.style);
                }
            }
        }

        if area.height > 3 && self.border {
            Block::bordered()
                .border_style(self.style)
                .border_type(BorderType::QuadrantOutside)
                .render(area, buf)
        }
    }
}
