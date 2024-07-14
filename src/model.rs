use egui::Slider;
use serde::Serialize;

use crate::regressor::Param;


pub const FEED_RATE: f64 = 0.03;
pub const VOLUME: f64 = 45.; // L
pub type State = ode_solvers::SVector<f64,7>;
pub type Time = f64;


#[derive(Debug, Clone, Serialize)]
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
#[derive(Debug, Clone, Serialize)]
pub struct Constants {
    pub product: f64,
    pub glucose: f64,
    pub glutamin: f64,
    pub oxigen: f64, // PID
    pub air: f64, // DO

}
impl Constants {
    pub fn default() -> Self {
        Self {
            product:    1e-3,
            glucose:    1e-4,
            glutamin:   1e-4,
            oxigen:     1e-1,
            air:        1e-3,
        }
    }
}

#[derive(Debug,Clone, Serialize)]
pub struct Airation {
    pub cell_metabolism: f64,
    pub air_flow: f64, // [VVh]
    pub henry: f64,
    pub pid: Pid
}
impl Airation{
    pub fn default() -> Self {
        Self {
            cell_metabolism: 1.9444,
            air_flow: 0.5,//4.6e-2,
            henry: 1.6e-3,
            pid: Pid::default(),
        }
    }
}

#[derive(Debug,Clone, Serialize)]
pub struct Pid {
    pub minimum: f64,
    pub fi_oxigen_max: f64,
    pub max_flow: f64,
}
impl Pid {
    pub fn default() -> Self {
        Self {
            minimum: 25.,
            fi_oxigen_max: 15.,
            max_flow: 15.,
        }
    }
}

#[derive(Debug,Clone, Serialize)]
pub struct Initial {
    pub volume: f64,
    pub vcd: f64,
    pub glucose: f64,
    pub glutamin: f64,
    pub oxigen_part: f64,
}
impl Initial {
    pub fn default() -> Self {
        Self {
            volume: 45.,
            vcd: 0.5,
            glucose: 12.,
            glutamin: 7.,
            oxigen_part: 80.,
        }
    }
}

