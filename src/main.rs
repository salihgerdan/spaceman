// prevent a command line window on Windows
#![windows_subsystem = "windows"]
mod config;
mod filetree;
mod gui;

fn main() {
    gui::initiate_ui();
}
