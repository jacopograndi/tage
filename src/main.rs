use tage_flow::StartFlow;

fn main() {
    let flow = StartFlow::from_args();

    // Other ui interfaces can be implemented.
    // Switch between them via feature flags.
    // Interfaces control the game loop, meaning that only one of them can be active.
    #[cfg(feature = "interface_tui")]
    {
        tage_tui::game_main(flow).unwrap();
    }
}
