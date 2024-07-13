use ode_solvers::Rk4;

use crate::model::Bioreactor;

#[derive(Debug)]
pub struct Graphs {
    pub volume: Vec<[f64;2]>,
    pub vcd: Vec<[f64;2]>,
    pub glucose: Vec<[f64;2]>,
    pub glutamin: Vec<[f64;2]>,
    pub c_O2: Vec<[f64;2]>,
    pub O2: Vec<[f64;2]>,
    pub product: Vec<[f64;2]>,
}

impl Graphs {
    pub fn default() -> Self {
        Self {
            volume: Vec::new(),
            vcd: Vec::new(),
            glucose: Vec::new(),
            glutamin: Vec::new(),
            c_O2: Vec::new(),
            O2: Vec::new(),
            product: Vec::new(),
        }
    }

}

#[derive(Clone)]
pub struct Initial {
    pub vcd: f64,
    pub gluc: f64,
    pub glut: f64,
}
impl Initial {
    pub fn default() -> Self {
        Self {
            vcd: 0.5,
            gluc: 7.,
            glut: 12.,
        }
    }
}