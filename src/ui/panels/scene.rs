use egui::Panel;
use egui_winit::egui::Color32;

use crate::{
    app::app_state::AppState,
    scene::{
        shape::{ShapeId, Shapes},
        Scene,
    },
    ui::{dimensions, panels::title_panel},
};

pub struct ScenePanel;

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

                let mut roots: Vec<ShapeId> = scene
                    .shapes
                    .values()
                    .filter(|s| s.parent.is_none())
                    .map(|s| s.id)
                    .collect();
                roots.sort_unstable();

                for id in roots {
                    draw_shape_tree(ui, &scene.shapes, id, &mut dropped);
                }
            });

        if let Some((child_id, parent_id)) = dropped {
            reparent_shape(scene, child_id, parent_id);
        }
    }
}

fn draw_shape_tree(
    ui: &mut egui::Ui,
    shapes: &Shapes,
    id: ShapeId,
    dropped: &mut Option<(u64, u64)>,
) {
    let Some(shape) = shapes.get(&id) else { return };

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
        if *child_id != id.into() {
            *dropped = Some((*child_id, id));
        }
    }

    if !shape.children.is_empty() {
        ui.indent(id, |ui| {
            for &child_id in &shape.children {
                draw_shape_tree(ui, shapes, child_id, dropped);
            }
        });
    }
}

fn reparent_shape(scene: &mut Scene, child_id: u64, parent_id: u64) -> bool {
    scene.add_child(parent_id, child_id)
}
