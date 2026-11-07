use egui::Panel;
use egui_winit::egui::Color32;

use crate::{
    app::app_state::AppState,
    scene::{
        shape::{Properties, Shape},
        Scene,
    },
    ui::{dimensions, panels::title_panel},
};

pub struct InspectorPanel {
    active_tab: u32,
}

impl InspectorPanel {
    pub fn new() -> Self {
        Self { active_tab: 0 }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &AppState, scene: &mut Scene) {
        Panel::right("inspector_panel")
            .resizable(true)
            .default_size(dimensions::RIGHT_PANEL_WIDTH)
            .frame(egui::Frame::new().fill(Color32::from_rgb(15, 15, 15)))
            .show(ui, |ui| {
                title_panel(ui, "Inspector");
                self.scene_properties(ui, state, scene);
            });
    }

    fn scene_properties(&mut self, ui: &mut egui::Ui, state: &AppState, scene: &mut Scene) {
        egui::Frame::new()
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.label("Render");
                ui.separator();

                ui.label("Camera");
                ui.separator();
                ui.label(format!("Position: {}", format_vec3(state.camera.position)));
                ui.label(format!("Target: {}", format_vec3(state.camera.target)));
                ui.label("View mode:");
                ui.label("Placement mode:");

                let Some(selection) = &state.selection else {
                    return;
                };

                let id = selection.picker.selected_shape_id;

                if id == 0 {
                    ui.label("No shape selected");
                    return;
                }

                let Some(shape) = scene.find_shape_by_id_mut(id as u64) else {
                    ui.label("Shape not found");
                    return;
                };

                ui.label(format!("Selected: {} ({:2})", shape.name, shape.id));
                ui.separator();

                let available_height = ui.available_height();
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;

                        self.selectable_builder(ui, "C", 0);
                        self.selectable_builder(ui, "T", 1);
                        self.selectable_builder(ui, "P", 2);
                        self.selectable_builder(ui, "R", 3);
                    });

                    let available_width = ui.available_width();

                    egui::Frame::new()
                        .fill(Color32::from_rgb(45, 45, 45))
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(
                                (available_width - 16.0).max(0.0),
                                (available_height - 16.0).max(0.0),
                            ));

                            match self.active_tab {
                                0 => draw_color_tab(ui, shape),
                                1 => draw_transform_tab(ui, shape),
                                _ => {}
                            }
                        });
                });
            });
    }

    pub fn selectable_builder(&mut self, ui: &mut egui::Ui, tab_name: &str, selection_number: u32) {
        let selected = self.active_tab == selection_number;

        let color = if selected {
            Color32::from_rgb(45, 45, 45)
        } else {
            Color32::from_rgb(15, 15, 15)
        };

        if ui
            .add_sized(
                [40.0, 40.0],
                egui::Button::new(tab_name)
                    .selected(self.active_tab == selection_number)
                    .fill(color),
            )
            .clicked()
        {
            self.active_tab = selection_number;
        }
    }
}

fn draw_color_tab(ui: &mut egui::Ui, shape: &mut Shape) {
    let color = &mut shape.material.color;

    let mut rgba = Color32::from_rgba_unmultiplied(
        (color.x * 255.0) as u8,
        (color.y * 255.0) as u8,
        (color.z * 255.0) as u8,
        (color.w * 255.0) as u8,
    );

    ui.vertical(|ui| {
        ui.label("Color");

        if ui.color_edit_button_srgba(&mut rgba).changed() {
            shape.material.color = glm::Vec4::new(
                rgba.r() as f32 / 255.0,
                rgba.g() as f32 / 255.0,
                rgba.b() as f32 / 255.0,
                rgba.a() as f32 / 255.0,
            );
        }
    });
}

fn draw_transform_tab(ui: &mut egui::Ui, shape: &mut Shape) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;

        draw_vec3(ui, "Position", &mut shape.transform.position, 0.05);
        draw_vec3(ui, "Rotation", &mut shape.transform.rotation, 0.05);
        draw_vec3(ui, "Scale", &mut shape.transform.scale, 0.05);

        let mut proprties_changed = false;
        if let Properties::Round(properties) = &mut shape.properties {
            ui.label("Radius");
            proprties_changed |= ui
                .add_sized(
                    [ui.available_width(), 30.0],
                    egui::DragValue::new(&mut properties.radius)
                        .speed(0.05)
                        .range(0.01..=1000.0),
                )
                .changed();

            ui.label("Segments");
            proprties_changed |= ui
                .add_sized(
                    [ui.available_width(), 30.0],
                    egui::DragValue::new(&mut properties.segments)
                        .speed(1.0)
                        .range(3..=128),
                )
                .changed();
        }

        if proprties_changed {
            shape.rebuild_mesh();
        }
    });
}

fn draw_vec3(ui: &mut egui::Ui, label: &str, value: &mut glm::Vec3, speed: f64) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        ui.label(label);

        for (i, axis) in ["X", "Y", "Z"].iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(*axis);
                ui.add_sized(
                    [ui.available_width(), 30.0],
                    egui::DragValue::new(&mut value[i]).speed(speed),
                );
            });
        }
    });
}

fn format_vec3(v: [f32; 3]) -> String {
    format!("{:2}, {:2}, {:2}", v[0], v[1], v[2])
}
