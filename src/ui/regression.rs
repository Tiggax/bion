use core::fmt;
use std::{fs, path::PathBuf};

use crate::{base::{Graphs, Initial}, model::VOLUME};
use ndarray::array;
use argmin::{core::{CostFunction, Executor}, solver::neldermead::NelderMead};

use egui::{CollapsingHeader, Color32, Ui, Vec2, Vec2b};
use egui_plot::{Line, Plot, PlotBounds, PlotPoints, PlotResponse, Points};

use crate::model::{Bioreactor, State};

use super::{tree::{Par, ParentNode, Tree}, Front};

#[derive(Debug, PartialEq, Eq)]
pub enum Group {
    VCD,
    Glucose,
    Glutamin,
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct RegressionApp {
    nodes: Tree,
    current_group: Group,
    initial: Initial,
    graphs: Graphs,
    selected_file: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct Record {
    days: Option<f64>,
    vcd: Option<f64>,
    gln: Option<f64>,
    gluc: Option<f64>,
    do_50: Option<f64>,
    product: Option<f64>
}

impl RegressionApp {
    pub fn default() -> RegressionApp {
        Self {
            nodes: Tree {
                nodes: vec![
                    ParentNode::new(Group::VCD.to_string()),
                    ParentNode::new(Group::Glucose.to_string()),
                    ParentNode::new(Group::Glutamin.to_string()),
                ],
            },
            current_group: Group::VCD,
            initial: Initial::default(),
            graphs: Graphs::default(),
            
            selected_file: None,
        }
    }
}

impl Front for RegressionApp {
    fn left_panel(&mut self, ui: &mut egui::Ui) {
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.nodes.ui(ui);

            ui.separator();
            ui.collapsing("Initial conditions", |ui| {
                ui.add(egui::Slider::new(&mut self.initial.vcd, 0.0..=120.).text("VCD"));
                ui.add(egui::Slider::new(&mut self.initial.gluc, 0.0..=120.).text("Gluc"));
                ui.add(egui::Slider::new(&mut self.initial.glut, 0.0..=120.).text("Glut"));
            });
            if ui.button("reset").clicked() {
                let newapp = RegressionApp::default();
                *self = newapp;
            }

            ui.separator();
            ui.label("input data");
            if (ui.button("Load data")).clicked() {

                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.selected_file = Some(path.display().to_string());
                    let file = 
                    if let Ok(content) = fs::read_to_string(path) {
                        let mut rdr = csv::Reader::from_reader(content.as_bytes());
                        for result in rdr.deserialize() {
                            if let Ok(res) = result {
                                let record: Record = res;
                                if let Some(day) = record.days {
                                    if let Some(vcd) = record.vcd {
                                        self.nodes.add("VCD".to_string(), day, vcd/100.);
                                    }
                                    if let Some(gln) = record.gln {
                                        self.nodes.add("Glutamin".to_string(), day, gln);
                                    }
                                    if let Some(gluc) = record.gluc {
                                        self.nodes.add("Glucose".to_string(), day, gluc);
                                    }
                                }
                            } else {
                            }
                        }
                    };
                };
            }
            
            if let Some(path) = &self.selected_file {
                ui.horizontal(|ui| {
                    ui.label("Selected file:");
                    ui.monospace(path.split("/").last().unwrap_or("None"));
                });
            }
            
            


            ui.separator();
            ui.label("Add constants");
            ui.separator();

            if ui.button("Minimize").clicked() {

                let cost = crate::regressor::Regressor {
                    initial: self.initial.clone(),
                    nodes: self.nodes.clone(),
                };
                println!("cost init");
            
                let solver = NelderMead::new(vec![
                    // vec![0.001, 0.7,j 0.05, 0.0001, 0.05, 0.0001],
                    
                    
                    //vec![0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.0001],
                    vec![0.99  , 0.0001, 0.0001, 0.0001, 0.0001, 0.0001],
                    vec![0.0001, 0.99  , 0.0001, 0.0001, 0.0001, 0.0001],
                    vec![0.0001, 0.0001, 0.99  , 0.0001, 0.0001, 0.0001],
                    vec![0.0001, 0.0001, 0.0001, 0.99  , 0.0001, 0.0001],
                    vec![0.0001, 0.0001, 0.0001, 0.0001, 0.99  , 0.0001],
                    vec![0.0001, 0.0001, 0.0001, 0.0001, 0.0001, 0.99  ],
                ])
                .with_sd_tolerance(0.00001).unwrap();

                println!("solver made");
                let res = Executor::new(cost, solver)
                    .configure(|state| state.max_iters(1000))
                    .run();
                
                println!("running");
                match res {
                    Ok(val) => {
                    // add to graphs
                    println!("OK:");
                    println!("{:?}", val.state);
                    println!("BEST: {:?}", val.state.best_param);
                    
                    if let Some(p) = val.state.best_param {
                        let mut p = p;
                        let k_glut = p.pop().expect("no k_glut");
                        let ks_glut = p.pop().expect("no ks_glut");
                        let k_gluc = p.pop().expect("no k_gluc");
                        let ks_gluc = p.pop().expect("no ks_gluc");
                        let n_vcd = p.pop().expect("no n_vcd");
                        let mu = p.pop().expect("no mu");
                        let Initial {
                            vcd,
                            gluc,
                            glut,
                        } = self.initial;
                        
                        const MINUTES: f64 = 14. * 24. * 60.;
                
                        let init_cond = State::from([VOLUME, vcd, gluc, glut, 80., 0., 0. ]);
                        let system = Bioreactor::fit(mu, ks_gluc, k_gluc, ks_glut, k_glut);
                        
                        let mut stepper = ode_solvers::Rk4::new(system, 0., init_cond, MINUTES, 1.);
                        let res = stepper.mut_integrate();


                        let res = stepper.mut_integrate();
                        if let Ok(_val) = res {
                            self.graphs.volume = Vec::new();
                            self.graphs.vcd = Vec::new();
                            self.graphs.glucose = Vec::new();
                            self.graphs.glutamin = Vec::new();
                            self.graphs.c_O2 = Vec::new();
                            self.graphs.O2 = Vec::new();
                            self.graphs.product = Vec::new();
                            for (t,y) in stepper.x_out().iter().zip(stepper.y_out()) {
                                self.graphs.volume.push([*t, y[0] ]);
                                self.graphs.vcd.push([*t, y[1] ]);
                                self.graphs.glucose.push([*t, y[2] ]);
                                self.graphs.glutamin.push([*t, y[3] ]);
                                self.graphs.c_O2.push([*t, y[4] ]);
                                self.graphs.O2.push([*t, y[5] ]);
                                self.graphs.product.push([*t, y[6] ]);
                            }
                        }
                    }

                },
                Err(er) => {
                    println!("oops: {:?}", er);
                }
                }
            }
        });
    }

    fn center_panel(&mut self, ui: &mut egui::Ui) {
        let my_plot = Plot::new("my_plot")
        //.allow_zoom(false)
        //.allow_boxed_zoom(false)
        //.allow_drag(false)
        //.allow_double_click_reset(false)
        //.allow_scroll(false)
        ;

        let plot_resp = my_plot.show(ui, |plot_ui| {
            //plot_ui.set_plot_bounds(PlotBounds::from_min_max([-1.,-5.], [20_160., 100.]));

            let mut plot_points = self.nodes.plot_points();
            
            // Glutamin
            let glut_points = plot_points.pop();
            if let Some(points) = glut_points {
                plot_ui.points(points
                    .radius(4.)
                    .color(Color32::YELLOW)
                );
            }
            plot_ui.line(Line::new(PlotPoints::from(self.graphs.glutamin.clone())).color(Color32::YELLOW).name("Glutamin"));

            // glucose
            let gluc_points = plot_points.pop();
            if let Some(points) = gluc_points {
                plot_ui.points(points
                    .radius(4.)
                    .color(Color32::GREEN)
                );
            }

            plot_ui.line(Line::new(PlotPoints::from(self.graphs.glucose.clone())).color(Color32::GREEN).name("Glucose"));

            // vcd
            let vcd_points = plot_points.pop();
            if let Some(points) = vcd_points {
                plot_ui.points(points
                    .radius(4.)
                    .color(Color32::RED)
                );
            }

            plot_ui.line(Line::new(PlotPoints::from(self.graphs.vcd.clone())).color(Color32::RED).name("VCD"));


            // draw the graph.

            plot_ui.line(Line::new(PlotPoints::from(self.graphs.volume.clone())).color(Color32::BLUE).name("Volume"));
            plot_ui.line(Line::new(PlotPoints::from(self.graphs.c_O2.clone())).color(Color32::LIGHT_BLUE).name("c_O2"));
            plot_ui.line(Line::new(PlotPoints::from(self.graphs.O2.clone())).color(Color32::WHITE).name("O2"));
            //plot_ui.line(Line::new(PlotPoints::from(self.graphs.product.clone())).color(Color32::GOLD).name("product"));

            


            (
                //plot_ui.screen_from_plot(PlotPoint::new(0.0, 0.0)),
                plot_ui.pointer_coordinate(),
                plot_ui.pointer_coordinate_drag_delta(),
                plot_ui.plot_bounds(),
                plot_ui.response().hovered(),
            )

        });


        let PlotResponse {
            response,
            inner: ( pointer_coordinate, pointer_coordinate_drag_delta, bounds, hovered),
            hovered_plot_item,
            ..
        } = plot_resp;

        response.context_menu(|ui| {
            ui.set_min_width(220.);

            let cart = |r:f64,a:f64| [r * a.cos(), r * a.sin()];
            let num = 3.;
            let ang = 360. / num;

            //let wid_rect = egui::Rect::from_min_size(ui.min_rect().min, Vec2::new(2., 2.));
            
            //ui.put(wid_rect, egui::RadioButton::new(false, "text"));
                
            if ui.radio_value(&mut self.current_group, Group::VCD, Group::VCD.to_string()).clicked()
                || ui.radio_value(&mut self.current_group, Group::Glucose, Group::Glucose.to_string()).clicked()
                || ui.radio_value(&mut self.current_group, Group::Glutamin, Group::Glutamin.to_string()).clicked() {
                    ui.close_menu();
                }
        });

        if response.clicked_by(egui::PointerButton::Middle) {
            if let Some(point) = hovered_plot_item {
                match point {
                    id if id == egui::Id::new(Group::VCD.to_string()) => {
                        if let Some(point) = pointer_coordinate {
                            self.nodes.remove(Group::VCD.to_string(), point.x, point.y);
                        }
                    },
                    id if id == egui::Id::new(Group::Glucose.to_string()) => {
                        if let Some(point) = pointer_coordinate {
                            self.nodes.remove(Group::Glucose.to_string(), point.x, point.y);
                        }
                    },
                    id if id == egui::Id::new(Group::Glutamin.to_string()) => {
                        if let Some(point) = pointer_coordinate {
                            self.nodes.remove(Group::Glutamin.to_string(), point.x, point.y);
                        }
                    },
                    _ => {}
                }
            }
        }

        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(point) = pointer_coordinate {
                self.nodes.add(self.current_group.to_string(), point.x, point.y);
            }
        }
    }
}