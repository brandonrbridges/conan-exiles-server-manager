// Suppress the additional console window that ships with the Windows release build.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cesm_desktop_lib::run()
}
