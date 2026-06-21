// prevent a command line window on Windows
#![windows_subsystem = "windows"]
mod config;
mod mounts;
mod node_color;
mod scan;
mod squarify;
mod types;
mod ui;
mod utils;

fn main() {
    dbg!(mounts::get_mounts());
    ui::init().expect("Failed to initiate UI");
}
