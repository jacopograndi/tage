use ratatui::{prelude::*, widgets::*};

#[derive(Clone, Copy)]
pub struct BoardCursorWidget {
    pub style: Style,
}

impl Default for BoardCursorWidget {
    fn default() -> Self {
        Self {
            style: Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
        }
    }
}

impl Widget for BoardCursorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // manual check NO_COLOR, ratatui isn't exposing crossterm's ansi_color_disabled
        if !std::env::var("NO_COLOR")
            .unwrap_or("".to_string())
            .is_empty()
        {
            Block::bordered()
                .border_type(BorderType::QuadrantInside)
                .render(area, buf)
        } else {
            Block::default().style(self.style).render(area, buf)
        }
    }
}
