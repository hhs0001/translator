// Release builds are a GUI app: no console window behind the window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    translator::run();
}
