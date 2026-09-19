#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod backend;
mod config;
mod engine;
mod hotkeys;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Lion AutoClicker")
            .with_app_id("io.github.lionautoclicker.LionAutoclicker")
            .with_inner_size([820.0, 850.0])
            .with_min_inner_size([600.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Lion AutoClicker",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)?))),
    )
}
