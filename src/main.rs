#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


pub mod bioreactor;

use eframe::egui;
use egui::Color32;
use egui_plot::{Legend, Line, Plot, PlotPoints};

use bioreactor::{State, Bioreactor};

const FEED_RATE: f64 = 0.03;
const VOLUME: f64 = 100.; // L

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

struct MyApp {
    mu: f64,
    n_vcd: f64,
    feed_min: f64,
    step:f64,
    ks_gluc: f64,
    k_gluc: f64,
    gluc_feed: f64,
    ks_glut: f64,
    k_glut: f64,
    glut_feed: f64,
    k_do: f64,
    i_v: f64,
    i_vcd :f64,
    i_gluc: f64,
    i_glut: f64,
    i_do: f64,
    g_v:Vec<[f64; 2]>,
    g_vcd:Vec<[f64; 2]>,
    g_gluc:Vec<[f64; 2]>,
    g_glut:Vec<[f64; 2]>,
    g_do:Vec<[f64; 2]>,
}

impl Default  for MyApp {
    fn default() -> Self {
        Self {
            mu: 0.001,
            n_vcd: 0.7,
            feed_min: 2., // feed start on day 2
            step: 1.,
            ks_gluc: 0.05,
            k_gluc: 0.0001,
            gluc_feed: 12.,
            ks_glut: 0.05,
            k_glut: 0.0001,
            glut_feed: 7.,
            k_do: 0.6,
            i_v: 45.,
            i_vcd: 0.5,
            i_gluc: 12.,
            i_glut: 7.,
            i_do: 0.9,
            g_v: vec![[0., 0.]],
            g_vcd: vec![[0., 0.]],
            g_gluc: vec![[0., 0.]],
            g_glut: vec![[0., 0.]],
            g_do: vec![[0., 0.]],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let Self { mu, n_vcd, feed_min, step, ks_gluc, k_gluc, gluc_feed, ks_glut, k_glut, glut_feed, k_do, i_v, i_vcd, i_gluc, i_glut, i_do, g_v, g_vcd, g_gluc, g_glut, g_do} = self;

        egui::SidePanel::left("options").show(ctx, |ui| {

            ui.add(egui::Slider::new(mu, 0.0..=120.).text("mu"));
            ui.add(egui::Slider::new(n_vcd, 0.0..=120.).text("n_vcd"));
            //ui.add(egui::Slider::new(fi_v, 0.0..=120.).text("fi V"));
            ui.add(egui::Slider::new(feed_min, 0.0..=14.).text("feed min"));
            ui.add(egui::Slider::new(step, 0.001..=1.).text("Step value").logarithmic(true));
            ui.add(egui::Slider::new(k_do, 0.0..=120.).text("K DO"));

            ui.collapsing("Initial conditions", |ui| {
                ui.add(egui::Slider::new(i_v, 0.0..=120.).text("V"));
                ui.add(egui::Slider::new(i_vcd, 0.0..=120.).text("VCD"));
                ui.add(egui::Slider::new(i_gluc, 0.0..=120.).text("Gluc"));
                ui.add(egui::Slider::new(i_glut, 0.0..=120.).text("Glut"));
                ui.add(egui::Slider::new(i_do, 0.0..=120.).text("DO"));
            });
            ui.collapsing("Glucose", |ui| {
                ui.add(egui::Slider::new(ks_gluc, 0.0..=120.).text("KS"));
                ui.add(egui::Slider::new(k_gluc, 0.0..=120.).text("K"));
                ui.add(egui::Slider::new(gluc_feed, 0.0..=120.).text("feed"));
            });

            ui.collapsing("Glutamin", |ui| {
                ui.add(egui::Slider::new(ks_glut, 0.0..=120.).text("KS"));
                ui.add(egui::Slider::new(k_glut, 0.0..=120.).text("K"));
                ui.add(egui::Slider::new(glut_feed, 0.0..=120.).text("feed"));
            });


                // Days * 24h * 60mns
                const MINUTES: f64 = 14. * 24. * 60.;
                let fi_v: f64 = VOLUME * FEED_RATE / (24.* 60.);
                
                let init_cond = State::new(*i_v, *i_vcd, *i_gluc, *i_glut, *i_do);
                let system = Bioreactor::new(*mu, *n_vcd, *ks_gluc, *k_gluc, *gluc_feed, *k_do, *ks_glut, *k_glut, *glut_feed, *feed_min, fi_v);
                let mut stepper = ode_solvers::Rk4::new(system, 0., init_cond, MINUTES, *step);
                let res = stepper.integrate();
                if let Ok(_val) = res {
                    *g_v = Vec::new();
                    *g_vcd = Vec::new();
                    *g_gluc = Vec::new();
                    *g_glut = Vec::new();
                    *g_do = Vec::new();
                    for (t,y) in stepper.x_out().iter().zip(stepper.y_out()) {
                        g_v.push([*t, y[0] ]);
                        g_vcd.push([*t, y[1] ]);
                        g_gluc.push([*t, y[2] ]);
                        g_glut.push([*t, y[3] ]);
                        g_do.push([*t, y[4] ]);
                    }
                }
            //}
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let my_plot = Plot::new("My Plot")
            .legend(Legend::default().position(egui_plot::Corner::LeftTop))
            .x_axis_formatter(|gm, _max_n, _rng| {
                let val = gm.value / (60. * 24.);

                format!("Day {:.2}", val)
            })
            .label_formatter(|name, value| {
                let mins = value.x;
                let days = mins / (24. * 60.);

                if !name.is_empty() {
                    format!("{name}\ny: {:.4}\nx: {:.4} days", value.y, days)
                } else {
                    format!("y: {:.4}\nx: {:.4} days", value.y, days)
                }
            })
            ;
            
            my_plot.show(ui, |plot_ui| {

                plot_ui.line(
                    Line::new(PlotPoints::from(g_v.clone()))
                    .name("Volume")
                    .color(Color32::BLUE)
                );
                plot_ui.line(
                    Line::new(PlotPoints::from(g_vcd.clone()))
                    .name("VCD")
                    .color(Color32::RED)
                );
                plot_ui.line(
                    Line::new(PlotPoints::from(g_gluc.clone()))
                    .name("glucose")
                    .color(Color32::GREEN)
                );
                plot_ui.line(
                    Line::new(PlotPoints::from(g_glut.clone()))
                    .name("glutamin")
                    .color(Color32::YELLOW)
                );
                plot_ui.line(
                    Line::new(PlotPoints::from(g_do.clone()))
                    .name("DO")
                    .color(Color32::LIGHT_BLUE)
                );

            });



        });
    }
}
