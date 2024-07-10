#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#[allow(non_snake_case)]

pub mod ui;
pub mod model;
pub mod regressor;
pub mod base;

use std::process::id;

use eframe::{egui, App};
use egui::{epaint::RectShape, CentralPanel, Color32, Id, LayerId, Rect, Stroke, Ui};
use egui_dock::{DockArea, DockState, Style};
use egui_plot::{HLine, Legend, Line, LineStyle, Plot, PlotPoints};
use ui::{regression::RegressionApp, simulations::{self, SimulationApp}, Front};



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

            Box::<MyApp>::default()
        }),
    )
}
enum Panel {
    SimulationApp(SimulationApp),
    RegressionApp(RegressionApp),
}
impl Default for Panel {
    fn default() -> Self {
        Panel::SimulationApp(SimulationApp::default())
    }
}

struct TabInstance {
    name: String,
    tab: Panel,
}

struct TabViewer {}

impl egui_dock::TabViewer for TabViewer {
    type Tab = TabInstance;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab.name).into()
    }
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {

        match &mut tab.tab {
            Panel::SimulationApp(t) => {t.show_inside(ui);},
            Panel::RegressionApp(t) => {t.show_inside(ui);},
        };

    }
    
}

struct MyApp {
    tree: DockState<TabInstance>
}

impl Default for MyApp {
    fn default() -> Self {

        let sim_tab = TabInstance {
            name: "Simulation".to_owned(),
            tab: Panel::SimulationApp(SimulationApp::default()),
        };

        let reg_tab = TabInstance {
            name: "Regretion".to_owned(),
            tab: Panel::RegressionApp(RegressionApp::default()),
        };

        let mut tree = DockState::new(vec![sim_tab, reg_tab]);

        Self {tree}
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        DockArea::new(&mut self.tree)
        .show_close_buttons(false)
        .style(Style::from_egui(ctx.style().as_ref()))
        .show( ctx, &mut TabViewer {});
    }
}