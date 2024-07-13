use egui::Ui;


pub mod tree;
pub mod app;

pub trait Front {
    fn left_panel(&mut self, ui: &mut Ui, ctx: &egui::Context);
    fn center_panel(&mut self, ui: &mut Ui, ctx: &egui::Context);

    fn show_inside(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        egui::SidePanel::left("options").show_inside(ui,|ui| { self.left_panel(ui, ctx)});     
        egui::CentralPanel::default().show_inside( ui, |ui|{ self.center_panel(ui, ctx) });
    }
}