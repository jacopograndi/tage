use crossterm::event::{self, Event, KeyCode};
use tage_core::vec2::IVec2;

use crate::Keybinds;

#[derive(Debug, Clone, Default)]
pub struct GameInput {
    pub acc: IVec2,
    pub select: bool,
    pub back: bool,
    pub quit: bool,
    pub next: bool,
    pub toggle_details: bool,
    pub toggle_blueprints: bool,
    pub toggle_map_dim: bool,
    pub zoom: i32,
    pub quicksave: bool,
    pub quickload: bool,
    pub toggle_pause_queue: bool,
    pub step_queue: bool,
    pub short_move: bool,
    pub short_attack: bool,
    pub short_build: bool,
    pub short_done: bool,
    pub short_tech: bool,
    pub short_pass: bool,
}

impl GameInput {
    pub fn from_events(keybinds: &Keybinds, events: &Vec<Event>) -> Self {
        let mut input = GameInput::default();
        for event in events.iter() {
            if let Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press {
                    if key.code == keybinds.pause {
                        input.quit = true;
                    }
                    if key.code == keybinds.right {
                        input.acc.x -= 1;
                    }
                    if key.code == keybinds.left {
                        input.acc.x += 1;
                    }
                    if key.code == keybinds.up {
                        input.acc.y -= 1;
                    }
                    if key.code == keybinds.down {
                        input.acc.y += 1;
                    }
                    if key.code == keybinds.fast_right {
                        input.acc.x -= 10;
                    }
                    if key.code == keybinds.fast_left {
                        input.acc.x += 10;
                    }
                    if key.code == keybinds.fast_up {
                        input.acc.y -= 10;
                    }
                    if key.code == keybinds.fast_down {
                        input.acc.y += 10;
                    }
                    if key.code == keybinds.back {
                        input.back = true;
                    }
                    if key.code == keybinds.forward {
                        input.select = true;
                    }
                    if key.code == keybinds.details {
                        input.toggle_details = true;
                    }
                    if key.code == keybinds.blueprints {
                        input.toggle_blueprints = true;
                    }
                    if key.code == keybinds.dim_map {
                        input.toggle_map_dim = true;
                    }
                    if key.code == keybinds.next {
                        input.next = true;
                    }
                    if key.code == keybinds.zoom_in {
                        input.zoom = -1;
                    }
                    if key.code == keybinds.zoom_out {
                        input.zoom = 1;
                    }
                    if key.code == keybinds.quicksave {
                        input.quicksave = true;
                    }
                    if key.code == keybinds.quickload {
                        input.quickload = true;
                    }
                    if key.code == keybinds.pause_queue {
                        input.toggle_pause_queue = true;
                    }
                    if key.code == keybinds.step_queue {
                        input.step_queue = true;
                    }
                    if key.code == keybinds.short_pass {
                        input.short_pass = true;
                    }
                    if key.code == keybinds.short_tech {
                        input.short_tech = true;
                    }
                    if key.code == keybinds.short_done {
                        input.short_done = true;
                    }
                    if key.code == keybinds.short_build {
                        input.short_build = true;
                    }
                    if key.code == keybinds.short_attack {
                        input.short_attack = true;
                    }
                    if key.code == keybinds.short_move {
                        input.short_move = true;
                    }
                }
            }
        }
        input
    }
}

#[derive(Debug, Clone, Default)]
pub struct MenuInput {
    pub acc: IVec2,
    pub select: bool,
    pub back: bool,
    pub quit: bool,
    pub keycode: Option<KeyCode>,
}

impl MenuInput {
    pub fn from_events(keybinds: &Keybinds, events: &Vec<Event>) -> Self {
        let mut input = MenuInput::default();
        for event in events.iter() {
            if let Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press {
                    if key.code == keybinds.pause {
                        input.quit = true;
                    }
                    if key.code == keybinds.right {
                        input.acc.x -= 1;
                    }
                    if key.code == keybinds.left {
                        input.acc.x += 1;
                    }
                    if key.code == keybinds.up {
                        input.acc.y -= 1;
                    }
                    if key.code == keybinds.down {
                        input.acc.y += 1;
                    }
                    if key.code == keybinds.fast_right {
                        input.acc.x -= 10;
                    }
                    if key.code == keybinds.fast_left {
                        input.acc.x += 10;
                    }
                    if key.code == keybinds.fast_up {
                        input.acc.y -= 10;
                    }
                    if key.code == keybinds.fast_down {
                        input.acc.y += 10;
                    }
                    if key.code == keybinds.back {
                        input.back = true;
                    }
                    if key.code == keybinds.forward {
                        input.select = true;
                    }
                    input.keycode = Some(key.code);
                }
            }
        }
        input
    }
}
