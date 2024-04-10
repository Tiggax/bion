
pub type State = ode_solvers::Vector5<f64>;
pub type Time = f64;


pub struct Bioreactor {
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
    k_do: f64,
}
impl Bioreactor {
    pub fn new(mu: f64, n_vcd: f64, ks_gluc: f64, k_gluc: f64, gluc_feed: f64,k_do: f64, ks_glut: f64, k_glut: f64, glut_feed: f64, feed_min: f64, fi_v: f64) -> Self {
        Bioreactor {
            mu, n_vcd, ks_gluc, k_gluc, gluc_feed, ks_glut, k_glut, glut_feed, feed_min, fi_v, k_do
        }
    }
}
impl ode_solvers::System<Time, State> for Bioreactor {
    fn system(&self, x: Time, y: &State, dy: &mut State) {
        let (v, vcd, gluc, glut, DO) = (y[0], y[1], y[2], y[3], y[4]);
        let mu = self.mu;
        let n_vcd = self.n_vcd;
        let feed_min = self.feed_min * 24. * 60.;
        let fi_v = self.fi_v;
        
        let k_do = self.k_do;

        let ks_gluc = self.ks_gluc;
        let k_gluc = self.k_gluc;
        let gluc_feed = self.gluc_feed;

        let ks_glut = self.ks_glut;
        let k_glut = self.k_glut;
        let glut_feed = self.glut_feed;



        // Volume
        //dy[0] = 

        // VCD
        let c_mu = mu * ( gluc / (ks_gluc + gluc)) * ( glut / ( ks_glut + glut ) ) * ( DO / ( k_do + DO ) );
        dy[1] = c_mu * vcd.powf(n_vcd);
        // Gluc
        dy[2] = - k_gluc * vcd * ( gluc / ( ks_gluc + gluc) );
        // Glut
        dy[3] = - k_glut * vcd * ( glut / ( ks_glut + glut) );
        // DO
        dy[4] = - vcd * k_do * 1e-3 * (DO / (DO + k_do )) + k_do * 1e-3 *((9. * DO) / (10. * DO));

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