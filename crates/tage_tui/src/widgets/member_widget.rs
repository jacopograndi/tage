use ratatui::{prelude::*, widgets::*};

use crate::{text_color_contrast, Member};

pub struct MemberWidget<'a> {
    pub member: &'a Member,
}

impl<'a> Widget for MemberWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear::default().render(area, buf);
        let m = self.member;
        let color = Color::Rgb(m.color[0], m.color[1], m.color[2]);
        Paragraph::new(m.name.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color_contrast(color)).bg(color))
            .render(area, buf)
    }
}
