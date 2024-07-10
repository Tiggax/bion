use argmin::{core::{CostFunction, Error}, solver};
use crate::model::{Bioreactor, VOLUME, State};
use crate::base::{Initial, Graphs};
use crate::ui::{tree::{Tree, ParentNode}, regression::Group};

pub struct Regressor {
    pub initial: Initial,
    pub nodes: Tree,
}

impl Regressor {
    fn default() -> Self {
        Self {
            initial: Initial::default(),
            nodes: Tree {
                nodes: vec![
                    ParentNode::new(Group::VCD.to_string()),
                    ParentNode::new(Group::Glucose.to_string()),
                    ParentNode::new(Group::Glutamin.to_string()),
                ],
            },
        }
    }
}

impl CostFunction for Regressor {
    type Param = Vec<f64>;
    type Output = f64;
    
    fn cost(&self, p: &Self::Param) -> Result<Self::Output, Error> {
        for v in p {
            if *v < 0. {
                // return Err(Error::msg("value is negative"))
                return Ok(100000.)
            }
        }

        println!("cost ran");
        let mut p = p.clone();
        let k_glut = p.pop();
        let ks_glut = p.pop();
        let k_gluc = p.pop();
        let ks_gluc = p.pop();
        let mu = p.pop();

        if let [None, None, None, None, None, ] = [mu, ks_gluc, k_gluc, ks_glut, k_glut] {
            return Err(Error::msg("no point data"));
        }


        println!("values:\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}", mu, ks_gluc, k_gluc, ks_glut, k_glut);

        let k_glut = k_glut.expect("no k_glut");
        let ks_glut = ks_glut.expect("no ks_glut");
        let k_gluc = k_gluc.expect("no k_gluc");
        let ks_gluc = ks_gluc.expect("no ks_gluc");
        let mu = mu.expect("no mu");

        let Initial {
            vcd,
            gluc,
            glut,
        } = self.initial;
        
        const MINUTES: f64 = 14. * 24. * 60.;

        let init_cond = State::from([VOLUME, vcd, gluc, glut, 80., 0., 0. ]);
        let system = Bioreactor::fit(mu, ks_gluc, k_gluc, ks_glut, k_glut);
        println!("{:?}", &system);
        
        let mut stepper = ode_solvers::Rk4::new(system, 0., init_cond, MINUTES, 1.);
        let res = stepper.mut_integrate();
        if let Ok(val) = res {
            let mut g_vcd = Vec::new();
            let mut g_gluc = Vec::new();
            let mut g_glut = Vec::new();

            let mut dist = 0.;

            for (t,y) in stepper.x_out().iter().zip(stepper.y_out()) {
                g_vcd.push([*t, y[1] ]);
                g_gluc.push([*t, y[2] ]);
                g_glut.push([*t, y[3] ]);
            }
            println!("made vector points");
            for (g_vec, name) in [(g_vcd, "VCD".to_string()), (g_gluc, "Glucose".to_string()), (g_glut, "Glutamin".to_string())] {
                if let Some(vec) = self.nodes.clone().get(name.clone()) {
                    println!("some: {}: {:?}\n....\n{:?}", name, vec, g_vec);
                    for [nx, ny] in vec {
                        println!("point: ({}, {})", nx, ny);
                        if let Some(ps) = g_vec.iter().position(|[x,y]| { (*x - nx).powf(2.) < 1.}) {
                            let point = g_vec[ps];
                            println!("point: {:?}", point);
                            dist += (nx - point[0]).powf(2.);
                            dist += (ny - point[1]).powf(2.);
                            println!("iter dist:{dist}");
                        }
                    }
                }
            }
            println!("result: {dist}");
            if dist.is_nan() {
                dist  = 10000.;
            }
            return Ok(dist)
        }
        
        
        Err(Error::msg("no point"))
    }
}