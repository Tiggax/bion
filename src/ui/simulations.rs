use egui::{Color32, Response, Ui};
use egui_plot::{HLine, Legend, Line, LineStyle, Plot, PlotPoints};

use crate::model::{Bioreactor, State, VOLUME, FEED_RATE};

use super::Front;


pub struct SimulationApp {
    name: String,

    k_product: f64,
    mu: f64,
    n_vcd: f64,
    feed_min: f64,
    step:f64,
    pzv: f64,
    henry: f64,
    ks_gluc: f64,
    k_gluc: f64,
    gluc_feed: f64,
    ks_glut: f64,
    k_glut: f64,
    glut_feed: f64,
    k_c_O2: f64,
    i_v: f64,
    i_vcd :f64,
    i_gluc: f64,
    i_glut: f64,
    i_c_O2: f64,
    i_O2_flow: f64,

    air_flow: f64,
    O2_min: f64,
    k_O2: f64,
    fi_O2_max: f64,
    O2_flow_max: f64,
    O2_feed_rate: f64,

    g_v:Vec<[f64; 2]>,
    g_vcd:Vec<[f64; 2]>,
    g_gluc:Vec<[f64; 2]>,
    g_glut:Vec<[f64; 2]>,
    g_c_O2:Vec<[f64; 2]>,
    g_O2:Vec<[f64; 2]>,
    g_product:Vec<[f64; 2]>,
}
impl Default  for SimulationApp {
    fn default() -> Self {
        let henry = 1.6066e-3; // mol / (L atm)
        Self {
            name: "Simulation".into(),
            k_product: 0.01,
            mu: 0.001,
            n_vcd: 0.7,
            feed_min: 2., // feed start on day 2
            step: 1.,
            pzv: 13.,
            henry, // mol / (L atm)
            ks_gluc: 0.05,
            k_gluc: 0.0001,
            gluc_feed: 12.,
            ks_glut: 0.05,
            k_glut: 0.0001,
            glut_feed: 7.,
            k_c_O2: 0.001,
            i_v: 45.,
            i_vcd: 0.5,
            i_gluc: 12.,
            i_glut: 7.,
            i_c_O2: 80.,
            i_O2_flow: 0.,

            air_flow: 0.5, // VVh
            O2_min: 25.,
            k_O2: 1.,
            fi_O2_max: 15.,
            O2_flow_max: 15.,
            O2_feed_rate: 1.9444,
            
            g_v: vec![[0., 0.]],
            g_vcd: vec![[0., 0.]],
            g_gluc: vec![[0., 0.]],
            g_glut: vec![[0., 0.]],
            g_c_O2: vec![[0., 0.]],
            g_O2: vec![[0., 0.]],
            g_product: vec![[0., 0.]],

        }
    }
}

impl Front for SimulationApp {
    
