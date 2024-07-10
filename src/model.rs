
pub const FEED_RATE: f64 = 0.03;
pub const VOLUME: f64 = 45.; // L
pub type State = ode_solvers::SVector<f64,7>;
pub type Time = f64;

#[derive(Debug)]
pub struct Bioreactor {
    k_product: f64,
    mu: f64,
    n_vcd: f64,
    ks_gluc: f64,
    k_gluc: f64,
    gluc_feed: f64,
    ks_glut: f64,
    k_glut: f64, 
    glut_feed: f64,
    feed_min: f64,
    fi_v: f64,
    k_c_O2: f64,
    pzv: f64,
    air_flow: f64,
    O2_min: f64,
    k_O2: f64,
    henry: f64,
    c_O2_SAT: f64,
    fi_O2_max: f64,
    O2_flow_max: f64,
    O2_feed_rate: f64,
}
impl Bioreactor {
    pub fn new(k_product: f64, mu: f64, n_vcd: f64, ks_gluc: f64, k_gluc: f64, gluc_feed: f64,k_do: f64, ks_glut: f64, k_glut: f64, glut_feed: f64, feed_min: f64, fi_v: f64, pzv: f64, air_flow: f64, O2_min: f64, k_O2:f64, henry: f64, fi_O2_max: f64, O2_flow_max: f64, O2_feed_rate: f64) -> Self {
        Bioreactor {
            k_product, mu, n_vcd, ks_gluc, k_gluc, gluc_feed, ks_glut, k_glut, glut_feed, feed_min, fi_v, k_c_O2: k_do, pzv, air_flow, O2_min, k_O2, henry, c_O2_SAT: henry * 0.21, fi_O2_max, O2_flow_max, O2_feed_rate: O2_feed_rate * 1e-10,
        }
    }
    pub fn default() -> Self {
        let henry = 1.6066e-3;

        Self {
            k_product: 0.001,
            mu: 0.001,
            n_vcd: 0.7,
            ks_gluc: 0.05,
            k_gluc: 0.0001,
            gluc_feed: 12.,
            ks_glut: 0.05,
            k_glut: 0.0001,
            glut_feed: 7.,
            feed_min: 2.,
            fi_v: 45. * 0.03 / (24.* 60.), // Volume * feed rate in min
            k_c_O2: 0.001,
            pzv: 13.,
            air_flow: 0.5, // VVh
            O2_min: 25.,
            k_O2: 1.,
            henry, // mol / (L atm)
            c_O2_SAT: henry * 0.21,
            fi_O2_max: 15.,
            O2_flow_max: 15.,
            O2_feed_rate: 1.9444,
        }
    }
    pub fn fit(mu: f64, ks_gluc: f64, k_gluc: f64, ks_glut: f64, k_glut: f64) -> Self {

        let mut def = Self::default();
        def.mu = mu;
        def.ks_gluc = ks_gluc;
        def.k_gluc = k_gluc;
        def.ks_glut = ks_glut;
        def.k_glut = k_glut;

        def
    }
}
impl ode_solvers::System<Time, State> for Bioreactor {
    fn system(&self, x: Time, y: &State, dy: &mut State) {
        
    }

    fn mut_system(&self, x: Time, y: &mut State, dy: &mut State) {
        let (v, vcd, gluc, glut, c_O2, mut O2_flow, product) = (y[0], y[1], y[2], y[3], y[4], y[5], y[6]);

        let k_product = self.k_product;
        let mu = self.mu;
        let n_vcd = self.n_vcd;
        let feed_min = self.feed_min * 24. * 60.;
        let fi_v = self.fi_v;
        
        let k_c_O2 = self.k_c_O2;
        let pzv = self.pzv;
        let air_flow = self.air_flow;
        let DO_setpoint = self.O2_min;
        let Kc = self.k_O2;
        
        let ks_gluc = self.ks_gluc;
        let k_gluc = self.k_gluc;
        let gluc_feed = self.gluc_feed;
        
        let ks_glut = self.ks_glut;
        let k_glut = self.k_glut;
        let glut_feed = self.glut_feed;
        let henry = self.henry;
        let c_O2_SAT = self.c_O2_SAT;
        let fi_O2_max = self.fi_O2_max;
        let O2_flow_max = self.O2_flow_max;
        let O2_feed_rate = self.O2_feed_rate;
        

        // Volume
        //dy[0] = 

        // VCD
        let c_mu = mu * ( gluc / (ks_gluc + gluc)) * ( glut / ( ks_glut + glut ) ) * (c_O2 / (k_c_O2 + c_O2));
        dy[1] = c_mu * vcd.powf(n_vcd);
        // Gluc
        dy[2] = - k_gluc * vcd * ( gluc / ( ks_gluc + gluc) );
        // Glut
        dy[3] = - k_glut * vcd * ( glut / ( ks_glut + glut) );

        // PRODUCT

        dy[6] = k_product * vcd;


        let DO = (c_O2 / c_O2_SAT) * 100.; // za PiD parametre
        let DO_error = DO_setpoint - DO;
        let mv = Kc * DO_error;
        if DO_error > 0. {
            O2_flow= (mv * fi_O2_max) * 10.;
            if O2_flow > O2_flow_max {
                O2_flow = O2_flow_max;
            }
        } else {
            O2_flow = 0.;
        }
        
        y[5] = O2_flow;

        let flow_total = air_flow + O2_flow; // 
        let kLa = 1.3e-3 * pzv.powf(1.1) * flow_total.powf(0.9) / 60.; // 1/s

        let fiv_O2_c = air_flow * 0.21 + O2_flow; // flow je stalen , kisik je odvisen
        
        let x_O2 = fiv_O2_c / (flow_total);
        let p_O2 = x_O2; // * 1 bar


        let c_O2_s = henry * p_O2;

        let out = vcd * O2_feed_rate * 60.; // mol / cel * sec
        let otr = kLa * (c_O2_s - c_O2);

        // c_O2
        if c_O2 > 0. {
            dy[4] = -out + otr;
        } else {
            dy[4] = otr;
        }






        if x < feed_min {
            dy[0] = 0.;
        } else {
            dy[0] = fi_v;
            dy[1] -= vcd * ( fi_v / v );
            dy[2] += ( gluc_feed - gluc ) * ( fi_v / v );
            dy[3] += ( glut_feed - glut ) * ( fi_v / v );
        }
    }
}