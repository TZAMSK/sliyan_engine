use crate::{
    app::app_state::Console,
    editor::gizmo::{GizmoApp, GizmoModePicker},
};

pub struct GizmoSettingsMenu;

impl GizmoSettingsMenu {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        viewport_position: egui::Pos2,
        gizmo: &mut GizmoApp,
        console: &mut Console,
    ) {
        egui::Area::new(egui::Id::new("gizmo_settings_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::new(
                viewport_position.x + 10.0,
                viewport_position.y + 10.0,
            ))
            .show(ui, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    if ui.button("Translation").clicked() {
                        gizmo.set_to(GizmoModePicker::Translation, console);
                    }

                    if ui.button("Rotation").clicked() {
                        gizmo.set_to(GizmoModePicker::Rotation, console);
                    }

                    if ui.button("Scale").clicked() {
                        gizmo.set_to(GizmoModePicker::Scale, console);
                    }
                });
            });
    }
}
