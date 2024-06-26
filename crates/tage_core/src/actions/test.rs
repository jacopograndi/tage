#[cfg(test)]
mod travel {
    use crate::{actions::travel::ActTravel, prelude::*, v};
    use std::{collections::HashMap, sync::Arc};

    #[macro_export]
    macro_rules! travel_case {
        ($name: ident, $terrain: literal, $expected: literal, $from: expr, $unitid: expr) => {
            #[test]
            fn $name() {
                let blueprints =
                    Blueprints::from_assets_location(&("../../".to_string() + BLUEPRINTS_PATH))
                        .unwrap();
                let mut board = Board {
                    bp: Arc::new(blueprints.clone()),
                    grid: parse_map(&blueprints, &$terrain).unwrap().grid,
                    players: vec![Player {
                        id: PlayerId::new(0),
                        color: 0x00900000,
                        symbol: "@".to_string(),
                        ..Default::default()
                    }],
                    day: 0,
                    current_player_turn: PlayerId::new(0),
                    player_turn_order: vec![PlayerId::new(0)],
                    fog: HashMap::new(),
                    fog_base: FogTile::Visible,
                };
                {
                    let tile = board.grid.get_at_mut(&$from);
                    let _ = tile.unit.insert(Unit {
                        blueprint_id: $unitid,
                        owner: PlayerId::new(0),
                        ..Default::default()
                    });
                }
                let destinations = ActTravel::get_reachable(
                    &board.get_pos_target(&UnitPos::top($from)).unwrap(),
                    &board,
                );
                let expected_grid = $expected
                    .trim()
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| {
                        l.split_whitespace()
                            .map(|g| g.trim().parse().unwrap())
                            .collect::<Vec<i32>>()
                    })
                    .collect::<Vec<Vec<i32>>>();
                for xy in iter_area(board.grid.size) {
                    let expected_value = expected_grid[xy.y as usize][xy.x as usize];
                    assert_eq!(
                        destinations.iter().find(|d| d.destination == xy).is_some(),
                        (expected_value > 0),
                        "at {}",
                        xy
                    );
                }
            }
        };
    }

    travel_case!(
        simple,
        r"
--- --- /\\ /\\ ---
--- --- --- /\\ ---
--- --- --- --- ---",
        r"
 1   3   3   0   0  
 3   5   7   3   1  
 1   3   5   3   1 ",
        v!(2, 1),
        UnitId(0)
    );

    travel_case!(
        roads,
        r"
/\\ /\\ ---
=-- /\\ =--
=-- =-- =--",
        r"
 3   0   1  
 7   3   3  
 6   5   4 ",
        v!(0, 1),
        UnitId(0)
    );

    travel_case!(
        water,
        r"
/\\ /\\ ---
=-- /\\ =--
=-- ... =--",
        r"
 3   0   1
 7   3   2  
 6   0   2 ",
        v!(0, 1),
        UnitId(0)
    );
}

#[cfg(test)]
mod exhaustive {
    use crate::{actions::player_action::Pre, prelude::*, v};
    use std::{collections::HashMap, sync::Arc};

    #[macro_export]
    macro_rules! undo_case {
        ($name: ident, $terrain: literal, $from: expr, $units: expr) => {
            #[test]
            fn $name() {
                let blueprints =
                    Blueprints::from_assets_location(&("../../".to_string() + BLUEPRINTS_PATH))
                        .unwrap();
                let mut board = Board {
                    bp: Arc::new(blueprints.clone()),
                    grid: parse_map(&blueprints, &$terrain).unwrap().grid,
                    players: vec![
                        Player {
                            id: PlayerId::new(0),
                            color: 0x00900000,
                            symbol: "@".to_string(),
                            resources: Resources {
                                food: 1200,
                                gold: 1200,
                            },
                            ..Default::default()
                        },
                        Player {
                            id: PlayerId::new(1),
                            color: 0x00900090,
                            symbol: "!".to_string(),
                            resources: Resources {
                                food: 1200,
                                gold: 1200,
                            },
                            ..Default::default()
                        },
                    ],
                    day: 0,
                    current_player_turn: PlayerId::new(0),
                    player_turn_order: vec![PlayerId::new(0)],
                    fog: HashMap::new(),
                    fog_base: FogTile::Visible,
                };
                for (pos, unit, owner) in $units.iter() {
                    let tile = board.grid.get_at_mut(&pos);
                    let bp = blueprints.get_unit(&unit);
                    match bp.header.class {
                        UnitClass::Bld => {
                            tile.building = Some(Unit {
                                blueprint_id: unit.clone(),
                                owner: owner.clone(),
                                ..Default::default()
                            });
                        }
                        _ => {
                            tile.unit = Some(Unit {
                                blueprint_id: unit.clone(),
                                owner: owner.clone(),
                                ..Default::default()
                            });
                        }
                    }
                }
                let actions = PlayerAction::generate(&Pre::Tile($from), &mut board);
                for action in actions.iter() {
                    let starting_board = board.clone();
                    action.apply(&mut board);
                    action.undo(&mut board);
                    assert_eq!(starting_board, board);
                }
            }
        };
    }

