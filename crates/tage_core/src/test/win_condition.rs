#[cfg(test)]
mod test_win_condition {
    use std::{collections::HashMap, sync::Arc};

    use crate::prelude::*;

    fn assert_eq_players(list_a_opt: Option<Vec<PlayerId>>, list_b_opt: Option<Vec<PlayerId>>) {
        match (&list_a_opt, &list_b_opt) {
            (Some(list_a), Some(list_b)) => {
                for a in list_a {
                    assert!(list_b.contains(a));
                }
                for b in list_b {
                    assert!(list_a.contains(b));
                }
            }
            (None, None) => {}
            _ => eprintln!("{:?} != {:?}", &list_a_opt, &list_b_opt),
        }
    }

    fn test_board(ids: Vec<(PlayerId, Option<TeamId>)>) -> Board {
        Board {
            bp: Arc::new(Blueprints::default()),
            grid: Grid::default(IVec2::splat(3)),
            players: ids
                .clone()
                .into_iter()
                .map(|(player, team)| Player {
                    id: player,
                    team,
                    ..Default::default()
                })
                .collect(),
            day: 0,
            current_player_turn: ids.first().unwrap().0.clone(),
            player_turn_order: ids.into_iter().map(|(id, _)| id).collect(),
            fog: HashMap::new(),
            fog_base: FogTile::Visible,
        }
    }

    #[test]
    fn everybody_dead() {
        let board = test_board(vec![(PlayerId::new(0), None), (PlayerId::new(1), None)]);
        assert_eq_players(board.get_winners(), Some(vec![]));
    }

    #[test]
    fn one_left_wins() {
        let mut board = test_board(vec![(PlayerId::new(0), None), (PlayerId::new(1), None)]);
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(0),
            ..Default::default()
        });
        assert_eq_players(board.get_winners(), Some(vec![PlayerId::new(0)]));
    }

    #[test]
    fn game_continues() {
        let mut board = test_board(vec![(PlayerId::new(0), None), (PlayerId::new(1), None)]);
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(0),
            ..Default::default()
        });
        board.grid.get_at_mut(&IVec2::ONE).unit = Some(Unit {
            owner: PlayerId::new(1),
            ..Default::default()
        });
        assert_eq_players(board.get_winners(), None);
    }

    #[test]
    fn alliance_wins_against_none() {
        let mut board = test_board(vec![
            (PlayerId::new(0), Some(TeamId::new(0))),
            (PlayerId::new(1), Some(TeamId::new(0))),
            (PlayerId::new(2), None),
        ]);
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(0),
            ..Default::default()
        });
        board.grid.get_at_mut(&IVec2::ONE).unit = Some(Unit {
            owner: PlayerId::new(1),
            ..Default::default()
        });
        assert_eq_players(
            board.get_winners(),
            Some(vec![PlayerId::new(0), PlayerId::new(1)]),
        );
    }

    #[test]
    fn alliance_wins_against_another_alliance() {
        let mut board = test_board(vec![
            (PlayerId::new(0), Some(TeamId::new(1))),
            (PlayerId::new(1), Some(TeamId::new(1))),
            (PlayerId::new(2), Some(TeamId::new(0))),
            (PlayerId::new(3), Some(TeamId::new(0))),
        ]);
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(0),
            ..Default::default()
        });
        board.grid.get_at_mut(&IVec2::ONE).unit = Some(Unit {
            owner: PlayerId::new(1),
            ..Default::default()
        });
        assert_eq_players(
            board.get_winners(),
            Some(vec![PlayerId::new(0), PlayerId::new(1)]),
        );
    }

    #[test]
    fn two_alliances_dead_and_none_alive() {
        let mut board = test_board(vec![
            (PlayerId::new(0), Some(TeamId::new(1))),
            (PlayerId::new(1), Some(TeamId::new(1))),
            (PlayerId::new(2), Some(TeamId::new(0))),
            (PlayerId::new(3), Some(TeamId::new(0))),
            (PlayerId::new(4), None),
        ]);
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(4),
            ..Default::default()
        });
        assert_eq_players(board.get_winners(), Some(vec![PlayerId::new(4)]));
    }

    #[test]
    fn two_nones_alive_one_dead() {
        let mut board = test_board(vec![
            (PlayerId::new(2), None),
            (PlayerId::new(3), None),
            (PlayerId::new(4), None),
        ]);
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(3),
            ..Default::default()
        });
        board.grid.get_at_mut(&IVec2::ZERO).unit = Some(Unit {
            owner: PlayerId::new(4),
            ..Default::default()
        });
        assert_eq_players(board.get_winners(), None);
    }
}