#[derive(Debug,Clone, Serialize)]
pub struct Feeding {
    pub start : f64, // Day
    pub rate: f64, // feed_rate
    pub glucose: f64,
    pub glutamin: f64,
}
impl Feeding {
    pub fn default() -> Self {
        Self {
            start: 2.,
            rate: 0.03,
            glucose: 12.,
            glutamin: 7.,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Bioreactor {
    pub mu_max: f64,
    pub mix_rate: f64, // pzv

    pub ks_glucose: f64,
    pub ks_glutamin: f64,

    pub temp_shift: TempShift,
    pub constants: Constants,
    pub airation: Airation,
    pub initial: Initial,
    pub feeding: Feeding,
}
impl Bioreactor {

    pub fn default() -> Self {
        Self {
            mu_max: 0.002,
            mix_rate: 13.,
            ks_glucose: 0.05,
            ks_glutamin: 0.05,

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
        def.constants.glucose = k_gluc;
        def.constants.glutamin = k_glut;
        def.airation.air_flow = air_flow;
        def.feeding.rate = feed_rate;


        def
    }

    pub fn view(&mut self, ui: &mut egui::Ui) -> bool{

        ui.add(Slider::new(&mut self.mu_max, 0.0..=10.).text("Mu max")).changed() ||
        ui.add(Slider::new(&mut self.mix_rate, 0.0..=100.).text("Mix rate [P/V]")).changed() ||
        ui.add(Slider::new(&mut self.ks_glucose, 0.0..=1.).text("ks_glucose")).changed() ||
        ui.add(Slider::new(&mut self.ks_glutamin, 0.0..=1.).text("ks glutamin")).changed() ||
        ui.collapsing("Initial", |ui|{
            ui.add(Slider::new(&mut self.initial.vcd, 0.0..=10.).text("VCD")).changed() ||
            ui.add(Slider::new(&mut self.initial.volume, 0.0..=100.).text("volume")).changed() ||
            ui.add(Slider::new(&mut self.initial.glucose, 0.0..=20.).text("glucose")).changed() ||
            ui.add(Slider::new(&mut self.initial.glutamin, 0.0..=20.).text("glutamin")).changed() ||
            ui.add(Slider::new(&mut self.initial.oxigen_part, 0.0..=100.).text("oxigen part [DO]")).changed() ||
            false
        }).body_returned.unwrap_or(false) ||
        ui.collapsing("Constants", |ui|{
            ui.add(Slider::new(&mut self.constants.product, 0.0..=1.).text("product")).changed() ||
            ui.add(Slider::new(&mut self.constants.glucose, 0.0..=1.).text("glucose")).changed() ||
            ui.add(Slider::new(&mut self.constants.glutamin, 0.0..=1.).text("glutamin")).changed() ||
            ui.add(Slider::new(&mut self.constants.air, 0.0..=1.).text("air [DO]")).changed() ||
            ui.add(Slider::new(&mut self.constants.oxigen, 0.0..=1.).text("oxigen [Pid]")).changed() ||
            ui.add(Slider::new(&mut self.airation.henry,0.0..=10.).text("Henry's constant")).changed() ||
            false
        }).body_returned.unwrap_or(false) ||
        ui.label("Time shift").changed() ||
        ui.collapsing("Time shift", |ui| {
            ui.add(Slider::new(&mut self.temp_shift.day, 0.0..=14.).text("shift day")).changed() ||
            ui.add(Slider::new(&mut self.temp_shift.n_vcd, 0.0..=2.).text("post shift growth")).changed() ||
            false

        }).body_returned.unwrap_or(false) ||
        ui.collapsing("Airation", |ui|{
            ui.add(Slider::new(&mut self.airation.cell_metabolism, 0.0..=100.).text("Cell metabolism")).changed() ||
            ui.add(Slider::new(&mut self.airation.air_flow, 0.0..=100.).text("Air flow ")).changed() ||
            ui.collapsing("PID", |ui|{
                ui.add(Slider::new(&mut self.airation.pid.minimum, 0.0..=100.).text("minimum [%]")).changed() ||
                ui.add(Slider::new(&mut self.airation.pid.max_flow, 0.0..=100.).text("max_flow")).changed() ||
                ui.add(Slider::new(&mut self.airation.pid.fi_oxigen_max, 0.0..=100.).text("fi_oxigen_max")).changed() ||
                false
            }).body_returned.unwrap_or(false) ||
            false
        }).body_returned.unwrap_or(false) ||
        ui.collapsing("Feeding", |ui|{
            ui.add(Slider::new(&mut self.feeding.start, 0.0..=14.).text("start")).changed() ||
            ui.add(Slider::new(&mut self.feeding.rate, 0.0..=1.).text("rate")).changed() ||
            ui.add(Slider::new(&mut self.feeding.glucose, 0.0..=100.).text("glucose")).changed() ||
            ui.add(Slider::new(&mut self.feeding.glutamin, 0.0..=100.).text("glutamin")).changed() ||
            false
        }).body_returned.unwrap_or(false) ||
        false
    }


    pub fn update(&mut self, param: &Param, val: f64) {

        match param.target {
            crate::regressor::Target::MuMax => self.mu_max = val,
            crate::regressor::Target::NVcd => self.temp_shift.n_vcd = val,
            crate::regressor::Target::FeedRate => self.feeding.rate = val,
            crate::regressor::Target::Glucose => self.constants.glucose = val,
            crate::regressor::Target::Glutamin => self.constants.glutamin = val,
        }
    }
}
impl ode_solvers::System<Time, State> for Bioreactor {

    fn mut_system(&self, x: Time, y: &mut State, dy: &mut State) {
        let (v, vcd, gluc, glut, c_o2, mut o2_flow, product) = (y[0], y[1], y[2], y[3], y[4], y[5], y[6]);

        let mix_rate = self.mix_rate * 1e-2;
        let air_flow = self.airation.air_flow * self.initial.volume;

        
        // Temp shift
        let n_vcd = if x < self.temp_shift.day * 24. * 60. {
            1.
        } else {
            self.temp_shift.n_vcd
        };
        // Volume
        //dy[0] = 

        // VCD
        let mut c_mu = self.mu_max * ( gluc / (self.ks_glucose + gluc)) * ( glut / ( self.ks_glutamin + glut ) ) * (c_o2 / ( self.constants.air + c_o2));
        c_mu = if gluc < 0. || glut < 0. || c_o2 < 0. {-1. * c_mu.abs()} else {c_mu}; // old -1. * c_mu.abs()
        
        dy[1] = c_mu * vcd * n_vcd;
        // Gluc
        dy[2] = - self.constants.glucose * vcd * ( gluc / ( self.ks_glucose + gluc) );
        // Glut
        dy[3] = - self.constants.glutamin * vcd * ( glut / ( self.ks_glutamin + glut) );

        // PRODUCT

        dy[6] = self.constants.product * vcd;

        #[allow(non_snake_case)]
        let DO = (c_o2 / self.oxigen_saturation()) * 100.; // za PiD parametre
        #[allow(non_snake_case)]
        let DO_error = self.airation.pid.minimum - DO;
        let mv = self.constants.oxigen * DO_error;
        if DO_error > 0. {
            o2_flow= (mv * self.airation.pid.fi_oxigen_max) * 10.;
            if o2_flow > self.airation.pid.max_flow {
                o2_flow = self.airation.pid.max_flow;
            }
        } else {
            o2_flow = 0.;
        }
        
        y[5] = o2_flow;

        let flow_total = air_flow + o2_flow; // 
        let k_la = 1.3e-3 * mix_rate.powf(1.1) * flow_total.powf(0.9) * 60.; // 1/s

        let fiv_o2_c = air_flow * 0.21 + o2_flow; // flow je stalen , kisik je odvisen
        
        let x_o2 = fiv_o2_c / (flow_total);
        let p_o2 = x_o2; // * 1 bar


        let c_o2_s = self.airation.henry * p_o2;

        let out =  vcd * self.airation.cell_metabolism * 1e-10 * 60.; // mol / cel * sec
        let otr = k_la * (c_o2_s - c_o2);




        // c_O2
        if c_o2 > 0. {
            dy[4] = -out + otr;
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
            dy[3] += ( self.feeding.glutamin - glut ) * ( fi_v / v );

            dy[6] += product * (fi_v / v);
        }
    }
    fn system(&self, x: Time, y: &State, dy: &mut State) {}
}
