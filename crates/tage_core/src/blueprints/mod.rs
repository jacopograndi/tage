pub mod ability_bp;
pub mod blueprints;
pub mod bonus;
pub mod civilization_bp;
pub mod constraints;
pub mod id;
pub mod power_bp;
pub mod resources;
pub mod tech_bp;
pub mod terrain_bp;
pub mod unit_bp;

pub use ability_bp::*;
pub use blueprints::*;
pub use bonus::*;
pub use civilization_bp::*;
pub use constraints::*;
pub use id::*;
pub use power_bp::*;
pub use resources::*;
pub use tech_bp::*;
pub use terrain_bp::*;
pub use unit_bp::*;

pub fn view_level(level: i32) -> String {
    match level {
        0 => "Dark Age",
        1 => "Feudal Age",
        2 => "Castle Age",
        3 => "Imperial Age",
        _ => "Unknown Age",
    }
    .to_string()
}