    undo_case!(
        villager,
        r"
/\\ /\\ ---
=-- /\\ =--
=-- ... =--",
        v!(0, 1),
        [(v!(0, 1), UnitId(0), PlayerId::new(0))]
    );

    undo_case!(
        villager_fight,
        r"
/\\ /\\ ---
=-- /\\ =--
=-- ... =--",
        v!(0, 1),
        [
            (v!(0, 1), UnitId(0), PlayerId::new(0)),
            (v!(1, 1), UnitId(0), PlayerId::new(1))
        ]
    );

    undo_case!(
        towncenter,
        r"
/\\ /\\ ---
=-- /\\ =--
=-- ... =--",
        v!(0, 1),
        [(v!(0, 1), UnitId(100), PlayerId::new(0)),]
    );

    #[test]
    fn every_action() {
        let bp =
            Blueprints::from_assets_location(&("../../".to_string() + BLUEPRINTS_PATH)).unwrap();
        let mut board = Board {
            bp: Arc::new(bp.clone()),
            grid: parse_map(
                &bp,
                r"
/\\ /\\ ---
=-- /\\ =--
=-- ... =--",
            )
            .unwrap()
            .grid,
            players: vec![
                Player {
                    id: PlayerId::new(0),
                    color: 0x00900000,
                    symbol: "@".to_string(),
                    resources: Resources {
                        food: 1200,
                        gold: 1200,
                    },
                    ..Default::default()
                },
                Player {
                    id: PlayerId::new(1),
                    color: 0x00900090,
                    symbol: "!".to_string(),
                    resources: Resources {
                        food: 1200,
                        gold: 1200,
                    },
                    ..Default::default()
                },
            ],
            day: 0,
            current_player_turn: PlayerId::new(0),
            player_turn_order: vec![PlayerId::new(0)],
            fog: HashMap::new(),
            fog_base: FogTile::Visible,
        };

        if let Some(unit_id) = bp.get_unit_from_name("Monk") {
            let unit_bp = bp.get_unit(&unit_id);
            board.grid.get_at_mut(&v!(1, 0)).set_unit(
                Some(Unit {
                    blueprint_id: unit_id,
                    owner: PlayerId::new(0),
                    ..Default::default()
                }),
                &unit_bp,
            );
        }

        if let Some(unit_id) = bp.get_unit_from_name("Monk") {
            let unit_bp = bp.get_unit(&unit_id);
            board.grid.get_at_mut(&v!(2, 0)).set_unit(
                Some(Unit {
                    blueprint_id: unit_id,
                    owner: PlayerId::new(1),
                    ..Default::default()
                }),
                &unit_bp,
            );
        }

        for (unit_id, unit_bp) in bp.units.iter() {
            if unit_bp.unit_size.size > 1 {
                continue;
            }
            let pos = v!(1, 1);
            let owner = PlayerId::new(0);
            let tile = board.grid.get_at_mut(&pos);
            tile.set_unit(
                Some(Unit {
                    blueprint_id: unit_id.clone(),
                    owner: owner.clone(),
                    ..Default::default()
                }),
                unit_bp,
            );
            let actions = PlayerAction::generate(&Pre::Tile(v!(1, 1)), &mut board);
            for action in actions.iter() {
                let starting_board = board.clone();
                action.apply(&mut board);
                action.undo(&mut board);
                assert_eq!(starting_board, board);
            }
        }
    }

