use egui::Ui;

pub mod simulations;
pub mod regression;
pub mod tree;

pub trait Front {
    fn left_panel(&mut self, ui: &mut Ui);
    fn center_panel(&mut self, ui: &mut Ui);

    fn show_inside(&mut self, ui: &mut Ui) {
        egui::SidePanel::left("options").show_inside(ui,|ui| { self.left_panel(ui)});     
        egui::CentralPanel::default().show_inside( ui, |ui|{ self.center_panel(ui) });
    }
}