//! Tage gameplay code
//!
//! Activate the feature `integration_test` to test full games, it's disabled by default
//! as it is very slow.

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
