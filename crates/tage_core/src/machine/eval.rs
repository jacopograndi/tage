use std::collections::HashMap;

use crate::player::PlayerId;

use super::{peak::Pack, Board};

/// An heuristic that evaluates the strength of each player
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eval {
    pub scores: Vec<i32>,
}

impl Eval {
    /// O(n) -> One board scan
    pub fn from_board(board: &Board, pack: &Pack) -> Eval {
        let mut map: HashMap<PlayerId, i32> = HashMap::new();
        for (_, tile) in board.grid.iter() {
            for unit in tile.get_units() {
                let value = (pack.unit_value_table.get(&unit.blueprint_id) * unit.health) / 100;
                *map.entry(unit.owner.clone()).or_insert(value) += value;
            }
        }
        for player in board.players.iter() {
            // tunable tech and level weigths
            let value = player.researched_technologies.len() as i32 * 100 + player.level * 300;
            *map.entry(player.id.clone()).or_insert(value) += value;
        }
        let mut scores = vec![];
        for player_id in board.player_turn_order.iter() {
            scores.push(map.get(player_id).unwrap().clone());
        }
        Eval { scores }
    }

    pub fn min(board: &Board) -> Eval {
        Eval {
            scores: board.players.iter().map(|_| -100000).collect(),
        }
    }

    pub fn zero_sum(self) -> Eval {
        let average = self.scores.iter().sum::<i32>() / self.scores.len() as i32;
        Eval {
            scores: self.scores.into_iter().map(|s| s - average).collect(),
        }
    }

    pub fn sum(&self) -> i32 {
        self.scores.iter().sum()
    }

    pub fn get(&self, i: usize) -> i32 {
        *self.scores.get(i).unwrap()
    }

    pub fn get_mut(&mut self, i: usize) -> &mut i32 {
        self.scores.get_mut(i).unwrap()
    }
}
