use std::{collections::HashMap, hash::Hash};

use egui::{accesskit::Point, CollapsingHeader, Rect, RichText, Ui};
use egui_plot::{PlotPoint, PlotPoints, Points};

pub enum Par {
    VCD,
    Point
}

#[derive(Clone, Copy, PartialEq)]
pub enum Action {
    Keep,
    Delete,
}

#[derive(Clone, Debug)]
pub struct Tree {
    pub nodes: Vec<ParentNode>
}
impl Tree {

    pub fn ui(&mut self, ui: &mut Ui){
        self.ui_impl(ui)
    }

    fn ui_impl(&mut self, ui: &mut Ui){

        CollapsingHeader::new("Nodes")
            .show(ui, |ui| self.children_ui(ui));
    }
    
    fn children_ui(&mut self, ui: &mut Ui){
        for node in &mut self.nodes {
            CollapsingHeader::new(node.name.to_owned())
            .show(ui, |ui| {
                node.children_ui(ui);
            });
        }
    }

    pub fn plot_points(&mut self) -> Vec<Points> {
        let mut out = Vec::new();
        for node in &mut self.nodes {
            out.push(node.points());
        }

        out
    }
    pub fn index_of(&mut self, name: String) -> Option<usize> {
        self
            .nodes
            .iter()
            .position(|node| {
                node.name == name 
            })
    }

    pub fn remove(&mut self, name: String, x: f64, y: f64) {
        if let Some(pos) = self.index_of(name) {
                self.nodes[pos].remove(x, y);
        }
    }
    pub fn add(&mut self, name: String, x: f64, y: f64) {
        if let Some(pos) = self.index_of(name) {
                self.nodes[pos].add(x, y);
            }
    }
    pub fn get(&mut self, name: String) -> Option<Vec<[f64; 2]>> {
        if let Some(pos) = self.index_of(name.clone()) {
            return self.nodes[pos].get_vec();
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct ParentNode {
    pub name: String,
    pub children: Vec<Node>
}
impl ParentNode {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            children: vec![],
        }
    }

    fn children_ui(&mut self, ui: &mut Ui) {
        
        // add childeren
        self.children.retain_mut(|node| {
            if node.ui(ui) == Action::Delete {
                false
            } else {
                true
            }
        });
        ui.separator();
        if ui.button(RichText::new("+")).clicked() {
            self.children.push(Node::default());
        }
    }
    fn points(&self) -> Points {
        let series: Vec<[f64;2]> = self.children
        .iter()
        .map(|node| {
            [node.x, node.y]
        }).collect();
    
        Points::new(series).name(self.name.clone()).id(egui::Id::new(self.name.clone()))
    }
    pub fn add(&mut self, x: f64, y: f64) {
        self.children.push(Node::new(x, y));
    }
    pub fn remove(&mut self, x: f64, y: f64) {
        
        if let Some(pos) = self
        .children
        .iter()
        .position(|node| {
            let x_diff = node.x - x;
            let y_diff = node.y - y;

            let alpha = x_diff.powf(2.) + y_diff.powf(2.);

            alpha < 0.8
        }) {
            self.children.remove(pos);
        }
    }
    fn get_vec(&mut self) -> Option<Vec<[f64; 2]>> {
        if self.children.len() > 0 {
            let mut vec = self.children.clone();
            let out: Vec<[f64;2]> = vec.iter_mut().map(|i| i.point()).collect();
            return Some(out)
        }

        None
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Node {
    pub x: f64,
    pub y: f64,
}

impl Node {
    pub fn default() -> Self {
        Self {
            x: 0.,
            y: 0.
        }
    }

    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y
        }
    }

    fn ui(&mut self, ui: &mut Ui)  -> Action{
        ui.separator();
        ui.label("point");
        ui.label("X:");
        ui.add(egui::DragValue::new(&mut self.x));
        ui.label("Y:");
        ui.add(egui::DragValue::new(&mut self.y));
        if ui.button(RichText::new("Delete").color(ui.visuals().warn_fg_color)).clicked() {
            return Action::Delete
        };
        
        Action::Keep
    }
    fn point(&mut self) -> [f64; 2] {
        [self.x, self.y]
    }
}
