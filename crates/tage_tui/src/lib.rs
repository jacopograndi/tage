use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Alignment, Size},
    prelude::*,
    widgets::*,
};
use renet::ClientId;
use std::{
    collections::HashMap,
    fs,
    io::{self, stdout},
    path::PathBuf,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tage_core::{get_assets_dir, prelude::*};
use tage_flow::StartFlow;
use tracing::{info, Level};
use tracing_subscriber::{self};

mod widgets;
use widgets::*;

mod navigator;
use navigator::*;

mod net;
use net::*;

mod settings;
use settings::*;

mod input;
use input::*;

mod member;
use member::*;

mod menu_main;
use menu_main::*;

mod menu_pause;
use menu_pause::*;

mod menu_settings;
use menu_settings::*;

mod menu_lobby;
use menu_lobby::*;

mod menu_load;
use menu_load::*;

mod menu_profile;
use menu_profile::*;

mod menu_connect;
use menu_connect::*;

pub fn game_main(flow: StartFlow) -> io::Result<()> {
    setup_tracing();
    initialize_panic_handler();

    let bp = Blueprints::from_assets().unwrap();

    let settings = Settings::from_disk().unwrap_or_default();

    let (mut game_state, mut interface_state) = match flow {
        StartFlow::Menu => (
            None,
            InterfaceState {
                main_menu: Some(MenuState::Home(MenuHome::new())),
                ..Default::default()
            },
        ),
        StartFlow::LocalNewMap {
            settings: map_settings,
        } => (
            Some(setup_gamestate(map_settings, &bp).unwrap()),
            InterfaceState::default(),
        ),
    };

    interface_state.settings = settings;

    if let Ok(grid) = load_map(
        &bp,
        &MapSettings::default().with_path(
            format!("{}/assets/riverland.txt", get_assets_dir())),
    ) {
        interface_state.background_board = Some(Board {
            grid,
            bp: Arc::new(bp.clone()),
            players: vec![],
            day: 0,
            current_player_turn: PlayerId::new(0),
            player_turn_order: vec![],
            fog: HashMap::new(),
            fog_base: FogTile::Visible,
        });
    }

    let mut net = Net::new();

    let mut terminal = init_terminal()?;

    // main loop
    let mut time_last = Instant::now();
    let mut is_running = true;
    while is_running {
        let now = Instant::now();
        let duration = now - time_last;
        time_last = now;

        net.update(duration);

        let mut event_list = vec![];
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Ok(event) = event::read() {
                event_list.push(event);
            }
        }

        input(
            &mut is_running,
            &event_list,
            &mut game_state,
            &mut interface_state,
            &bp,
            &mut net,
        )?;

        render(
            &mut terminal,
            &mut game_state,
            &mut interface_state,
            &bp,
            &net,
        )?;
    }

    restore_terminal()
}

fn get_data_dir() -> Option<PathBuf> {
    let mut path = dirs::data_dir()?;
    path.push("tage/");
    let _ = std::fs::create_dir_all(path.clone());
    Some(path)
}

fn get_data_dir_sub(sub: &str) -> Option<PathBuf> {
    let mut path = get_data_dir()?;
    path.push(format!("{}/", sub));
    let _ = std::fs::create_dir_all(path.clone());
    Some(path)
}

fn setup_tracing() {
    let Some(mut path) = get_data_dir_sub("traces") else {
        return;
    };

    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    path.push(format!("trace_{}.txt", epoch));

    let Ok(log_file) = std::fs::File::create(path.clone()) else {
        println!("Can't create trace file at {:?}", path);
        return;
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_writer(log_file)
        .with_max_level(Level::TRACE)
        .with_line_number(true)
        .with_ansi(false)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));
}

fn setup_gamestate(settings: MapSettings, bp: &Blueprints) -> Result<GameState, ParseMapError> {
    let mut board = Board {
        bp: Arc::new(bp.clone()),
        grid: load_map(&bp, &settings)?,
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
                civilization: player.civ(bp),
                controller: player.controller.clone(),
                ..Default::default()
            })
            .collect(),
        day: 0,
        current_player_turn: PlayerId::new(0),
        player_turn_order: settings
            .players
            .iter()
            .map(|player| player.id.clone())
            .collect(),
        fog: HashMap::new(),
        fog_base: settings.fog_base,
    };

    board.init_fog();
    board.refresh_fog();

    Ok(GameState {
        board,
        blueprints: bp.clone(),
        navigator: None,
        current_picker: UiPicker::Tile(UiTilePicker {
            cursor: IVec2::ZERO,
            unit: None,
            valid_tiles: vec![],
            selection_type: None,
        }),
        turn_timeline: vec![],
    })
}

fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

fn init_terminal() -> io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn input(
    is_running: &mut bool,
    event_list: &Vec<Event>,
    game_state: &mut Option<GameState>,
    ui_state: &mut InterfaceState,
    bp: &Blueprints,
    net: &mut Net,
) -> io::Result<()> {
    let menu_input = MenuInput::from_events(&ui_state.settings.keybinds, &event_list);
    let mut game_input = GameInput::from_events(&ui_state.settings.keybinds, &event_list);

    if let Some(pause_menu_state) = ui_state.pause_menu.take() {
        ui_state.pause_menu =
            pause_menu_state.input(menu_input, is_running, ui_state, game_state, bp, net);
        return Ok(());
    }

    if let Some(main_menu_state) = ui_state.main_menu.take() {
        ui_state.main_menu =
            main_menu_state.input(menu_input, is_running, ui_state, game_state, bp, net);
        return Ok(());
    };

    let Some(game_state) = game_state.as_mut() else {
        return Ok(());
    };

    let GameInput {
        quit,
        toggle_details,
        toggle_blueprints,
        toggle_map_dim,
        zoom,
        quicksave,
        quickload,
        toggle_pause_queue,
        step_queue,
        ..
    } = game_input;

    if quit {
        ui_state.pause_menu = Some(PauseMenuState::new());
        return Ok(());
    }

    if toggle_details {
        ui_state.show_details = !ui_state.show_details;
    }

    if toggle_blueprints {
        ui_state.show_blueprint = !ui_state.show_blueprint;
    }

    if toggle_map_dim {
        ui_state.map_only_player_color = !ui_state.map_only_player_color;
    }

    if zoom != 0 {
        ui_state.zoom = (ui_state.zoom + zoom).clamp(1, 5);
    }

    if quicksave {
        if let Some(mut path) = get_data_dir_sub("saves") {
            path.push(format!(
                "quicksave_{}.ron",
                std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ));
            let _ = game_state.board.save(path.as_path().to_str().unwrap());
        }
    }

    if quickload {
        if let Some(mut path) = get_data_dir_sub("saves") {
            if let Ok(paths) = fs::read_dir(path.clone()) {
                let dates: Vec<i64> = paths
                    .map(|path| path.unwrap().file_name().to_str().unwrap().to_string())
                    .filter(|filename| filename.starts_with("quicksave_"))
                    .map(|filename| filename.trim_start_matches("quicksave_").to_string())
                    .map(|filename| filename.trim_end_matches(".ron").to_string())
                    .filter_map(|timestamp_str| timestamp_str.parse().ok())
                    .collect();
                if let Some(lastest) = dates.iter().max() {
                    path.push(format!("quicksave_{}.ron", lastest));
                    if let Ok(board) = Board::load(bp, path.to_str().unwrap()) {
                        game_state.board = board;
                    }
                    ui_state.reset()
                }
            }
        }
    }

    if toggle_pause_queue {
        ui_state.queue_paused = !ui_state.queue_paused;
    }

    if game_input.select || game_input.back {
        ui_state.battle_state.temp_board = None;
    }

    let mut send_board = false;
    let mut send_to_lobby = false;

    if ui_state.winning_players.is_some() {
        let show_win = game_state.board.fog_base == FogTile::Visible;

        if show_win {
            if game_input.select {
                if net.is_server() {
                    net.server_send(&ServerMessages::ToLobby);
                    send_to_lobby = true;
                } else if !net.is_client() {
                    ui_state.main_menu = Some(MenuState::Home(MenuHome::new()));
                }
            }
            if ui_state.close_on_end {
                info!(target: "outcome", "winners {:?}", ui_state.winning_players);
                *is_running = false;
            }
        }
    }

    match &mut net.connection {
        Some(Connection::Client(client)) => {
            while let Some(message) = client.queue.pop() {
                match message {
                    ServerMessages::Board { board } => {
                        let board = board.to(&game_state.blueprints);
                        game_state.board = board;
                    }
                    ServerMessages::ToLobby => send_to_lobby = true,
                    _ => {}
                }
            }
        }
        Some(Connection::Server(server)) => {
            while let Some((_client_id, message)) = server.queue.pop() {
                match message {
                    ClientMessages::PlayerAction { action } => {
                        game_state.turn_timeline.push(action.clone());

                        let action = game_state.board.fog_bonk(action.clone());
                        action.apply(&mut game_state.board);
                        game_state.board.refresh_fog();

                        send_board = true;
                    }
                    ClientMessages::Undo => {
                        if let Some(last_action) = game_state.turn_timeline.pop() {
                            last_action.undo(&mut game_state.board)
                        }
                        send_board = true;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }

    match game_state.board.get_current_player().controller.clone() {
        Controller::Human => {
            if !net.is_client() {
                let num = game_state.turn_timeline.len();
                let action = game_state.apply_input(game_input);
                let undo = game_state.turn_timeline.len() < num;
                if net.is_server() && (action.is_some() || undo) {
                    send_board = true;
                }
            } else {
                // look around
                game_input = GameInput {
                    acc: game_input.acc,
                    ..Default::default()
                };
                game_state.apply_input(game_input);
            }
        }
        Controller::Remote(id) => {
            let client_id = ClientId::from_raw(id);
            if Some(client_id) == net.get_local_id() {
                if client_id == HOST_CLIENT_ID {
                    // host
                    // like Human
                    let num = game_state.turn_timeline.len();
                    let action = game_state.apply_input(game_input);
                    let undo = game_state.turn_timeline.len() < num;
                    if net.is_server() && (action.is_some() || undo) {
                        send_board = true;
                    }
                } else {
                    // remote local
                    // apply locally and send moves to server
                    let num = game_state.turn_timeline.len();
                    let action = game_state.apply_input(game_input);
                    let undo = game_state.turn_timeline.len() < num;
                    if net.is_client() {
                        if let Some(action) = action.clone() {
                            net.client_send(&ClientMessages::PlayerAction { action })
                        }
                        if undo && action.is_none() {
                            net.client_send(&ClientMessages::Undo)
                        }
                    }
                }
            } else {
                // look around
                game_input = GameInput {
                    acc: game_input.acc,
                    ..Default::default()
                };
                game_state.apply_input(game_input);
            }
        }
        Controller::Machine(machine) => {
            // look around
            game_input = GameInput {
                acc: game_input.acc,
                ..Default::default()
            };
            game_state.apply_input(game_input);

            if net.is_client() {
                return Ok(());
            }

            if ui_state.queue_paused && !step_queue {
                return Ok(());
            }

            if !ui_state.winning_players.is_some() {
                if ui_state.queued_actions.is_empty() {
                    ui_state.queued_actions =
                        get_machine_turn(&game_state.blueprints, &mut game_state.board, &machine);
                } else {
                    let mut do_step = false;
                    let speed = &ui_state.settings.machine_speed;
                    match speed {
                        MachineSpeed::Skip => {
                            for action in ui_state.queued_actions.iter() {
                                let action = game_state.board.fog_bonk(action.clone());
                                action.apply(&mut game_state.board);
                                game_state.board.refresh_fog();
                            }
                            ui_state.queued_actions.clear();
                            if net.is_server() {
                                send_board = true;
                            }
                        }
                        MachineSpeed::StepMovesSlow => {
                            ui_state.queue_delay += 1;
                            if ui_state.queue_delay > 10 {
                                do_step = true;
                                ui_state.queue_delay = 0;
                            }
                        }
                        MachineSpeed::StepMoves => {
                            do_step = true;
                        }
                        MachineSpeed::StepSelects => {
                            if let Some(_path) = ui_state.queued_path.as_mut() {
                                //todo
                                //game_state.apply_input(game_input);
                            } else {
                                let _action = ui_state.queued_actions.remove(0);
                                //todo construct select
                                //ui_state.play_selects =
                            }
                        }
                    }
                    if do_step {
                        let action = ui_state.queued_actions.remove(0);
                        let action = game_state.board.fog_bonk(action.clone());
                        action.apply(&mut game_state.board);
                        game_state.board.refresh_fog();

                        if let Some(cursor) = match &action {
                            PlayerAction::Unit { destination, .. } => Some(destination),
                            PlayerAction::Building {
                                target: UnitTarget { at, .. },
                                ..
                            } => Some(at),
                            _ => None,
                        } {
                            if let UiPicker::Tile(picker) = &mut game_state.current_picker {
                                picker.cursor = *cursor;
                            }
                        }

                        if net.is_server() {
                            send_board = true;
                        }
                    }
                }
            }
        }
    }

    if send_to_lobby {
        ui_state.main_menu = Some(MenuState::Lobby(MenuLobby::new(ui_state, net)));
    }

    if send_board && net.is_server() {
        for (id, _) in net
            .get_members()
            .iter()
            .filter(|(id, _)| id != &&HOST_CLIENT_ID)
        {
            if let Some(player) = game_state
                .board
                .players
                .iter()
                .find(|p| match p.controller {
                    Controller::Remote(raw_id) if raw_id == id.raw() => true,
                    _ => false,
                })
            {
                let stripped = game_state.board.strip_fog(&player.id);
                net.server_send_at(
                    id.clone(),
                    &ServerMessages::Board {
                        board: BoardView::from(&game_state.blueprints, &stripped),
                    },
                )
            }
        }
    }

    ui_state.winning_players = game_state.board.get_winners();

    Ok(())
}

fn render(
    terminal: &mut Terminal<impl Backend>,
    game_state: &Option<GameState>,
    ui_state: &mut InterfaceState,
    bp: &Blueprints,
    net: &Net,
) -> io::Result<()> {
    if let Some(main_menu_state) = ui_state.main_menu.as_ref() {
        terminal.draw(|frame| main_menu_state.render(frame, bp, net, &ui_state))?;
        return Ok(());
    };

    let Some(game_state) = game_state.as_ref() else {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new("How did you even got in this state?"),
                frame.size(),
            )
        })?;
        return Ok(());
    };

    terminal.draw(|frame| {
        use Constraint::*;

        let [topbar, area] = Layout::vertical([Length(1), Min(0)]).areas(frame.size());

        let player = game_state.board.get_current_player();
        let player_text = format!(
            "Day {}; Player {} ({}), {}, {}",
            game_state.board.day + 1,
            player.name,
            player.symbol,
            view_level(player.level),
            game_state
                .board
                .bp
                .get_civilization(&player.civilization)
                .name
        );

        let [day_player, _, resources] =
            Layout::horizontal([Length(player_text.len() as u16 + 1), Min(0), Max(30)])
                .areas(topbar);
        frame.render_widget(
            Paragraph::new(player_text).style(
                Style::default()
                    .bg(Color::from_u32(player.color))
                    .fg(Color::Black),
            ),
            day_player,
        );
        let resource_layout = Layout::horizontal([Fill(1), Fill(1)]).split(resources);
        for (resource, layout) in Resource::iter().zip(resource_layout.iter()) {
            frame.render_widget(
                Paragraph::new(format!(
                    " {}: {} ",
                    resource.view(),
                    player.get_resource(resource)
                ))
                .style(
                    Style::default()
                        .bg(resource_color(resource))
                        .fg(Color::Black),
                ),
                *layout,
            );
        }

        // always show top tile picker
        let top_tile_picker = game_state.top_tile_picker();
        let (mut movement, mut attack, mut target) = (vec![], vec![], vec![]);
        if let Some(sel) = top_tile_picker.selection_type {
            match sel {
                TileSelectionType::Movement => movement = top_tile_picker.valid_tiles,
                TileSelectionType::Attack => attack = top_tile_picker.valid_tiles,
                TileSelectionType::Target => target = top_tile_picker.valid_tiles,
            }
        }

        let board: &Board = if let Some(nav) = &game_state.navigator {
            &nav.stack
                .last()
                .expect("stack is nonempty by construction")
                .preview_board
        } else {
            &game_state.board
        };

        if let UiPicker::Menu(menu_picker) = &game_state.current_picker {
            match menu_picker.choices.get(menu_picker.cursor) {
                Some(MenuChoice::BuildArea(area)) => {
                    target = area.iter().collect();
                }
                _ => {}
            }
        }

        let rest = if ui_state.show_blueprint {
            let [details, rest] =
                Layout::horizontal([Max(UnitBlueprintWidget::WIDTH), Fill(1)]).areas(area);
            frame.render_widget(Clear::default(), details);
            frame.render_widget(
                Block::bordered().border_type(BorderType::QuadrantOutside),
                details,
            );
            let [inner_details] = Layout::default()
                .constraints([Min(0)])
                .margin(1)
                .areas(details);

            let tile = board.grid.get_at(&top_tile_picker.cursor);
            if let Some(unit) = &tile.get_top_unit() {
                frame.render_widget(
                    UnitBlueprintWidget {
                        blueprints: &game_state.blueprints,
                        id: &unit.blueprint_id,
                    },
                    inner_details,
                );
            }
            rest
        } else {
            area
        };

        let rest = if ui_state.show_details {
            let [rest, details] =
                Layout::horizontal([Fill(1), Max(UnitWidget::RECT.width + 2)]).areas(rest);
            frame.render_widget(Clear::default(), details);
            frame.render_widget(
                Block::bordered().border_type(BorderType::QuadrantOutside),
                details,
            );
            let [details] = Layout::default()
                .constraints([Min(0)])
                .margin(1)
                .areas(details);
            let tile = board.grid.get_at(&top_tile_picker.cursor);
            let [_, det_unit, det_building, det_terrain] = Layout::vertical([
                Fill(1),
                Max(UnitWidget::RECT.height),
                Max(UnitWidget::RECT.height),
                Max(TerrainWidget::RECT.height),
            ])
            .areas(details);
            if let Some(unit) = &tile.unit {
                frame.render_widget(
                    UnitWidget {
                        board,
                        blueprints: &game_state.blueprints,
                        unit,
                        bonus: &board.get_unit_total_bonus(&UnitTarget::new(
                            unit.clone(),
                            top_tile_picker.cursor,
                        )),
                    },
                    det_unit,
                );
            } else {
                frame.render_widget(PanelWidget::new(DECOR_2), det_unit);
                frame.render_widget(
                    Paragraph::new("No unit").alignment(Alignment::Center),
                    det_unit,
                );
            }
            if let Some(building) = &tile.building {
                frame.render_widget(
                    UnitWidget {
                        board,
                        blueprints: &game_state.blueprints,
                        unit: building,
                        bonus: &board.get_unit_total_bonus(&UnitTarget::new(
                            building.clone(),
                            top_tile_picker.cursor,
                        )),
                    },
                    det_building,
                );
            } else {
                frame.render_widget(PanelWidget::new(DECOR_2), det_building);
                frame.render_widget(
                    Paragraph::new("No building").alignment(Alignment::Center),
                    det_building,
                );
            }
            frame.render_widget(
                TerrainWidget {
                    blueprints: &game_state.blueprints,
                    terrain: &tile.terrain,
                },
                det_terrain,
            );
            rest
        } else {
            rest
        };

        let travel_path: Vec<IVec2> = game_state
            .navigator
            .as_ref()
            .filter(|nav| nav.picked().last() == Some(&Select::Menu(MenuChoice::Move)))
            .iter()
            .find_map(|nav| {
                nav.star()
                    .iter()
                    .find(|node| match node.select {
                        Select::Tile(xy) if xy == top_tile_picker.cursor => true,
                        _ => false,
                    })
                    .iter()
                    .find_map(|node| match &node.action {
                        PlayerAction::Unit {
                            path, destination, ..
                        } => {
                            let mut path = path.clone();
                            path.push(*destination);
                            Some(path)
                        }
                        _ => None,
                    })
            })
            .unwrap_or_default();

        let fog_player = if net.connection.is_some() {
            if let Some(client_id) = net.get_local_id() {
                board
                    .players
                    .iter()
                    .find_map(|p| match p.controller {
                        Controller::Remote(raw_id) if raw_id == client_id.raw() => Some(&p.id),
                        _ => None,
                    })
                    .unwrap_or(&board.current_player_turn)
            } else {
                &board.current_player_turn
            }
        } else {
            &board.current_player_turn
        };

        frame.render_widget(
            BoardWidget {
                board,
                blueprints: &game_state.blueprints,
                cursor: top_tile_picker.cursor,
                movement_tiles: &movement,
                attack_tiles: &attack,
                target_tiles: &target,
                only_player_color: ui_state.map_only_player_color,
                zoom: ui_state.zoom,
                show_spawns: false,
                travel_path: &travel_path,
                fog_player,
            },
            rest,
        );

        if let Some(nav) = &game_state.navigator {
            let top = nav.stack.last().expect("stack is nonempty by construction");
            if let UiPicker::Confirm = game_state.current_picker {
                match top.selected_action.as_ref() {
                    Some(PlayerAction::Unit {
                        action: UnitAction::Attack(UnitTarget { at: def_pos, .. }),
                        destination,
                        ..
                    }) => {
                        if ui_state.battle_state.temp_board.is_none() {
                            let mut temp_board = game_state.board.clone();
                            let top_action = &top.selected_action.as_ref().unwrap();
                            top_action.apply(&mut temp_board);
                            ui_state.battle_state.temp_board = Some(temp_board);
                        };
                        let [_, inner, _] =
                            Layout::vertical([Fill(1), Min(17), Fill(1)]).areas(rest);
                        let [_, inner, _] =
                            Layout::horizontal([Fill(1), Min(60), Fill(1)]).areas(inner);
                        frame.render_stateful_widget(
                            BattleWidget {
                                board,
                                blueprints: &game_state.blueprints,
                                atk_pos_moved: destination.clone(),
                                def_pos: def_pos.clone(),
                            },
                            inner,
                            &mut ui_state.battle_state,
                        );
                    }
                    Some(PlayerAction::Unit {
                        action: UnitAction::Power(id, targets),
                        ..
                    }) => {
                        let [_, inner] = Layout::vertical([Fill(1), Length(13)]).areas(rest);
                        let [_, inner, _] =
                            Layout::horizontal([Length(1), Min(40), Length(1)]).areas(inner);
                        frame.render_widget(Clear::default(), inner);
                        frame.render_widget(
                            Block::bordered().border_type(BorderType::QuadrantOutside),
                            inner,
                        );
                        let [inner] = Layout::default()
                            .constraints([Min(0)])
                            .margin(1)
                            .areas(inner);
                        let bp = game_state.blueprints.get_power(id);
                        let [name, descr, ok] =
                            Layout::vertical([Length(1), Fill(1), Length(1)]).areas(inner);
                        frame.render_widget(Paragraph::new(format!("|{}|", bp.name)), name);
                        let mut description = String::new();
                        if bp.require_on_building != UnitConstraint::NoConstraint {
                            description += &format!(
                                "Requires that this unit to be on {}\n",
                                bp.require_on_building.view(&game_state.blueprints)
                            );
                        }
                        if bp.targets != PowerTargets::default() {
                            description += &format!("Targets {}", bp.targets.view());
                            description += &format!("Current targets: ");
                            if targets.len() < 5 {
                                for target in targets {
                                    description +=
                                        &format!("{}, ", target.view(game_state.board.bp()));
                                }
                            } else if target.len() <= 20 {
                                for target in targets {
                                    description += &format!("{}, ", target.at);
                                }
                            } else {
                                description += &format!("more than 20 units");
                            }
                            description += &format!("\n");
                        }
                        if bp.bonus != Bonus::default() {
                            description += &format!(
                                "\nThe targeted units gain:\n{}",
                                bp.bonus.view(&game_state.blueprints)
                            );
                        }
                        if bp.unit_bonus != UnitBonus::default() {
                            description += &format!(
                                "\nThe targeted units gets:\n{}",
                                bp.unit_bonus.view(&game_state.blueprints)
                            );
                        }
                        if bp.battle_bonus != BattleBonus::default() {
                            description += &format!(
                                "\nThe targeted units obtain:\n{}",
                                &bp.battle_bonus.view(&game_state.blueprints)
                            );
                        }
                        if bp.effects != PowerBlueprint::default().effects {
                            description += &format!("\n");
                            for effect in bp.effects.iter() {
                                description += &(format!("Effect: {}", effect.view() + "\n"));
                            }
                        }
                        frame.render_widget(
                            Paragraph::new(
                                description
                                    .split("\n")
                                    .map(|s| Line::raw(s))
                                    .collect::<Vec<Line>>(),
                            )
                            .wrap(Wrap { trim: false }),
                            descr,
                        );
                        frame.render_widget(
                            Paragraph::new("Use power")
                                .style(Style::default().add_modifier(Modifier::REVERSED)),
                            ok,
                        );
                    }
                    Some(PlayerAction::PassTurn) => {
                        let inner = popup(frame, rest, Size::new(40, 8));

                        let [title, _, tech, _, ok] =
                            Layout::vertical([Length(1), Length(1), Length(1), Fill(1), Length(1)])
                                .areas(inner);
                        frame.render_widget(Paragraph::new("End Turn"), title);
                        let player = game_state.board.get_current_player();
                        if let Some(research) = &player.research_queued {
                            let text = match research {
                                QueuedResearch::Tech(id) => {
                                    game_state.blueprints.get_tech(id).name.clone()
                                }
                                QueuedResearch::AgeUp => view_level(player.level + 1),
                            };
                            frame.render_widget(
                                Paragraph::new(format!("Researching {}", text)),
                                tech,
                            );
                        } else {
                            frame.render_widget(Paragraph::new("No research in progress!"), tech);
                        }

                        frame.render_widget(
                            Button {
                                string: "Advance",
                                pressed: true,
                            },
                            ok,
                        )
                    }
                    Some(PlayerAction::Building {
                        action: BuildingAction::AgeUp,
                        ..
                    }) => {
                        let inner = popup(frame, rest, Size::new(40, 8));
                        let [title, _, names, descr, ok] =
                            Layout::vertical([Length(1), Length(1), Length(1), Fill(1), Length(1)])
                                .areas(inner);
                        frame.render_widget(Paragraph::new("Age Up"), title);
                        let cost = board.get_current_player().get_age_up_cost();

                        frame.render_widget(
                            Paragraph::new(format!(
                                "From {} to {}",
                                view_level(player.level),
                                view_level(player.level + 1),
                            )),
                            names,
                        );
                        frame.render_widget(
                            Paragraph::new(format!("Cost: {} food {} gold", cost.food, cost.gold)),
                            descr,
                        );
                        frame.render_widget(
                            Button {
                                string: "Advance",
                                pressed: true,
                            },
                            ok,
                        )
                    }
                    _ => {
                        let inner = popup(frame, rest, Size::new(40, 8));
                        let [title, _, descr] =
                            Layout::vertical([Length(1), Fill(1), Length(1)]).areas(inner);
                        frame.render_widget(Paragraph::new("Confirm?"), title);
                        frame.render_widget(
                            Button {
                                string: "Yes",
                                pressed: true,
                            },
                            descr,
                        )
                    }
                }
            }

            if let UiPicker::Tech(tech_picker) = &game_state.current_picker {
                frame.render_widget(
                    ResearchWidget {
                        board,
                        blueprints: &game_state.blueprints,
                        tech_picker,
                    },
                    area,
                );
            }
            if let UiPicker::Menu(menu_picker) = &game_state.current_picker {
                if match menu_picker.choices.first() {
                    Some(MenuChoice::BuildArea(_)) => true,
                    _ => false,
                } {
                    // do not display menu when choosing area
                } else if !menu_picker.choices.is_empty() {
                    let [_, menu_bar] =
                        Layout::vertical([Min(0), Length(menu_picker.choices.len() as u16 + 2)])
                            .areas(rest);
                    let max_len = menu_picker
                        .choices
                        .iter()
                        .map(|choice| choice.view(&game_state.board).len())
                        .max()
                        .unwrap() as u16;
                    let max_len = max_len.max(32);
                    let [_, menu_bar, _] =
                        Layout::horizontal([Fill(1), Length(max_len + 2), Fill(1)]).areas(menu_bar);
                    frame.render_widget(Clear::default(), menu_bar);
                    frame.render_widget(
                        Block::bordered().border_type(BorderType::QuadrantOutside),
                        menu_bar,
                    );
                    let [menu] = Layout::default()
                        .constraints([Min(0)])
                        .margin(1)
                        .areas(menu_bar);
                    let mut state = TableState::new().with_selected(Some(menu_picker.cursor));
                    frame.render_stateful_widget(
                        Table::new(
                            menu_picker.choices.iter().map(|c| {
                                Row::new(vec![Cell::new(
                                    Line::from(c.view(&game_state.board))
                                        .alignment(Alignment::Left),
                                )])
                            }),
                            [max_len],
                        )
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
                        menu,
                        &mut state,
                    );
                }
            }
        }

        if !net.is_client() || board.fog_base == FogTile::Visible {
            if let Some(winners) = &ui_state.winning_players {
                let inner = popup(frame, rest, Size::new(40, 18));

                let [header, body] = Layout::vertical([Length(1), Fill(1)]).areas(inner);
                frame.render_widget(
                    Paragraph::new("Winners:").alignment(Alignment::Center),
                    header,
                );
                frame.render_widget(
                    Table::new(
                        game_state
                            .board
                            .players
                            .iter()
                            .filter_map(|player| {
                                let style = Style::default()
                                    .bg(Color::from_u32(player.color))
                                    .fg(Color::Black);
                                winners.contains(&player.id).then(|| {
                                    Row::new(vec![
                                        Line::from(player.symbol.clone()).style(style),
                                        Line::from(
                                            game_state
                                                .blueprints
                                                .get_civilization(&player.civilization)
                                                .name
                                                .clone(),
                                        ),
                                    ])
                                    .style(style)
                                })
                            })
                            .collect::<Vec<Row>>(),
                        [1, 20],
                    ),
                    body,
                )
            }
        }

        //todo: step_queue ui
        if ui_state.queue_paused {
            let [_, inner] = Layout::vertical([Fill(1), Length(1)]).areas(rest);
            frame.render_widget(Clear::default(), inner);
            frame.render_widget(
                Paragraph::new(format!(
                    "Machine paused. {:?} to unpause, {:?} to step",
                    ui_state.settings.keybinds.pause_queue, ui_state.settings.keybinds.step_queue
                ))
                .alignment(Alignment::Center)
                .fg(Color::Black)
                .bg(Color::Yellow),
                inner,
            );
        }

        if let Some(pause_menu_state) = ui_state.pause_menu.as_ref() {
            pause_menu_state.render(frame, game_state, bp, net, &ui_state.settings);
        }
    })?;

    Ok(())
}

#[derive(Debug, Clone)]
struct InterfaceState {
    settings: Settings,
    show_blueprint: bool,
    show_details: bool,
    map_only_player_color: bool,
    battle_state: BattleWidgetState,
    winning_players: Option<Vec<PlayerId>>,
    queued_actions: Vec<PlayerAction>,
    queued_path: Option<Path>,
    queue_paused: bool,
    queue_delay: i32,
    zoom: i32,
    main_menu: Option<MenuState>,
    pause_menu: Option<PauseMenuState>,
    close_on_end: bool,
    member_profile: Member,
    background_board: Option<Board>,
}

impl Default for InterfaceState {
    fn default() -> Self {
        Self {
            show_blueprint: false,
            show_details: true,
            battle_state: BattleWidgetState::default(),
            map_only_player_color: false,
            settings: Settings {
                keybinds: Keybinds::default(),
                machine_speed: MachineSpeed::StepMovesSlow,
            },
            winning_players: None,
            queued_actions: vec![],
            queued_path: None,
            queue_delay: 0,
            zoom: 3,
            main_menu: None,
            pause_menu: None,
            queue_paused: false,
            close_on_end: false,
            member_profile: Member::default(),
            background_board: None,
        }
    }
}

impl InterfaceState {
    fn reset(&mut self) {
        self.battle_state = BattleWidgetState::default();
        self.winning_players = None;
        self.queued_actions.clear();
        self.queued_path = None;
        self.queue_paused = false;
    }
}

fn bordered(frame: &mut Frame, area: Rect) -> Rect {
    use Constraint::*;
    frame.render_widget(Clear::default(), area);
    frame.render_widget(
        Block::bordered().border_type(BorderType::QuadrantOutside),
        area,
    );
    let [inner] = Layout::default()
        .constraints([Min(0)])
        .margin(1)
        .areas(area);
    inner
}

fn popup(frame: &mut Frame, area: Rect, size: Size) -> Rect {
    use Constraint::*;
    let [_, inner, _] = Layout::vertical([Fill(1), Max(size.height), Fill(1)]).areas(area);
    let [_, inner, _] = Layout::horizontal([Fill(1), Max(size.width), Fill(1)]).areas(inner);
    let inner = bordered(frame, inner);
    inner
}
