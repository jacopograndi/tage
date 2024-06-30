//! Tage gameplay code
//!
//! Activate the feature `integration_test` to test full games, it's disabled by default
//! as it is very slow.

use std::env;

pub mod prelude;

pub mod actions;
pub mod blueprints;
pub mod game;
pub mod grid;
pub mod machine;
pub mod player;
pub mod unit;
pub mod vec2;

#[cfg(test)]
mod test;

/// Used to tell serde to not serialize default fields.
/// In combination with marking fields as default results in serde not serializing default fields
/// and setting as the default value fields if during deserialization the field is not present.
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Assets path can be specified at runtime with an env variable. Defaults to assets
pub fn get_assets_dir() -> String {
    if let Ok(assets_path) = env::var("TAGE_ASSETS") {
        assets_path
    } else {
        format!("assets")
    }
}