    fn left_panel(&mut self, ui: &mut Ui) {
        let rst = ui.button("Reset").clicked();
        
        if rst {
            let new = SimulationApp::default();
            *self = new;
            
        }

        let Self { name,k_product, mu, n_vcd, feed_min, step,pzv, henry, ks_gluc, k_gluc, gluc_feed, ks_glut, k_glut, glut_feed, k_c_O2, i_v, i_vcd, i_gluc, i_glut, i_c_O2, i_O2_flow, air_flow, O2_min, k_O2, fi_O2_max, O2_flow_max, O2_feed_rate,  g_v, g_vcd, g_gluc, g_glut, g_c_O2, g_O2, g_product} = self;

        let c_O2_SAT = *henry * 0.21;

        let mut something_changed = 
        ui.add(egui::Slider::new(mu, 0.0..=120.).text("mu")).changed() ||
        ui.add(egui::Slider::new(n_vcd, 0.0..=120.).text("n_vcd")).changed() ||
        ui.add(egui::Slider::new(pzv, 0.0..=50.).text("P/V")).changed() ||
        //ui.add(egui::Slider::new(fi_v, 0.0..=120.).text("fi V")).changed() ||
        ui.add(egui::Slider::new(feed_min, 0.0..=14.).text("feed min")).changed()
        //ui.add(egui::Slider::new(step, 0.001..=1.).text("Step value").logarithmic(true)).changed() ||
        ;
            
        ui.collapsing("Oxigen", |ui| {
            something_changed = something_changed || ui.add(egui::Slider::new(henry, 0.0..=10.).text("Henry's constant")).changed() ||
            ui.add(egui::Slider::new( O2_min, 0.0..=100.).text("Minimal level")).changed() ||
            ui.add(egui::Slider::new(O2_feed_rate, 0.0..=10.).text("Cell Feed rate [e-10]")).changed();
            ui.collapsing("Air", |ui| {
                something_changed = something_changed || ui.add(egui::Slider::new(air_flow, 0.0..=50.).text("Air Flow [VVh]")).changed();
            });
            ui.collapsing("Pure", |ui| {
                something_changed = something_changed || ui.add(egui::Slider::new(k_O2, 0.0..=200.).text("PID DO P parameter [K_o2]")).changed() ||
                ui.add(egui::Slider::new(fi_O2_max, 0.0..=50.).text("Constant [fi O2 max]")).changed() ||
                ui.add(egui::Slider::new(O2_flow_max, 0.0..=50.).text("O2  Flow max")).changed();
            });
            something_changed = something_changed || ui.add(egui::Slider::new(k_c_O2, 0.0..=1.).text("K DO")).changed();
        });

        ui.collapsing("Initial conditions", |ui| {
            something_changed = something_changed || ui.add(egui::Slider::new(i_v, 0.0..=120.).text("V")).changed() ||
            ui.add(egui::Slider::new(i_vcd, 0.0..=120.).text("VCD")).changed() ||
            ui.add(egui::Slider::new(i_gluc, 0.0..=120.).text("Gluc")).changed() ||
            ui.add(egui::Slider::new(i_glut, 0.0..=120.).text("Glut")).changed() ||
            ui.add(egui::Slider::new(i_c_O2, 0.0..=100.).text("DO")).changed() ||
            ui.add(egui::Slider::new(i_O2_flow, 0.0..=120.).text("O2 flow")).changed();
        });
        ui.collapsing("Glucose", |ui| {
            something_changed = something_changed || ui.add(egui::Slider::new(ks_gluc, 0.0..=120.).text("KS")).changed() ||
            ui.add(egui::Slider::new(k_gluc, 0.0..=120.).text("K")).changed() ||
            ui.add(egui::Slider::new(gluc_feed, 0.0..=120.).text("feed")).changed();
            let plt = Plot::new("ks_glucose");
        
            let points: Vec<[f64;2]> = (0..500).map(|i| {
                let x = f64::from(i) * 0.01;
                let rate = x / (x + *ks_gluc);
                [x,rate]
            }).collect();
        
            plt.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(points)))
            });
        });

        ui.collapsing("Glutamin", |ui| {
            something_changed = something_changed || ui.add(egui::Slider::new(ks_glut, 0.0..=120.).text("KS")).changed() ||
            ui.add(egui::Slider::new(k_glut, 0.0..=120.).text("K")).changed() ||
            ui.add(egui::Slider::new(glut_feed, 0.0..=120.).text("feed")).changed();
            let plt = Plot::new("ks_glutamin");
        
            let points: Vec<[f64;2]> = (0..500).map(|i| {
                let x = f64::from(i) * 0.01;
                let rate = x / (x + *ks_glut);
                [x,rate]
            }).collect();
        
            plt.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(points)))
            });
        });


        if something_changed || ui.button("update!").clicked() || rst {
            // Days * 24h * 60mns
            const MINUTES: f64 = 14. * 24. * 60.;
            let fi_v: f64 = VOLUME * FEED_RATE / (24.* 60.);
            let air_flow = *air_flow * *i_v;
            let i_DO = (*i_c_O2 * c_O2_SAT) / 100.;
            let init_cond = State::from([*i_v, *i_vcd, *i_gluc, *i_glut, i_DO, *i_O2_flow, 0. ]);
            let system = Bioreactor::new(*k_product, *mu, *n_vcd, *ks_gluc, *k_gluc, *gluc_feed, *k_c_O2, *ks_glut, *k_glut, *glut_feed, *feed_min, fi_v, *pzv * 1e-2, air_flow, *O2_min, *k_O2,  *henry, *fi_O2_max, *O2_flow_max, *O2_feed_rate);
            let mut stepper = ode_solvers::Rk4::new(system, 0., init_cond, MINUTES, *step);


            let res = stepper.mut_integrate();
            if let Ok(_val) = res {
                *g_v = Vec::new();
                *g_vcd = Vec::new();
                *g_gluc = Vec::new();
                *g_glut = Vec::new();
                *g_c_O2 = Vec::new();
                *g_O2 = Vec::new();
                *g_product = Vec::new();
                for (t,y) in stepper.x_out().iter().zip(stepper.y_out()) {
                    g_v.push([*t, y[0] ]);
                    g_vcd.push([*t, y[1] ]);
                    g_gluc.push([*t, y[2] ]);
                    g_glut.push([*t, y[3] ]);
                    g_c_O2.push([*t, y[4] ]);
                    g_O2.push([*t, y[5] ]);
                    g_product.push([*t, y[6] ]);
                    
                }
            }
        }         


    }

    fn center_panel(&mut self, ui: &mut Ui) {
        let Self { name, k_product, mu, n_vcd, feed_min, step,pzv, henry, ks_gluc, k_gluc, gluc_feed, ks_glut, k_glut, glut_feed, k_c_O2, i_v, i_vcd, i_gluc, i_glut, i_c_O2, i_O2_flow, air_flow, O2_min, k_O2, fi_O2_max, O2_flow_max, O2_feed_rate,  g_v, g_vcd, g_gluc, g_glut, g_c_O2, g_O2, g_product} = self;

        let c_O2_SAT = *henry * 0.21;

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

                let DO: Vec<[f64; 2]> = g_c_O2.clone().into_iter().map(|[x,y]| {
                    [x, (y / c_O2_SAT) * 100.]
                }).collect();

                plot_ui.line(
                    Line::new(PlotPoints::from(DO))
                    .name("c_O2")
                    .color(Color32::WHITE)
                );

                plot_ui.line(
                    Line::new(PlotPoints::from(g_O2.clone()))
                    .name("O2 input")
                    .color(Color32::LIGHT_BLUE)
                );
                
                plot_ui.line(
                    Line::new(PlotPoints::from(g_product.clone()))
                    .name("Product")
                    .color(Color32::GOLD)
                );
                plot_ui.hline(
                    HLine::new(O2_min.clone())
                    .style(LineStyle::dashed_loose())
                    .color(Color32::WHITE)
                );

            });

    }
}
