use ratatui::{prelude::*, widgets::*};

pub struct Button<'a> {
    pub string: &'a str,
    pub pressed: bool,
}

impl<'a> Widget for Button<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = if self.pressed {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        Paragraph::new(self.string).style(style).render(area, buf)
    }
}

pub struct ButtonAt<'a> {
    pub string: &'a str,
    pub index: i32,
    pub cursor: i32,
}

impl<'a> ButtonAt<'a> {
    pub fn new(string: &'a str, index: i32, cursor: i32) -> Self {
        Self {
            string,
            index,
            cursor,
        }
    }
}

impl<'a> Widget for ButtonAt<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Button {
            string: self.string,
            pressed: self.index == self.cursor,
        }
        .render(area, buf)
    }
}
