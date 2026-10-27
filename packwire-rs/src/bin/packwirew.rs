#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    if packwire_rs::gui::launch_gui(false, 5050).is_err() {
        std::process::exit(1);
    }
}
