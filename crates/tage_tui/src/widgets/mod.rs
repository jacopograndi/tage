pub mod battle_widget;
pub mod board_tile_widget;
pub mod board_widget;
pub mod bp_unit_widget;
pub mod button_widget;
pub mod cursor_widget;
pub mod member_widget;
pub mod panel_widget;
pub mod research_widget;
pub mod terrain_widget;
pub mod unit_stats_widget;
pub mod unit_widget;

pub use battle_widget::*;
pub use board_tile_widget::*;
pub use board_widget::*;
pub use bp_unit_widget::*;
pub use button_widget::*;
pub use cursor_widget::*;
pub use member_widget::*;
pub use panel_widget::*;
pub use research_widget::*;
pub use terrain_widget::*;
pub use unit_stats_widget::*;
pub use unit_widget::*;

use ratatui::prelude::*;
use tage_core::prelude::*;

pub fn map_color<F>(color: Color, f: F) -> Color
where
    F: Fn(u8, u8, u8) -> (u8, u8, u8),
{
    match color {
        Color::Rgb(r, g, b) => {
            let (r, g, b) = f(r, g, b);
            Color::Rgb(r, g, b)
        }
        _ => color,
    }
}

pub fn dim_color(color: Color, amt: f32) -> Color {
    map_color(color, |r, g, b| {
        (
            (r as f32 * amt) as u8,
            (g as f32 * amt) as u8,
            (b as f32 * amt) as u8,
        )
    })
}

// https://www.w3.org/TR/AERT/#color-contrast
pub fn text_color_contrast(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let perceived_luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            if perceived_luminance < 128. {
                Color::White
            } else {
                Color::Black
            }
        }
        _ => Color::LightMagenta,
    }
}

pub fn color_to_u32(color: [u8; 3]) -> u32 {
    let mut out: u32 = 0;
    out += color[2] as u32;
    out += (color[1] as u32) << 8;
    out += (color[0] as u32) << 16;
    out
}

pub fn terrain_tile_color(terrain: &TerrainTile, terrain_blueprint: &TerrainBlueprint) -> Color {
    if terrain.has_road {
        Color::Rgb(200, 200, 200)
    } else {
        terrain_color(terrain_blueprint)
    }
}

pub fn terrain_color(terrain_blueprint: &TerrainBlueprint) -> Color {
    match terrain_blueprint.header.glyph.as_str() {
        r"---" => Color::Rgb(154, 187, 38),
        r"..." => Color::Rgb(30, 100, 152),
        r"~~~" => Color::Rgb(250, 229, 107),
        r"/\\" => Color::Rgb(255, 255, 255),
        r"###" => Color::Rgb(50, 120, 60),
        r"&&&" => Color::Rgb(163, 142, 100),
        r"())" => Color::Rgb(180, 220, 120),
        r"<>>" => Color::Rgb(120, 120, 120),
        r",,," => Color::Rgb(40, 164, 100),
        _ => unreachable!(),
    }
}

pub fn resource_color(res: &Resource) -> Color {
    match res {
        Resource::Gold => Color::Rgb(240, 168, 43),
        Resource::Food => Color::Rgb(255, 210, 130),
    }
}

pub fn collectable_color(collectable: &Collectable) -> Color {
    match collectable {
        Collectable::BonusFood => resource_color(&Resource::Food),
        Collectable::BonusGold => resource_color(&Resource::Gold),
        Collectable::Ruins => Color::Rgb(255, 255, 255),
        Collectable::Relic => Color::Rgb(118, 47, 48),
    }
}
