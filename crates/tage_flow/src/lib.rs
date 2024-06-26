//! Entry point that parses the command line argument
//! and provides a shared way for interfaces to handle the start flow.

use std::fs;

use clap::Parser;
use tage_core::game::MapSettings;

pub enum StartFlow {
    Menu,
    LocalNewMap { settings: MapSettings },
}

impl StartFlow {
    pub fn from_args() -> StartFlow {
        FlowArgs::parse().into()
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct FlowArgs {
    /// Config path
    #[arg(short, long)]
    config: Option<String>,
}

impl From<FlowArgs> for StartFlow {
    fn from(value: FlowArgs) -> Self {
        if let Some(config_path) = value.config {
            let config_str = fs::read_to_string(&config_path).unwrap();
            StartFlow::LocalNewMap {
                settings: MapSettings::from_string(&config_str).unwrap(),
            }
        } else {
            StartFlow::Menu
        }
    }
}
