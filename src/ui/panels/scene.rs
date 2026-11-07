use egui::Panel;
use egui_winit::egui::Color32;

use crate::{
    app::app_state::AppState,
    scene::{shape::Shape, Scene},
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

                draw_shape_tree(ui, &scene.shapes, &mut dropped);
            });

        if let Some((child_id, parent_id)) = dropped {
            reparent_shape(&mut scene.shapes, child_id, parent_id);
        }
    }
}

fn draw_shape_tree(ui: &mut egui::Ui, shapes: &[Shape], dropped: &mut Option<(u64, u64)>) {
    for shape in shapes {
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
                draw_shape_tree(ui, &shape.children, dropped);
            });
        }
    }
}

fn reparent_shape(shapes: &mut Vec<Shape>, child_id: u64, parent_id: u64) {
    if child_id == parent_id {
        return;
    }

    fn contains_id(shape: &Shape, id: u64) -> bool {
        shape.id == id || shape.children.iter().any(|child| contains_id(child, id))
    }

    fn find(shapes: &[Shape], id: u64) -> Option<&Shape> {
        for shape in shapes {
            if shape.id == id {
                return Some(shape);
            }

            if let Some(found) = find(&shape.children, id) {
                return Some(found);
            }
        }

        None
    }

    if let Some(child) = find(shapes, child_id) {
        if contains_id(child, parent_id) {
            return;
        }
    } else {
        return;
    }

    if find(shapes, parent_id).is_none() {
        return;
    }

    fn take(shapes: &mut Vec<Shape>, id: u64) -> Option<Shape> {
        if let Some(index) = shapes.iter().position(|shape| shape.id == id) {
            return Some(shapes.remove(index));
        }

        for shape in shapes {
            if let Some(found) = take(&mut shape.children, id) {
                return Some(found);
            }
        }

        None
    }

    fn find_mut(shapes: &mut [Shape], id: u64) -> Option<&mut Shape> {
        for shape in shapes {
            if shape.id == id {
                return Some(shape);
            }

            if let Some(found) = find_mut(&mut shape.children, id) {
                return Some(found);
            }
        }

        None
    }

    if let Some(mut child) = take(shapes, child_id) {
        if let Some(parent) = find_mut(shapes, parent_id) {
            child.parent = Some(parent_id);
            parent.children.push(child);
        }
    }
}
