use crate::app::tooltip::TooltipApp;

mod app;
mod data;
mod ui;
mod utils;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Warcraft Text Strings Translation Manager",
        native_options,
        Box::new(|cc| Ok(Box::new(TooltipApp::new(cc)))),
    )
}
