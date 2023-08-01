// prevent a command line window on Windows
#![windows_subsystem = "windows"]
mod config;
mod filetree;
mod gui;
mod mounts;
mod node_color;
mod squarify;
mod utils;

fn main() {
    dbg!(mounts::get_mounts());
    gui::initiate_ui();
}
