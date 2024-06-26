use std::collections::HashMap;

use super::{Board, PlayerAction};
use crate::{
    prelude::{player_action::Pre, Act},
    vec2::IVec2,
};

struct Partial {
    actions: Vec<PlayerAction>,
    visited: HashMap<IVec2, i32>,
}

impl Partial {
    fn next_tech(&self, act: PlayerAction) -> Self {
        let mut actions = self.actions.clone();
        actions.push(act);
        Self {
            actions,
            visited: self.visited.clone(),
        }
    }

    fn next_unit(&self, act: PlayerAction, pos: IVec2) -> Self {
        let mut actions = self.actions.clone();
        actions.push(act);
        let mut visited = self.visited.clone();
        visited.entry(pos).and_modify(|acc| *acc -= 1);
        visited.retain(|_, amt| *amt > 0);
        Self { actions, visited }
    }
}

fn all_turns(board: &mut Board) -> Vec<Vec<PlayerAction>> {
    let mut player_actions = vec![];

    let mut frontier: Vec<Partial> = vec![];
    let mut done: Vec<Partial> = vec![];

    let initial_visited: HashMap<IVec2, i32> = board
        .get_player_units_pos(&board.current_player_turn)
        .map(|(_, xy)| (xy, 2))
        .collect();
    frontier.push(Partial {
        actions: vec![],
        visited: initial_visited,
    });

    const MAX_MACHINE_ITERATIONS: u32 = 10000000;
    let mut iter = 0;
    loop {
        if iter > MAX_MACHINE_ITERATIONS {
            crate::log(format!("Out of machine thinking iteration"));
            crate::dump(board.view(), "dump.txt");
            panic!();
        }
        iter += 1;

        let Some(partial) = frontier.pop() else {
            break;
        };

        for act in partial.actions.iter() {
            act.apply(board)
        }

        let mut star: Vec<Partial> = vec![];

        if board
            .get_player(&board.current_player_turn)
            .research_queued
            .is_none()
        {
            for act in PlayerAction::generate(&Pre::Global, board) {
                star.push(partial.next_tech(act));
            }
        }

        for (pos, _) in partial.visited.iter() {
            for act in PlayerAction::generate(&Pre::Tile(*pos), board) {
                star.push(partial.next_unit(act, *pos));
            }
        }

        for act in partial.actions.iter().rev() {
            act.undo(board)
        }
    }

    player_actions.push(PlayerAction::PassTurn);

    done.into_iter().map(|p| p.actions).collect()
}
