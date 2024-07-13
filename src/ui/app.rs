use std::fs;

use argmin::{core::Executor, solver::neldermead::NelderMead};
use egui::Color32;
use egui_plot::{HLine, Legend, Line, LineStyle, Plot, PlotPoints};

use crate::{base::Graphs, model::{Bioreactor, State}, regressor::{Group, Mode, Param, Regressor, RegressorNode, Target}};

use super::{tree::{ParentNode, Tree}, Front};

#[derive(Debug, serde::Deserialize)]
struct Record {
    minutes: Option<f64>,
    vcd: Option<f64>,
    gln: Option<f64>,
    gluc: Option<f64>,
    do_50: Option<f64>,
    product: Option<f64>
}

#[derive(Debug)]
pub struct BionApp {
    sim: Bioreactor,
    old_sim: Option<Bioreactor>,
    sim_graphs: Graphs,
    graphs: Graphs,
    point_nodes: Tree,
    current_group: Group,
    selected_file: Option<String>,
    results: Option<String>,
    minimization_param: Param,
}

impl Default for BionApp {
    fn default() -> Self {
        Self {
            sim: Bioreactor::default(),
            old_sim: None,
            point_nodes: Tree {
                nodes: vec![
                    ParentNode::new(Group::VCD.to_string()),
                    ParentNode::new(Group::Glucose.to_string()),
                    ParentNode::new(Group::Glutamin.to_string()),
                ],
            },
            current_group: Group::VCD,
            sim_graphs: Graphs::default(),
            graphs: Graphs::default(),
            selected_file: None,
            results: None,
            minimization_param: Param::default(),
            
        }
    }
}


impl Front for BionApp {
    fn left_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut sim_changed = false;
            let mut last_state = self.sim.clone();


            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {  
                if let Some(previus_sim) = self.old_sim.clone() {
                    self.sim = previus_sim;
                    sim_changed = true;
                }
            }
        
            ui.horizontal(|ui| {
                let reset = ui.button("Reset").clicked();
                if ui.button("previus simulation").clicked() {
                    if let Some(previus_sim) = self.old_sim.clone() {
                        self.sim = previus_sim;
                        sim_changed = true;
                    }
                }
                if reset {
                    let new_app = BionApp::default();
                    let mut olds = self.old_sim.clone();
                    *self = new_app;
                    self.old_sim = olds;
                }
            });



            


            ui.separator();
            sim_changed = sim_changed || ui.collapsing("Simulation", |ui| {
                self.sim.view(ui)
            }).body_returned.unwrap_or(false);

