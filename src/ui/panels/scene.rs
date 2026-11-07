use std::thread::current;

use egui::Panel;
use egui_winit::egui::Color32;

use crate::{
    app::app_state::AppState,
    scene::{shape::Shape, Scene, ShapeRef},
    ui::{dimensions, panels::title_panel},
};

pub struct ScenePanel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Location {
    row: usize,
}

impl ScenePanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &AppState, scene: &mut Scene) {
        Panel::left("scene_panel")
            .resizable(true)
            .default_size(dimensions::LEFT_PANEL_WIDTH)
            .frame(egui::Frame::new().fill(Color32::from_rgb(15, 15, 15)))
            .show(ui, |ui| {
                title_panel(ui, "Scene");
                self.scene_properties(ui, state, scene);
            });
    }

    fn scene_properties(&mut self, ui: &mut egui::Ui, _state: &AppState, scene: &mut Scene) {
        let mut dropped = None;

        egui::Frame::new()
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.label(format!("Object ({})", scene.shapes.len()));
                ui.separator();

                for shape in &scene.shapes {
                    if shape.borrow().parent.is_none() {
                        draw_shape_tree(ui, shape, &mut dropped);
                    }
                }
            });

        if let Some((child_id, parent_id)) = dropped {
            reparent_shape(scene, child_id, parent_id);
        }
    }
}

fn draw_shape_tree(ui: &mut egui::Ui, shape_ref: &ShapeRef, dropped: &mut Option<(u64, u64)>) {
    let shape = shape_ref.borrow();

    let id = shape.id;

    let response = ui
        .dnd_drag_source(egui::Id::new(("shape", id)), id, |ui| {
            ui.label(format!("{} ({})", shape.name, shape.id));
        })
        .response;

    if let Some(payload) = response.dnd_hover_payload::<u64>() {
        if *payload != id {
            ui.painter().rect_stroke(
                response.rect,
                2.0,
                egui::Stroke::new(1.0, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
        }
    }

    if let Some(child_id) = response.dnd_release_payload::<u64>() {
        if child_id != id.into() {
            *dropped = Some((*child_id, id));
        }
    }

    if !shape.children.is_empty() {
        ui.indent(id, |ui| {
            for child in &shape.children {
                draw_shape_tree(ui, child, dropped);
            }
        });
    }
}

fn reparent_shape(scene: &mut Scene, child_id: u64, parent_id: u64) {
    if child_id == parent_id {
        return;
    }

    let Some(parent) = scene.find_shape_by_id(parent_id) else {
        return;
    };

    let Some(child) = scene.find_shape_by_id(child_id) else {
        return;
    };

    let old_parent_id = child.borrow().parent;
    if old_parent_id == Some(parent_id) {
        return;
    }

    let mut current = Some(parent_id);
    let mut visited = std::collections::HashSet::new();

    while let Some(id) = current {
        if id == child_id || !visited.insert(id) {
            return;
        }

        current = scene
            .find_shape_by_id(id)
            .and_then(|shape| shape.borrow().parent);
    }

    if let Some(old_parent_id) = old_parent_id {
        if let Some(old_parent) = scene.find_shape_by_id(old_parent_id) {
            old_parent.borrow_mut().remove_child(child_id);
        }
    }

    parent.borrow_mut().add_child(child);
}
