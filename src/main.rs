// prevent a command line window on Windows
#![windows_subsystem = "windows"]
mod actions;
mod config;
mod mounts;
mod node_color;
mod scan;
mod squarify;
mod types;
mod ui;
mod utils;

use std::path::PathBuf;

fn main() {
    dbg!(mounts::get_mounts());
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        ui::init(Some(PathBuf::from(&args[1])))
    } else {
        ui::init(None)
    }
    .expect("Failed to initiate UI");
}
