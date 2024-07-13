#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#[allow(non_snake_case)]

pub mod ui;
pub mod model;
pub mod regressor;
pub mod base;

use eframe::egui;
use ui::{app::BionApp, Front};



fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| {
            Box::<BionApp>::default()
        }),
    )
}

impl eframe::App for BionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("options").show(ctx, |ui| { self.left_panel(ui, ctx)});     
        egui::CentralPanel::default().show( ctx, |ui|{ self.center_panel(ui, ctx) });
    }
}
