#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Disable console window for release builds

use cross::Cross;
use eframe::egui;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
const TITLE: &str = "Pattern Creator";

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init(); // Enable with RUST_LOG=debug

    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some(egui::vec2(WIDTH, HEIGHT)),
        ..Default::default()
    };

    eframe::run_native(
        TITLE,
        options,
        Box::new(|_cc| Box::new(Cross::default())),
    )
}