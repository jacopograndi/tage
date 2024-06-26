use std::collections::HashMap;

use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};

use crate::get_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub keybinds: Keybinds,
    pub machine_speed: MachineSpeed,
}

impl Settings {
    pub fn disk_path() -> Option<std::path::PathBuf> {
        let mut path = get_data_dir()?;
        path.push("settings.ron");
        tracing::trace!(target: "settings", "path: {:?}", path);
        Some(path)
    }

    pub fn from_disk() -> Option<Self> {
        let Some(path) = Self::disk_path() else {
            return None;
        };

        let Ok(raw) = std::fs::read_to_string(path) else {
            return None;
        };

        ron::from_str(raw.as_str()).ok()
    }

    pub fn to_disk(&self) {
        let Some(path) = Self::disk_path() else {
            return;
        };

        let string = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default()).unwrap();
        std::fs::write(path, string).unwrap();
    }
}

/// struct reflection
#[macro_export]
macro_rules! define_keybinds {
    ( $($field:ident),* ) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Keybinds {
            $(pub $field: KeyCode,)*
        }
        impl Keybinds {
            pub fn fields_to_string() -> Vec<String> {
                [
                    $(stringify!($field),)*
                ].into_iter().map(|s| s.to_string()).collect()
            }
            pub fn get_key_from_string(&mut self, s: &str) -> &mut KeyCode {
                match s {
                    $(stringify!($field) => &mut self.$field,)*
                    _ => unreachable!()
                }
            }
        }
    };
}

define_keybinds!(
    pause,
    up,
    down,
    left,
    right,
    fast_up,
    fast_down,
    fast_left,
    fast_right,
    forward,
    next,
    back,
    details,
    blueprints,
    dim_map,
    zoom_in,
    zoom_out,
    quicksave,
    quickload,
    pause_queue,
    step_queue,
    short_move,
    short_attack,
    short_build,
    short_done,
    short_tech,
    short_pass
);

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            pause: KeyCode::Esc,
            up: KeyCode::Char('w'),
            right: KeyCode::Char('a'),
            down: KeyCode::Char('s'),
            left: KeyCode::Char('d'),
            fast_up: KeyCode::Char('W'),
            fast_right: KeyCode::Char('A'),
            fast_down: KeyCode::Char('S'),
            fast_left: KeyCode::Char('D'),
            next: KeyCode::Char('f'),
            forward: KeyCode::Char(' '),
            back: KeyCode::Char('u'),
            details: KeyCode::Char('b'),
            blueprints: KeyCode::Char('v'),
            dim_map: KeyCode::Char('m'),
            zoom_in: KeyCode::Char('z'),
            zoom_out: KeyCode::Char('Z'),
            quicksave: KeyCode::F(5),
            quickload: KeyCode::F(6),
            step_queue: KeyCode::Char('.'),
            pause_queue: KeyCode::Char(','),
            short_move: KeyCode::Char('q'),
            short_attack: KeyCode::Char('x'),
            short_build: KeyCode::Char('c'),
            short_done: KeyCode::Char('g'),
            short_tech: KeyCode::Char('t'),
            short_pass: KeyCode::Tab,
        }
    }
}

impl Keybinds {
    pub fn vim() -> Self {
        Self {
            pause: KeyCode::Char(':'),
            up: KeyCode::Char('k'),
            right: KeyCode::Char('h'),
            down: KeyCode::Char('j'),
            left: KeyCode::Char('l'),
            fast_up: KeyCode::Char('K'),
            fast_right: KeyCode::Char('H'),
            fast_down: KeyCode::Char('J'),
            fast_left: KeyCode::Char('L'),
            next: KeyCode::Char('n'),
            forward: KeyCode::Enter,
            back: KeyCode::Esc,
            details: KeyCode::Char('b'),
            blueprints: KeyCode::Char('v'),
            dim_map: KeyCode::Char('m'),
            zoom_in: KeyCode::Char('z'),
            zoom_out: KeyCode::Char('Z'),
            quicksave: KeyCode::Char('Q'),
            quickload: KeyCode::Char('!'),
            step_queue: KeyCode::Char('.'),
            pause_queue: KeyCode::Char(','),
            short_move: KeyCode::Char('q'),
            short_attack: KeyCode::Char('x'),
            short_build: KeyCode::Char('c'),
            short_done: KeyCode::Char('g'),
            short_tech: KeyCode::Char('t'),
            short_pass: KeyCode::Tab,
        }
    }

    pub fn fields_to_description() -> HashMap<&'static str, &'static str> {
        [
            ("pause", "Opens the pause menu"),
            ("up", "Move the cursor up by 1"),
            ("right", "Move the cursor right by 1"),
            ("down", "Move the cursor down by 1"),
            ("left", "Move the cursor left by 1"),
            ("fast_up", "Move the cursor up by 10"),
            ("fast_right", "Move the cursor right by 10"),
            ("fast_down", "Move the cursor down by 10"),
            ("fast_left", "Move the cursor left by 10"),
            (
                "next",
                "Move the map cursor to the nearest unit that can still do actions",
            ),
            ("forward", "Press the focused button"),
            ("back", "Undo the last move"),
            (
                "details",
                "Shows or hides a panel in game with unit information",
            ),
            (
                "blueprints",
                "Shows or hides a panel in game where more information about that unit can be retrieved",
            ),
            ("dim_map", "Makes the terrain be less bright so that units stand out more"),
            ("zoom_in", "Zooms the map in, adding details"),
            ("zoom_out", "Zooms the map out, removing details"),
            ("quicksave", "Saves the game"),
            ("quickload", "Loads the last quicksaved game"),
            ("step_queue", "Makes the machine player do one action"),
            ("pause_queue", "Halts or resume the machine player doing actions"),
            ("short_move", "Hotkey to move the unit under the cursor"),
            ("short_attack", "Hotkey to attack with the unit under the cursor"),
            ("short_build", "Hotkey to build with the unit under the cursor"),
            ("short_tech", "Hotkey to open the research menu"),
            ("short_pass", "Hotkey to end the day"),
            ("short_done", "Hotkey to set the unit as done"),
        ]
        .into_iter()
        .collect()
    }

    pub fn get_name_from_field(&mut self, s: &str) -> &'static str {
        match s {
            "pause" => "Pause",
            "up" => "Up",
            "right" => "Right",
            "down" => "Down",
            "left" => "Left",
            "fast_up" => "Fast up",
            "fast_right" => "Fast right",
            "fast_down" => "Fast down",
            "fast_left" => "Fast left",
            "next" => "Next unit",
            "forward" => "Forward",
            "back" => "Back",
            "details" => "Details",
            "blueprints" => "Blueprints",
            "dim_map" => "Dim map",
            "zoom_in" => "Zoom in",
            "zoom_out" => "Zoom out",
            "quicksave" => "Quicksave",
            "quickload" => "Quickload",
            "step_queue" => "Step Queue",
            "pause_queue" => "Pause Queue",
            "short_move" => "Hotkey Move",
            "short_attack" => "Hotkey Attack",
            "short_build" => "Hotkey Build",
            "short_tech" => "Hotkey Research",
            "short_pass" => "Hotkey End Day",
            "short_done" => "Hotkey Done",
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MachineSpeed {
    StepSelects,
    #[default]
    StepMovesSlow,
    StepMoves,
    Skip,
}
