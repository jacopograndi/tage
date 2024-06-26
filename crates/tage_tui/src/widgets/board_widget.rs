use crate::TileSelectionType;

use super::*;
use ratatui::prelude::*;
use tage_core::prelude::*;
use tracing::trace;

#[derive(Debug, Clone, Copy)]
pub struct BoardWidget<'a> {
    pub board: &'a Board,
    pub blueprints: &'a Blueprints,
    pub cursor: IVec2,
    pub movement_tiles: &'a Vec<IVec2>,
    pub attack_tiles: &'a Vec<IVec2>,
    pub target_tiles: &'a Vec<IVec2>,
    pub only_player_color: bool,
    pub zoom: i32,
    pub show_spawns: bool,
    pub travel_path: &'a Vec<IVec2>,
    pub fog_player: &'a PlayerId,
}

impl<'a> Widget for BoardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        PanelWidget::new(DECOR_3)
            .with_border(false)
            .render(area, buf);

        let tile_size = match self.zoom {
            1 => IVec2::new(2, 1),
            2 => IVec2::new(3, 1),
            3 => IVec2::new(4, 1),
            4 => IVec2::new(5, 2),
            5 => IVec2::new(9, 4),
            _ => IVec2::new(4, 2),
        };
        let board_area = tile_size * self.board.grid.size;
        let board_rect = Rect::new(0, 0, board_area.x as u16, board_area.y as u16);
        let mut board_buffer = Buffer::empty(board_rect);
        for xy in iter_area(self.board.grid.size) {
            let selection_type = if self.target_tiles.contains(&xy) {
                Some(TileSelectionType::Target)
            } else if self.attack_tiles.contains(&xy) {
                Some(TileSelectionType::Attack)
            } else if self.movement_tiles.contains(&xy) {
                Some(TileSelectionType::Movement)
            } else {
                None
            };
            let mut dir = None;
            if let Some(i) = self.travel_path.iter().position(|pos| pos == &xy) {
                if i < self.travel_path.len() - 1 {
                    trace!("{}, {}", xy, self.travel_path[i + 1] - self.travel_path[i]);
                    dir = match self.travel_path[i + 1] - self.travel_path[i] {
                        IVec2 { x: 1, y: 0 } => Some(TravelDirection::Right),
                        IVec2 { x: -1, y: 0 } => Some(TravelDirection::Left),
                        IVec2 { x: 0, y: 1 } => Some(TravelDirection::Down),
                        IVec2 { x: 0, y: -1 } => Some(TravelDirection::Up),
                        _ => None,
                    }
                }
            }
            BoardTileWidget {
                board: self.board,
                blueprints: self.blueprints,
                tile_pos: xy,
                selection_type,
                only_player_color: self.only_player_color,
                show_spawns: self.show_spawns,
                travel_direction: dir,
                fog_player: self.fog_player,
            }
            .render(
                Rect {
                    x: (xy.x * tile_size.x) as u16,
                    y: (xy.y * tile_size.y) as u16,
                    width: tile_size.x as u16,
                    height: tile_size.y as u16,
                },
                &mut board_buffer,
            )
        }
        let cursor_pos = self.cursor * tile_size;
        BoardCursorWidget::default().render(
            Rect {
                x: (self.cursor.x * tile_size.x) as u16,
                y: (self.cursor.y * tile_size.y) as u16,
                width: tile_size.x as u16,
                height: tile_size.y as u16,
            },
            &mut board_buffer,
        );
        let view_area = IVec2::new(area.width.into(), area.height.into());
        let view_area_center = view_area / 2;

        let to_cursor = cursor_pos - view_area_center;
        let to_cursor = to_cursor.clamp(
            IVec2::new(0, 0),
            (board_area - view_area).max(IVec2::new(0, 0)),
        );
        let off = ((view_area - board_area) / 2).clamp(
            IVec2::new(0, 0),
            (board_area - view_area).max(view_area - board_area),
        );
        let to_cursor = to_cursor - off;
        for xy in iter_area(view_area) {
            let xy_off = xy + to_cursor;
            if rect_contains(&board_area, &xy_off) {
                *buf.get_mut(area.x + xy.x as u16, area.y + xy.y as u16) =
                    board_buffer.get(xy_off.x as u16, xy_off.y as u16).clone();
            }
        }
    }
}
