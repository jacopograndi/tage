use crate::TileSelectionType;

use super::*;
use ratatui::{prelude::*, widgets::*};
use tage_core::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct BoardTileWidget<'a> {
    pub board: &'a Board,
    pub blueprints: &'a Blueprints,
    pub tile_pos: IVec2,
    pub selection_type: Option<TileSelectionType>,
    pub only_player_color: bool,
    pub show_spawns: bool,
    pub travel_direction: Option<TravelDirection>,
    pub fog_player: &'a PlayerId,
}

#[derive(Debug, Clone, Copy)]
pub enum TravelDirection {
    Up,
    Down,
    Left,
    Right,
}

impl<'a> Widget for BoardTileWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tile = self.board.grid.get_at(&self.tile_pos);
        let mut hide_units = false;
        let mut hide_tile = false;
        if let Some(fog) = self.board.fog.get(&self.fog_player) {
            let visibility = fog.get_at(&self.tile_pos);
            match visibility {
                FogTile::Visible => {}
                FogTile::Explored => hide_units = true,
                FogTile::Hidden => hide_tile = true,
            }
        }

        let tile = if !hide_units {
            tile.clone()
        } else {
            let mut tile = tile.clone();
            tile.unit = None;
            tile.building = None;
            tile
        };

        let blueprint = self
            .blueprints
            .terrain
            .get(&tile.terrain.blueprint_id)
            .unwrap();
        let pos = area.as_position();
        let (mut overall_color, text_color_fg) = if let Some(_) = self.travel_direction {
            (Color::Rgb(240, 240, 0), Color::Black)
        } else if let Some(selected) = self.selection_type {
            match selected {
                TileSelectionType::Movement => (Color::Rgb(40, 50, 200), Color::Black),
                TileSelectionType::Attack => (Color::Rgb(255, 30, 10), Color::Black),
                TileSelectionType::Target => (Color::Rgb(120, 120, 120), Color::Black),
            }
        } else if !hide_tile {
            if let Some(unit) = tile.get_top_unit() {
                let owner = self.board.get_player(&unit.owner);
                let background = if !unit.done || unit.owner != self.board.current_player_turn {
                    Color::from_u32(owner.color)
                } else {
                    dim_color(Color::from_u32(owner.color), 0.5)
                };
                (background, text_color_contrast(background))
            } else {
                (terrain_tile_color(&tile.terrain, blueprint), Color::Black)
            }
        } else {
            (Color::Black, Color::White)
        };

        if self.only_player_color && tile.get_top_unit().is_none() {
            overall_color = dim_color(overall_color, 0.4)
        };

        if hide_units {
            overall_color = dim_color(overall_color, 0.6)
        }

        Block::default()
            .style(Style::default().bg(overall_color).fg(text_color_fg))
            .render(area, buf);

        if !hide_tile {
            if let Some(ref unit) = tile.unit {
                let unit_blueprint = self.blueprints.units.get(&unit.blueprint_id).unwrap();
                Paragraph::new(unit_blueprint.header.glyph.to_string())
                    .style(Style::default().fg(text_color_fg))
                    .render(Rect::new(pos.x, pos.y, 3, 1), buf);
                if area.height > 3 && area.width > 3 {
                    Paragraph::new(unit.health.to_string())
                        .style(Style::default().fg(text_color_fg))
                        .render(Rect::new(pos.x, pos.y + 1, 3, 1), buf);
                }
            }

            if area.width > 3 {
                if let Some(unit) = tile.get_top_unit() {
                    let owner = self.board.get_player(&unit.owner);
                    Paragraph::new(owner.symbol.as_str())
                        .style(Style::default().fg(text_color_fg))
                        .render(Rect::new(pos.x + area.width - 1, pos.y, 1, 1), buf);
                }
            }

            if let Some(height) = if area.height > 1 {
                Some(area.height - 1)
            } else {
                tile.get_top_unit().is_none().then_some(0)
            } {
                if area.width > 3 {
                    Paragraph::new(blueprint.header.glyph.as_str())
                        .style(Style::default().fg(text_color_fg).bg(overall_color))
                        .render(Rect::new(pos.x, pos.y + height, area.width, 1), buf);
                } else {
                    Paragraph::new(blueprint.header.glyph.as_str())
                        .style(Style::default().fg(text_color_fg).bg(overall_color))
                        .render(Rect::new(pos.x, pos.y + height, 1, 1), buf);
                }

                if let Some(ref resource) = tile.terrain.resource {
                    let mut color = resource_color(resource);
                    if self.only_player_color {
                        color = dim_color(color, 0.4)
                    };
                    Paragraph::new(resource.to_string())
                        .style(Style::default().bg(color).fg(Color::Black))
                        .render(Rect::new(pos.x, pos.y + height, 1, 1), buf);
                }
                if tile.terrain.has_road {
                    Paragraph::new("=")
                        .style(Style::default().fg(text_color_fg))
                        .render(Rect::new(pos.x, pos.y + height, 1, 1), buf);
                }
                if let Some(ref collectable) = tile.terrain.collectable {
                    let mut color = collectable_color(collectable);
                    if self.only_player_color {
                        color = dim_color(color, 0.4)
                    };
                    Paragraph::new(collectable.to_string())
                        .style(Style::default().bg(color).fg(Color::Black))
                        .render(Rect::new(pos.x + 1, pos.y + height, 1, 1), buf);
                }
            }
        }

        if let Some(dir) = self.travel_direction {
            let arrow = match dir {
                TravelDirection::Up => "^",
                TravelDirection::Down => "v",
                TravelDirection::Left => "<",
                TravelDirection::Right => ">",
            };
            let x = if area.width > 1 { 1 } else { 0 };
            Paragraph::new(arrow)
                .style(Style::default().fg(text_color_fg))
                .render(Rect::new(pos.x + x, pos.y, 1, 1), buf);
        }

        if !hide_tile {
            if let Some(ref building) = tile.building {
                let building_blueprint = self.blueprints.units.get(&building.blueprint_id).unwrap();
                if area.height == 1 {
                    if tile.unit.is_none() {
                        Paragraph::new(building_blueprint.header.glyph.to_string())
                            .style(Style::default().fg(text_color_fg))
                            .render(Rect::new(pos.x + 1, pos.y, 1, 1), buf);
                    }
                } else {
                    if area.height > 3 {
                        Paragraph::new(building_blueprint.header.glyph.to_string())
                            .style(Style::default().fg(text_color_fg))
                            .render(Rect::new(pos.x + 1, pos.y + 3, 1, 1), buf);
                    } else {
                        Paragraph::new(building_blueprint.header.glyph.to_string())
                            .style(Style::default().fg(text_color_fg))
                            .render(Rect::new(pos.x + 1, pos.y + 1, 1, 1), buf);
                    }
                }
                if area.height > 3 && area.width > 3 {
                    Paragraph::new(building.health.to_string())
                        .style(Style::default().fg(text_color_fg))
                        .render(Rect::new(pos.x, pos.y + 2, 3, 1), buf);
                }
            }

            if self.show_spawns {
                if let Some(spawn_point) = &tile.spawn_point {
                    if let Some(player) = self.board.players.iter().find(|p| &p.id == spawn_point) {
                        let background = Color::from_u32(player.color);
                        let text_color_fg = text_color_contrast(background);
                        Paragraph::new(format!("{:2}", spawn_point.get().to_string()))
                            .style(Style::default().fg(text_color_fg).bg(background))
                            .render(area, buf);
                    } else {
                        Paragraph::new(format!("{:2}", spawn_point.get().to_string()))
                            .style(Style::default().fg(Color::Black))
                            .render(area, buf);
                    }
                }
            }
        }
    }
}
