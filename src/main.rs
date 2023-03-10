// prevent a command line window on Windows
#![windows_subsystem = "windows"]
mod config;
mod filetree;
mod gui;
mod squarify;

fn main() {
    gui::initiate_ui();
}
