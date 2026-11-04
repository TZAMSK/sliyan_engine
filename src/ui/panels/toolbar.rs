use egui::{Panel, Popup, PopupCloseBehavior, RectAlign};

use crate::{
    app::app_state::AppState,
    scene::camera::{
        view_mode::{
            CameraViewSelector, FrontView, IsometricView, OrthographicView, SideView, TopView,
        },
        Camera,
    },
    ui::dimensions,
};

pub struct ToolbarPanel;

impl ToolbarPanel {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut AppState, camera: &mut Camera) {
        Panel::top("toolbar")
            .exact_size(dimensions::TOOLBAR_HEIGHT)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    let response = ui.button("Add Shape");

                    self.apply_options(Popup::menu(&response))
                        .show(|ui| self.menu(ui, state));

                    if ui.button("Iso").clicked() {
                        CameraViewSelector::select::<IsometricView>(camera);
                    }
                    if ui.button("Top XY").clicked() {
                        CameraViewSelector::select::<TopView>(camera);
                    }
                    if ui.button("Front XZ").clicked() {
                        CameraViewSelector::select::<FrontView>(camera);
                    }
                    if ui.button("Side YZ").clicked() {
                        CameraViewSelector::select::<SideView>(camera);
                    }
                    if ui.button("Orthographic").clicked() {
                        CameraViewSelector::select::<OrthographicView>(camera);
                    }
                });
            });
    }

    fn apply_options<'a>(&self, popup: Popup<'a>) -> Popup<'a> {
        popup
            .align(RectAlign::default())
            .gap(4.0)
            .close_behavior(PopupCloseBehavior::CloseOnClick)
    }

    fn menu(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.menu_button("2D", |ui| {
            self.nested_2d_menu(ui, state);
        });

        ui.menu_button("3D", |ui| {
            self.nested_3d_menu(ui, state);
        });
    }

    fn nested_2d_menu(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        if let Some(selection) = state.selection.as_mut() {
            if ui.button("Triangle").clicked() {
                selection.arm_triangle_placement();
                state
                    .console
                    .append("Triangle placement armed. Clicke inside the viewport");
            };

            if ui.button("Rectangle").clicked() {
                selection.arm_rectangle_placement();
                state
                    .console
                    .append("Rectangle placement armed. Clicke inside the viewport");
            }

            if ui.button("Circle").clicked() {
                selection.arm_circle_placement();
                state
                    .console
                    .append("Circle placement armed. Clicke inside the viewport");
            }
        }
    }

    fn nested_3d_menu(&mut self, ui: &mut egui::Ui, state: &mut AppState) {
        if let Some(selection) = state.selection.as_mut() {
            if ui.button("Cube").clicked() {
                selection.arm_cube_placement();
                state
                    .console
                    .append("Cube placement armed. Clicke inside the viewport");
            }

            if ui.button("Sphere").clicked() {
                selection.arm_sphere_placement();
                state
                    .console
                    .append("Sphere placement armed. Clicke inside the viewport");
            }
        }
    }
}
