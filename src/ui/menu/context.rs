use crate::{app::app_state::Console, scene::Scene};

pub struct ContextMenu {
    shape_id: Option<u32>,
    position: egui::Pos2,
}

impl ContextMenu {
    pub fn new() -> Self {
        Self {
            shape_id: None,
            position: egui::Pos2::ZERO,
        }
    }

    pub fn open(&mut self, id: u32, position: egui::Pos2) {
        self.shape_id = Some(id);
        self.position = position;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, scene: &mut Scene, console: &mut Console) {
        let Some(id) = self.shape_id else {
            return;
        };

        let response = egui::Area::new(egui::Id::new("mesh_context_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(self.position)
            .show(ui, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(format!("Shape {id}"));

                    if ui.button("Delete").clicked() {
                        scene.remove_shape(id as u64);
                        console.append(&format!("Deleted shape with id {id}"));
                        self.shape_id = None;
                    }
                });
            });

        let clicked_outside = ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !response.response.rect.contains(pos))
        });

        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));

        if clicked_outside || escape {
            self.shape_id = None;
        }
    }
}