            ui.separator();
            ui.collapsing("Regression", |ui| {

            });
            ui.label("input data");
            ui.horizontal(|ui|{
                if (ui.button("Load data")).clicked() {

                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.selected_file = Some(path.display().to_string());
                        let file = 
                        if let Ok(content) = fs::read_to_string(path) {
                            let mut rdr = csv::Reader::from_reader(content.as_bytes());
                            for result in rdr.deserialize() {
                                if let Ok(res) = result {
                                    let record: Record = res;
                                    if let Some(minute) = record.minutes {
                                        if let Some(vcd) = record.vcd {
                                            self.point_nodes.add("VCD".to_string(), minute, vcd);
                                        }
                                        if let Some(gln) = record.gln {
                                            self.point_nodes.add("Glutamin".to_string(), minute, gln);
                                        }
                                        if let Some(gluc) = record.gluc {
                                            self.point_nodes.add("Glucose".to_string(), minute, gluc);
                                        }
                                    }
                                }
                            }
                        };
                    };
                }

                if ui.button("Clear Nodes").clicked() {
                    self.point_nodes = Tree {
                        nodes: vec![
                            ParentNode::new(Group::VCD.to_string()),
                            ParentNode::new(Group::Glucose.to_string()),
                            ParentNode::new(Group::Glutamin.to_string()),
                        ],
                    };
                }
            });
            
            
            if let Some(path) = &self.selected_file {
                ui.horizontal(|ui| {
                    ui.label("Selected file:");
                    ui.monospace(path.split("/").last().unwrap_or("None"));
                });
            }
            ui.separator();
            ui.label("Minimization Target");
            ui.horizontal(|ui| {
                
                ui.selectable_value(&mut self.minimization_param.target, Target::MuMax, "mu max");
                ui.selectable_value(&mut self.minimization_param.target, Target::NVcd, "n_vcd");
                ui.selectable_value(&mut self.minimization_param.target, Target::FeedRate, "Feed rate");
                ui.selectable_value(&mut self.minimization_param.target, Target::Glucose, "Glucose");
                ui.selectable_value(&mut self.minimization_param.target, Target::Glutamin, "Glutamin");
            });
            ui.separator();

            ui.label("Minimization mode");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.minimization_param.mode, Mode::Mixed, "Mixed");
                ui.separator();
                //if let Mode::Single(_group) = &self.minimization_param.mode {
                ui.selectable_value(&mut self.minimization_param.mode, Mode::Single(Group::VCD), "VCD");
                ui.selectable_value(&mut self.minimization_param.mode, Mode::Single(Group::Glucose), "Glucose");
                ui.selectable_value(&mut self.minimization_param.mode, Mode::Single(Group::Glutamin), "Glutamin");
                    //}
            });
            ui.separator();




            if ui.button("Minimize").clicked() {

                self.results = Some("Calculating...".to_string());

                let cost = crate::regressor::Regressor {
                    nodes: RegressorNode::translate(self.point_nodes.clone()),
                    simulation: self.sim.clone(),
                    param: self.minimization_param.clone(),
                    epsilon: 1e-1,
                };

                let solver = NelderMead::new(vec![
                    1e-5,
                    0.99
                    ])
                .with_sd_tolerance(1e-5).unwrap();

                let res = Executor::new(cost, solver)
                .configure(|state| state.max_iters(1000))
                .run();

            
                
                let result = match res {
                    Ok(val) => {

                        if let Some(p) = val.state.best_param {
                            self.sim.update( &self.minimization_param, p);

                            println!("{:?}", self);

                        }
                        sim_changed = true;

                        format!("State: {:?}\n Best: {:?}", val.state, val.state.best_param)
                    },
                    Err(er) => {
                        format!("Something went wrong: \nState: {:?}", er )
                    }
                };

                self.results = Some(result);
            }
            if let Some(result) = &self.results {
                ui.label(result);
            }


            if sim_changed || self.sim_graphs.vcd.is_empty() {
                
                const MINUTES: f64 = 14. * 24. * 60.;

                const STEP: f64 = 2.; // step increment lower is more precise but more computationaly intense
                
                let initial_state = State::from([
                    self.sim.initial.volume, 
                    self.sim.initial.vcd, 
                    self.sim.initial.glucose, 
                    self.sim.initial.glutamin, 
                    (self.sim.initial.oxigen_part * self.sim.oxigen_saturation()) / 100., 
                    0., 
                    0. 
                ]);
                    self.old_sim = Some(last_state);
                let mut stepper = ode_solvers::Rk4::new(self.sim.clone(), 0., initial_state, MINUTES, STEP);

                let res = stepper.mut_integrate();
                if let Ok(_val) = res {

                    let Graphs { volume, vcd, glucose, glutamin, c_O2, O2, product } = &mut self.sim_graphs;

                    *volume = Vec::new();
                    *vcd = Vec::new();
                    *glucose = Vec::new();
                    *glutamin = Vec::new();
                    *c_O2 = Vec::new();
                    *O2 = Vec::new();
                    *product = Vec::new();
                    for (t,y) in stepper.x_out().iter().zip(stepper.y_out()) {

                        volume.push([*t, y[0] ]);
                        vcd.push([*t, y[1] ]);
                        glucose.push([*t, y[2] ]);
                        glutamin.push([*t, y[3] ]);
                        c_O2.push([*t, y[4] ]);
                        O2.push([*t, y[5] ]);
                        product.push([*t, y[6] ]);
                        
                    }
                }


            }

        });
    }

    fn center_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let my_plot = Plot::new("main_plot")
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

        let plot_resp = my_plot.show(ui, |plot_ui| {
            
            let mut plot_points = self.point_nodes.plot_points();
            // Glutamin
            let glut_points = plot_points.pop();
            if let Some(points) = glut_points {
                plot_ui.points(points
                    .radius(4.)
                    .color(Color32::YELLOW)
                );
            }
            plot_ui.line(Line::new(PlotPoints::from(self.graphs.glutamin.clone())).color(Color32::YELLOW).name("Glutamin Node"));

            // glucose
            let gluc_points = plot_points.pop();
            if let Some(points) = gluc_points {
                plot_ui.points(points
                    .radius(4.)
                    .color(Color32::GREEN)
                );
            }

            plot_ui.line(Line::new(PlotPoints::from(self.graphs.glucose.clone())).color(Color32::GREEN).name("Glucose Node"));

            // vcd
            let vcd_points = plot_points.pop();
            if let Some(points) = vcd_points {
                plot_ui.points(points
                    .radius(4.)
                    .color(Color32::RED)
                );
            }

            plot_ui.line(Line::new(PlotPoints::from(self.graphs.vcd.clone())).color(Color32::RED).name("VCD Node"));



            // ------------------- show sim -------------------
        
            plot_ui.line(
                Line::new(PlotPoints::from(self.sim_graphs.volume.clone()))
                .name("Volume")
                
                .color(Color32::BLUE)
            );
            plot_ui.line(
                Line::new(PlotPoints::from(self.sim_graphs.vcd.clone()))
                .name("VCD")
                
                .color(Color32::RED)
            );
            plot_ui.line(
                Line::new(PlotPoints::from(self.sim_graphs.glucose.clone()))
                .name("Glucose")
                
                .color(Color32::GREEN)
            );
            
            plot_ui.line(
                Line::new(PlotPoints::from(self.sim_graphs.glutamin.clone()))
                .name("Glutamin")
                
                .color(Color32::YELLOW)
            );

            let DO: Vec<[f64; 2]> = self.sim_graphs.c_O2.clone().into_iter().map(|[x,y]| {
                [x, (y / self.sim.oxigen_saturation()) * 100.]
            }).collect();

            plot_ui.line(
                Line::new(PlotPoints::from(DO))
                .name("c_O2")
                .color(Color32::WHITE)
            );

            plot_ui.line(
                Line::new(PlotPoints::from(self.sim_graphs.O2.clone()))
                .name("O2 input")
                
                .color(Color32::LIGHT_BLUE)
            );
            
            plot_ui.line(
                Line::new(PlotPoints::from(self.sim_graphs.product.clone()))
                .name("Product")
                
                .color(Color32::GOLD)
            );
            plot_ui.hline(
                HLine::new(self.sim.airation.pid.minimum.clone())
                .style(LineStyle::dashed_loose())
                .color(Color32::WHITE)
            );

        });
    }
}