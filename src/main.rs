#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use wts_translation_manager::app::tooltip::TooltipApp;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Warcraft Text Strings Translation Manager",
        native_options,
        Box::new(|cc| Ok(Box::new(TooltipApp::new(cc)))),
    )
}