    #[test]
    fn castle_duplication() {
        let bp =
            Blueprints::from_assets_location(&("../../".to_string() + BLUEPRINTS_PATH)).unwrap();
        let mut board = Board {
            bp: Arc::new(bp.clone()),
            grid: parse_map(
                &bp,
                r"
--- --- ---
--- --- ---
--- --- ---",
            )
            .unwrap()
            .grid,
            players: vec![
                Player {
                    id: PlayerId::new(0),
                    color: 0x00900000,
                    symbol: "@".to_string(),
                    resources: Resources {
                        food: 1200,
                        gold: 1200,
                    },
                    ..Default::default()
                },
                Player {
                    id: PlayerId::new(1),
                    color: 0x00900090,
                    symbol: "!".to_string(),
                    resources: Resources {
                        food: 1200,
                        gold: 1200,
                    },
                    ..Default::default()
                },
            ],
            day: 0,
            current_player_turn: PlayerId::new(0),
            player_turn_order: vec![PlayerId::new(0)],
            fog: HashMap::new(),
            fog_base: FogTile::Visible,
        };

        let initial = board.clone();

        for hp in 0..=100 {
            board = initial.clone();

            let castle_id = bp.get_unit_from_name("Castle").unwrap();
            let castle_bp = bp.get_unit(&castle_id);
            let build_area = vec![v!(0, 1), v!(0, 2), v!(1, 1), v!(1, 2)];
            for build_spot in build_area.iter() {
                board.grid.get_at_mut(&build_spot).set_unit(
                    Some(Unit {
                        blueprint_id: castle_id.clone(),
                        owner: PlayerId::new(0),
                        linked_units: build_area.clone(),
                        health: hp,
                        ..Default::default()
                    }),
                    &castle_bp,
                );
            }

            let villager_id = bp.get_unit_from_name("Villager").unwrap();
            let villager_bp = bp.get_unit(&villager_id);
            board.grid.get_at_mut(&v!(1, 1)).set_unit(
                Some(Unit {
                    blueprint_id: villager_id.clone(),
                    owner: PlayerId::new(0),
                    ..Default::default()
                }),
                &villager_bp,
            );

            let unit_id = bp.get_unit_from_name("Longswordsmen").unwrap();
            let unit_bp = bp.get_unit(&unit_id);
            board.grid.get_at_mut(&v!(2, 1)).set_unit(
                Some(Unit {
                    blueprint_id: unit_id.clone(),
                    owner: PlayerId::new(1),
                    ..Default::default()
                }),
                &unit_bp,
            );
            board.grid.get_at_mut(&v!(1, 0)).set_unit(
                Some(Unit {
                    blueprint_id: unit_id.clone(),
                    owner: PlayerId::new(1),
                    ..Default::default()
                }),
                &unit_bp,
            );

            let attacker = board.grid.get_at(&v!(2, 1)).unit.clone().unwrap();
            let defender = board.grid.get_at(&v!(1, 1)).get_top_unit().unwrap().clone();
            let action_0 = PlayerAction::Unit {
                target: UnitTarget::new(attacker, v!(2, 1)),
                destination: v!(2, 1),
                action: UnitAction::Attack(UnitTarget::new(defender, v!(1, 1))),
                pickup: None,
                path: vec![],
            };

            let attacker = board.grid.get_at(&v!(1, 0)).unit.clone().unwrap();
            let defender = board.grid.get_at(&v!(1, 1)).get_top_unit().unwrap().clone();
            let action_1 = PlayerAction::Unit {
                target: UnitTarget::new(attacker, v!(1, 0)),
                destination: v!(1, 0),
                action: UnitAction::Attack(UnitTarget::new(defender, v!(1, 1))),
                pickup: None,
                path: vec![],
            };

            let previous_board = board.clone();

            action_0.apply(&mut board);
            action_0.undo(&mut board);

            assert_eq!(previous_board, board, "hp: {}", hp);

            action_1.apply(&mut board);
            action_1.undo(&mut board);

            assert_eq!(previous_board, board, "hp: {}", hp);
        }
    }
}
