use egui::Options;

use crate::selection::{PlacementMode, SelectionState};

#[derive(Debug)]
pub struct AppState {
    pub viewport: ViewportState,
    pub camera: CameraState,
    pub selection: Option<SelectionState>,
    pub console: Console,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            viewport: ViewportState::default(),
            camera: CameraState::default(),
            selection: None,
            console: Console::new(),
        }
    }
}

impl AppState {
    pub fn new(device: &wgpu::Device) -> Self {
        let selection = SelectionState::new(device);

        Self {
            selection: Some(selection),
            ..Default::default()
        }
    }

    pub fn update(&mut self) {
        // Update any time-dependent state
    }

    pub fn log_shape_added(&mut self, position: &glm::Vec3) {
        if let Some(selection) = &self.selection {
            match selection.placement_mode {
                PlacementMode::Triangle => self.console.append_shape_added("Triangle", &position),
                PlacementMode::Rectangle => self.console.append_shape_added("Rectangle", &position),
                PlacementMode::Circle => self.console.append_shape_added("Circle", &position),
                PlacementMode::Cube => self.console.append_shape_added("Cube", &position),
                PlacementMode::Sphere => self.console.append_shape_added("Sphere", &position),
                _ => {}
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ViewportState {
    pub width: u32,
    pub height: u32,
    pub position: glm::Vec2,
    pub zoom: f32,
    pub pan: [f32; 2],
    pub hovered: bool,
    pub clicked: bool,
    pub rect: [f32; 4],
}

impl ViewportState {
    pub fn local_position(&self, mouse: glm::Vec2) -> Option<glm::Vec2> {
        let local = mouse - self.position;

        (local.x >= 0.0
            && local.y >= 0.0
            && local.x < self.width as f32
            && local.y < self.height as f32)
            .then_some(local)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CameraState {
    pub position: [f32; 3],
    pub target: [f32; 3],
}

#[derive(Debug)]
pub struct Console {
    pub log: String,
}

impl Console {
    pub fn new() -> Self {
        Self {
            log: String::from("Ready.\n"),
        }
    }

    pub fn append(&mut self, content: &str) {
        self.log.push_str(content);
        self.log.push('\n');
    }

    pub fn clear(&mut self) {
        self.log.clear();
        self.log.push_str("Ready.");
    }

    pub fn append_shape_added(&mut self, shape: &str, position: &glm::Vec3) {
        self.log.push_str(&format!(
            "Added {} at ({:2}, {:2}, {:2}\n",
            shape, position.x, position.y, position.z,
        ));
    }
}
