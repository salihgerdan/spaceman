// prevent a command line window on Windows
#![windows_subsystem = "windows"]
mod bytes_display;
mod config;
mod filetree;
mod gui;
mod node_color;
mod squarify;

fn main() {
    gui::initiate_ui();
}
