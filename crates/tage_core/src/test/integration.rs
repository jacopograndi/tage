#[cfg(feature = "integration_test")]
#[cfg(test)]
mod exhaustive {
    use std::sync::Arc;

    use crate::prelude::*;

    #[test]
    fn wander() {
        const TEST_RUNS: u32 = 10;

        let bp =
            Blueprints::from_assets_location(&("../../".to_string() + BLUEPRINTS_PATH)).unwrap();
        let settings =
            MapSettings::from_string(include_str!("templates/four_players_oasis.ron")).unwrap();
        let settings = settings.resolve(&ResolveInto::Id, &bp);
        for i in 0..TEST_RUNS {
            println!("Iter {}", i);
            play_game(&bp, &settings);
        }
    }

    fn play_game(bp: &Blueprints, settings: &MapSettings) {
        let mut board = Board {
            bp: Arc::new(bp.clone()),
            grid: load_map(bp, settings).unwrap(),
            players: settings
                .players
                .iter()
                .map(|player| Player {
                    id: player.id.clone(),
                    color: player.color,
                    level: player.level,
                    resources: Resources::new(1500, 1500),
                    team: player.team.clone(),
                    name: player.name.clone(),
                    symbol: player.symbol.clone(),
                    civilization: player
                        .civilization
                        .as_ref()
                        .map_or(bp.civilizations.iter().next().unwrap().0.clone(), |civ| {
                            civ.civilization().clone()
                        }),
                    controller: player.controller.clone(),
                    ..Default::default()
                })
                .collect(),
            day: 0,
            current_player_turn: PlayerId::new(0),
            player_turn_order: vec![PlayerId::new(0)],
        };
        for _turn in 0..1000 {
            let machine = match &board.get_current_player().controller {
                Controller::Human => panic!("In testing only use machines"),
                Controller::Machine(m) => m.clone(),
            };
            let actions = get_machine_turn(&bp, &mut board, &machine);
            for action in actions.iter() {
                action.apply(&mut board)
            }
            if board.get_winners().is_some() {
                return;
            }
        }
        panic!("out of iterations");
    }
}
