#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// FerrumCalc – Main Entry Point
///
/// Initializes the eframe application with window configuration
/// and launches the FerrumCalc calculator.

mod calculator;
mod icon;
mod ui;

use eframe::egui;
use ui::FerrumCalcApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("FerrumCalc")
            .with_icon(std::sync::Arc::new(icon::app_icon()))
            .with_inner_size([380.0, 600.0])
            .with_min_inner_size([320.0, 480.0])
            .with_max_inner_size([800.0, 1000.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FerrumCalc",
        options,
        Box::new(|cc| Ok(Box::new(FerrumCalcApp::new(cc)))),
    )
}
