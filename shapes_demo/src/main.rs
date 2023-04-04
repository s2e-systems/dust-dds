#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod shapes_type;

fn main() -> Result<(), eframe::Error> {
    const ICON: &[u8] = include_bytes!("../res/logo.png");
    let icon = image::load_from_memory_with_format(ICON, image::ImageFormat::Png)
        .expect("Failed to open icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(500.0, 500.0)),
        default_theme: eframe::Theme::Light,
        icon_data: Some(eframe::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };
    eframe::run_native(
        "Dust DDS Shapes Demo",
        options,
        Box::new(|_cc| Box::new(app::MyApp::new())),
    )
}
