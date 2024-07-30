use egui::Slider;
use serde::{Deserialize, Serialize};

use crate::regressor::Param;


pub const FEED_RATE: f64 = 0.03;
pub const VOLUME: f64 = 45.; // L
pub type State = ode_solvers::SVector<f64,7>;
pub type Time = f64;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempShift {
    pub n_vcd: f64, // temp shift post
    pub day: f64, // day of shift
}

impl TempShift {
    pub fn default() -> Self {
        Self {
            n_vcd: 0.7,
            day: 3.5,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constants {
    pub product: f64,   // [ml/(MVC min)]
    pub k_glucose: f64,   // [1/min]
    pub k_glutamine: f64, // [1/min]
    pub kP: f64,        // []
    pub kDO: f64,       // [%]

}
impl Constants {
    pub fn default() -> Self {
        Self {
            product:    1e-4,
            k_glucose:    1e-4,
            k_glutamine:  1e-4,
            kP:         1e-2,
            kDO:        1e-4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airation {
    pub cell_metabolism: f64,
    pub air_flow: f64, // [VVh]
    pub henry: f64,
    pub pid: Pid
}
impl Airation{
    pub fn default() -> Self {
        Self {
            cell_metabolism: 1.266,
            air_flow: 5e-3,
            henry: 1.6e-3,
            pid: Pid::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pid {
    pub minimum: f64,
    pub fi_oxygen_max: f64,
    pub max_flow: f64,
}
impl Pid {
    pub fn default() -> Self {
        Self {
            minimum: 25.,
            fi_oxygen_max: 15.,
            max_flow: 15.,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Initial {
    pub volume: f64,
    pub vcd: f64,
    pub glucose: f64,
    pub glutamine: f64,
    pub oxygen_part: f64,
}
impl Initial {
    pub fn default() -> Self {
        Self {
            volume: 45.,
            vcd: 0.5,
            glucose: 12.,
            glutamine: 7.,
            oxygen_part: 80.,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feeding {
    pub start : f64, // [day]
    pub rate: f64, // [(%"IWV")/"day]
    pub glucose: f64, // [g/L]
    pub glutamine: f64,
}
impl Feeding {
    pub fn default() -> Self {
        Self {
            start: 2.,
            rate: 0.03,
            glucose: 12.,
            glutamine: 7.,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bioreactor {
    pub mu_max: f64,
    pub power_input: f64, // pzv

    pub ks_glucose: f64,
    pub ks_glutamine: f64,

    pub temp_shift: TempShift,
    pub constants: Constants,
    pub airation: Airation,
    pub initial: Initial,
    pub feeding: Feeding,
}
impl Bioreactor {

    pub fn default() -> Self {
        Self {
            mu_max: 0.0005,
            power_input: 20.,
            ks_glucose: 0.05,
            ks_glutamine: 0.05,

            temp_shift: TempShift::default(),
            constants: Constants::default(),
            airation: Airation::default(),
            initial: Initial::default(),
            feeding: Feeding::default(),
        }
    }

    pub fn fi_v(&self) -> f64 {
        self.initial.volume * self.feeding.rate / (24. * 60.)
    }
    pub fn oxigen_saturation(&self) -> f64 {
        self.airation.henry * 0.21
    }

    pub fn fit(mu_max: f64, feed_rate: f64, air_flow: f64, k_gluc: f64,k_glut: f64 ) -> Self {

        let mut def = Self::default();
        def.mu_max = mu_max;
        def.constants.k_glucose = k_gluc;
        def.constants.k_glutamine = k_glut;
        def.airation.air_flow = air_flow;
        def.feeding.rate = feed_rate;


        def
    }

    pub fn view(&mut self, ui: &mut egui::Ui) -> bool{

        ui.add(Slider::new(&mut self.mu_max, 0.0..=0.01).text("Mu max [MVC/(ml min)]")).changed() ||
        ui.add(Slider::new(&mut self.power_input, 0.0..=100.).text("Power input [W/m3]")).changed() ||
        ui.add(Slider::new(&mut self.ks_glucose, 0.0..=0.2).text("ks_glucose [g/L]")).changed() ||
        ui.add(Slider::new(&mut self.ks_glutamine, 0.0..=0.2).text("ks glutamine [g/L]")).changed() ||
        ui.collapsing("Initial", |ui|{
            ui.add(Slider::new(&mut self.initial.vcd, 0.0..=10.).text("VCD [MVC/mL]")).changed() ||
            ui.add(Slider::new(&mut self.initial.volume, 0.0..=100.).text("volume [L]")).changed() ||
            ui.add(Slider::new(&mut self.initial.glucose, 0.0..=20.).text("glucose [g/L]")).changed() ||
            ui.add(Slider::new(&mut self.initial.glutamine, 0.0..=20.).text("glutamine [g/L]")).changed() ||
            ui.add(Slider::new(&mut self.initial.oxygen_part, 0.0..=100.).text("oxigen part [%]")).changed() ||
            false
        }).body_returned.unwrap_or(false) ||
        ui.collapsing("Constants", |ui|{
            ui.add(Slider::new(&mut self.constants.product, 0.0..=0.001).text("product [mg/MVC min]")).changed() ||
            ui.add(Slider::new(&mut self.constants.k_glucose, 0.0..=0.001).text("glucose [MVC/min]")).changed() ||
            ui.add(Slider::new(&mut self.constants.k_glutamine, 0.0..=0.001).text("glutamine [MVC/min]")).changed() ||
            ui.add(Slider::new(&mut self.constants.kDO, 0.0..=0.001).text("kDO [mol/L]")).changed() ||
            ui.add(Slider::new(&mut self.constants.kP, 0.0..=1.).text("kP [/]")).changed() ||
            ui.add(Slider::new(&mut self.airation.henry,0.0..=10.).text("Henry's constant [mol/(bar L)]")).changed() ||
            false
        }).body_returned.unwrap_or(false) ||
        ui.label("Time shift").changed() ||
        ui.collapsing("Time shift", |ui| {
            ui.add(Slider::new(&mut self.temp_shift.day, 0.0..=14.).text("shift day")).changed() ||
            ui.add(Slider::new(&mut self.temp_shift.n_vcd, 0.0..=2.).text("post shift growth")).changed() ||
            false

        }).body_returned.unwrap_or(false) ||
        ui.collapsing("Airation", |ui|{
            ui.add(Slider::new(&mut self.airation.cell_metabolism, 0.0..=100.).text("Cell metabolism [mol / (cell min)]")).changed() ||
            ui.add(Slider::new(&mut self.airation.air_flow, 0.0..=10.).text("Air flow [L / min]")).changed() ||
            ui.collapsing("PID", |ui|{
                ui.add(Slider::new(&mut self.airation.pid.minimum, 0.0..=100.).text("minimum [%]")).changed() ||
                ui.add(Slider::new(&mut self.airation.pid.max_flow, 0.0..=100.).text("max_flow [L/min]")).changed() ||
                ui.add(Slider::new(&mut self.airation.pid.fi_oxygen_max, 0.0..=100.).text("fi_oxigen_max [L / min]")).changed() ||
                false
            }).body_returned.unwrap_or(false) ||
            false
        }).body_returned.unwrap_or(false) ||
        ui.collapsing("Feeding", |ui|{
            ui.add(Slider::new(&mut self.feeding.start, 0.0..=14.).text("start")).changed() ||
            ui.add(Slider::new(&mut self.feeding.rate, 0.0..=1.).text("rate")).changed() ||
            ui.add(Slider::new(&mut self.feeding.glucose, 0.0..=100.).text("glucose")).changed() ||
            ui.add(Slider::new(&mut self.feeding.glutamine, 0.0..=100.).text("glutamine")).changed() ||
            false
        }).body_returned.unwrap_or(false) ||
        false
    }


    pub fn update(&mut self, param: &Param, val: f64) {

        match param.target {
            crate::regressor::Target::MuMax => self.mu_max = val,
            crate::regressor::Target::NVcd => self.temp_shift.n_vcd = val,
            crate::regressor::Target::FeedRate => self.feeding.rate = val,
            crate::regressor::Target::Glucose => self.constants.k_glucose = val,
            crate::regressor::Target::Glutamin => self.constants.k_glutamine = val,
            crate::regressor::Target::Product => self.constants.product = val,
            crate::regressor::Target::DO => self.constants.kDO = val,
        }
    }
}
impl ode_solvers::System<Time, State> for Bioreactor {

    fn mut_system(&self, x: Time, y: &mut State, dy: &mut State) {
        let (v, vcd, gluc, glut, c_o2, mut o2_flow, product) = (y[0], y[1], y[2], y[3], y[4], y[5], y[6]);

        let power_input = self.power_input; // [W/m^3]
        
        let air_flow = self.airation.air_flow * self.initial.volume; // [L/min]

        
        // Temp shift
        let n_vcd = if x < self.temp_shift.day * 24. * 60. {
            1.
        } else {
            self.temp_shift.n_vcd 
        };
        // Volume
        //dy[0] = 

        // VCD
        let mut c_mu = self.mu_max * ( gluc / (self.ks_glucose + gluc)) * ( glut / ( self.ks_glutamine + glut ) ) * (c_o2 / ( self.constants.kDO + c_o2));
        c_mu = if gluc < 0. || glut < 0. || c_o2 < 0. {-1. * c_mu.abs()} else {c_mu}; // old -1. * c_mu.abs()
        
        dy[1] = c_mu * vcd * n_vcd;
        // Gluc
        dy[2] = - self.constants.k_glucose * vcd * ( gluc / ( self.ks_glucose + gluc) );
        // Glut
        dy[3] = - self.constants.k_glutamine * vcd * ( glut / ( self.ks_glutamine + glut) );

        // PRODUCT

        dy[6] = self.constants.product * vcd;

        #[allow(non_snake_case)]
        let DO = (c_o2 / self.oxigen_saturation()) * 100.; // za PiD parametre
        #[allow(non_snake_case)]
        let DO_error = self.airation.pid.minimum - DO;
        let mv = self.constants.kP * DO_error;
        if DO_error > 0. {
            o2_flow = (mv * self.airation.pid.fi_oxygen_max) * 1000.;
        if o2_flow > self.airation.pid.max_flow {
            o2_flow = self.airation.pid.max_flow;
        }
        } else {
            o2_flow = 0.;
        }
        
        y[5] = o2_flow;


        let flow_total = air_flow + o2_flow; // 
        let mut k_la = 2.17e-5 * power_input.powf(1.1) * flow_total.powf(0.9); // 1/min
        k_la *= (self.initial.volume/v); // dilution volume correction 

        let fiv_o2_c = air_flow * 0.21 + o2_flow; // flow je stalen , kisik je odvisen
        
        let x_o2 = fiv_o2_c / (flow_total);
        let p_o2 = x_o2; // * 1 bar


        let c_o2_s = self.airation.henry * p_o2;

        let q_o2 = self.airation.cell_metabolism * 1e-8; // mol / cel * min
        let our =  vcd * q_o2; //  mol/(L min)
        let otr = k_la * (c_o2_s - c_o2); //  mol/(L min)




        // c_O2
        if c_o2 > 0. {
            dy[4] = -our + otr;
        } else {
            dy[4] = otr;
        }

        let fi_v = self.fi_v();

        if x < self.feeding.start * 24. *60. {
            dy[0] = 0.;
        } else {
            dy[0] = fi_v;
            dy[1] -= vcd * ( fi_v / v );
            dy[2] += ( self.feeding.glucose - gluc ) * ( fi_v / v );
            dy[3] += ( self.feeding.glutamine - glut ) * ( fi_v / v );

            dy[6] -= product * (fi_v / v);
        }
    }
    fn system(&self, x: Time, y: &State, dy: &mut State) {}
}
