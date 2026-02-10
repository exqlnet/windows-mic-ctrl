#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    windows_mic_ctrl_lib::run();
}
