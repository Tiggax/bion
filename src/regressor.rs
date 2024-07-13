use std::fmt::{self, Display, Formatter};

use argmin::core::{CostFunction, Error};
use crate::{model::{Bioreactor, State}, ui::tree::{self}};
use crate::ui::tree::{Tree, ParentNode};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Group {
    VCD,
    Glucose,
    Glutamin,
}

impl Display for Group {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Param {
    pub target: Target,
    pub mode: Mode
}
impl Param {
    pub fn default() -> Self {
        Self {
            target: Target::MuMax,
            mode: Mode::Mixed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Single(Group),
    Mixed
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    MuMax,
    NVcd,
    FeedRate,
    Glucose,
    Glutamin
}
#[derive(Clone, Debug)]
pub struct RegressorNode {
    pub group: Group,
    pub x: f64,
    pub y: f64
}

impl RegressorNode {
    fn new(group: Group, x: f64, y:f64) -> Self  {
        Self { group, x, y }
    }
    pub fn translate(tree: Tree) -> Vec<RegressorNode> {
        let mut out = Vec::new();
        for ParentNode { name, children } in tree.nodes {

            match name.as_ref() {
                "VCD" => {
                    for tree::Node { x, y } in children {
                        out.push(RegressorNode::new(Group::VCD, x, y));
                    }
                },
                "Glucose" => {
                    for tree::Node { x, y } in children {
                        out.push(RegressorNode::new(Group::Glucose, x, y));
                    }
                },
                "Glutamin" => {
                    for tree::Node { x, y } in children {
                        out.push(RegressorNode::new(Group::Glutamin, x, y));
                    }
                },
                _ => {}
            }

        }

        out.sort_by(|a,b| {
            a.x.partial_cmp(&b.x).unwrap()
        });
        let out = out.into_iter().rev().collect();
        out
    }
}

pub struct Regressor {
    pub nodes: Vec<RegressorNode>,
    pub simulation: Bioreactor,
    pub param: Param,
    pub epsilon: f64,

}

impl Regressor {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            simulation: Bioreactor::default(),
            epsilon: 1e-3,
            param: Param::default(),
        }
    }
}

impl CostFunction for Regressor {
    type Param = f64;
    type Output = f64;

    fn cost(&self, val: &Self::Param) -> Result<Self::Output, Error> {

        if *val < 0. {
            return Ok(100_000.)
        }

        const MINUTES: f64 = 14. * 24. * 60.;

        const STEP: f64 = 2.; // step increment lower is more precise but more computationaly intense
        
        let initial_state = State::from([
            self.simulation.initial.volume, 
            self.simulation.initial.vcd, 
            self.simulation.initial.glucose, 
            self.simulation.initial.glutamin, 
            (self.simulation.initial.oxigen_part * self.simulation.oxigen_saturation()) / 100., 
            0., 
            0. 
        ]);

        let mut simulation = self.simulation.clone();
        simulation.update(&self.param, *val);

        let mut stepper = ode_solvers::Rk4::new(simulation, 0., initial_state, MINUTES, STEP);
        let res = stepper.mut_integrate();

        if let Ok(_val) = res {

            let nodes = match &self.param.mode {
                Mode::Single(val) => {
                    self.nodes.clone().into_iter()
                    .filter(|node| {
                        node.group == *val
                    }).collect::<Vec<RegressorNode>>()
                }
                Mode::Mixed => self.nodes.clone(),
            };

            //let mut current_node = nodes.pop();
            let mut result = 0.;
            // for (t, y) in stepper.x_out().iter().zip(stepper.y_out()) {

            //     match &current_node {
            //         None => {println!("no more nodes"); break;},
            //         Some(node) => {
            //             println!("in some");
            //             if (node.x - t).abs() > self.epsilon {
            //                 continue;
            //             }
            //             let y = match node.group {
            //                 Group::VCD => y[1],
            //                 Group::Glucose => y[2],
            //                 Group::Glutamin => y[3],
            //             };

            //             result += (y.powf(2.) - node.y.powf(2.)).abs();
            //             current_node = nodes.pop();
            //         }
            //     }
            // }

            for node in nodes {
                for (t, y) in stepper.x_out().iter().zip(stepper.y_out()) {
                    if (node.x - t).abs() > self.epsilon {
                        continue;
                    }
                    let y = match node.group {
                        Group::VCD => y[1],
                        Group::Glucose => y[2],
                        Group::Glutamin => y[3],
                    };
                    result += (y.powf(2.) - node.y.powf(2.)).abs();
                }
            }
            Ok(result)

        } else {
            return Err(Error::msg("no result"))
        }
    }
}