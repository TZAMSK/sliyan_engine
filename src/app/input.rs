use std::collections::HashSet;

use winit::event::{MouseButton, MouseScrollDelta};
use winit::window::{Fullscreen, Window};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::app::app_state::ViewportState;
use crate::scene::camera::Camera;

#[derive(Default)]
pub struct Input {
    pressed: HashSet<KeyCode>,
    buttons: HashSet<MouseButton>,
    mouse_pos: glm::Vec2,
    mouse_delta: glm::Vec2,
    scroll: f32,
    drag: Drag,
    pending_left_click: Option<glm::Vec2>,
    pending_right_click: Option<glm::Vec2>,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum Drag {
    #[default]
    None,
    Orbit,
}

impl Input {
    pub fn mouse_button_down(&self, b: MouseButton) -> bool {
        self.buttons.contains(&b)
    }

    #[allow(dead_code)]
    pub fn mouse_pos(&self) -> glm::Vec2 {
        self.mouse_pos
    }

    pub fn mouse_delta(&self) -> glm::Vec2 {
        self.mouse_delta
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    pub fn drag(&self) -> Drag {
        self.drag
    }

    pub fn end_frame(&mut self) {
        self.mouse_delta = glm::vec2(0.0, 0.0);
        self.scroll = 0.0
    }

    pub fn update_drag(&mut self, viewport_hovered: bool) {
        if !self.mouse_button_down(MouseButton::Middle) {
            self.drag = Drag::None;
        } else if self.drag == Drag::None && viewport_hovered {
            self.drag = Drag::Orbit;
        }
    }

    pub fn handle_mouse(&mut self, _window: &Window, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new = glm::vec2(position.x as f32, position.y as f32);
                self.mouse_delta += new - self.mouse_pos;
                self.mouse_pos = new;
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.buttons.insert(*button);

                    match button {
                        MouseButton::Left => self.pending_left_click = Some(self.mouse_pos),
                        MouseButton::Right => self.pending_right_click = Some(self.mouse_pos),
                        _ => {}
                    }
                }
                ElementState::Released => {
                    self.buttons.remove(button);
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll += match delta {
                    MouseScrollDelta::LineDelta(_, y) => *y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 50.0,
                };
            }
            _ => {}
        }
        false
    }

    pub fn handle_keyboard(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let WindowEvent::KeyboardInput { event, .. } = event else {
            return false;
        };
        let PhysicalKey::Code(code) = event.physical_key else {
            return false;
        };

        match event.state {
            ElementState::Pressed => {
                if !event.repeat {
                    match code {
                        KeyCode::F11 => {
                            let fs = if window.fullscreen().is_some() {
                                None
                            } else {
                                Some(Fullscreen::Borderless(None))
                            };
                            window.set_fullscreen(fs);
                        }
                        KeyCode::Escape => return true,
                        _ => {}
                    }
                }
                self.pressed.insert(code);
            }

            ElementState::Released => {
                self.pressed.remove(&code);
            }
        }
        false
    }

    pub fn take_left_click(&mut self) -> Option<glm::Vec2> {
        self.pending_left_click.take()
    }

    pub fn take_right_click(&mut self) -> Option<glm::Vec2> {
        self.pending_right_click.take()
    }
}

pub fn mouse_local_to_world_space(
    viewport: &ViewportState,
    camera: &Camera,
    mouse_x: f32,
    mouse_y: f32,
) -> Option<glm::Vec3> {
    let viewport_size = glm::Vec2::new(viewport.width as f32, viewport.height as f32);

    if viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return None;
    }

    let local_x = mouse_x - viewport.position.x;
    let local_y = mouse_y - viewport.position.y;

    let ndc_x = (2.0 * local_x) / viewport_size.x - 1.0;
    let ndc_y = 1.0 - (2.0 * local_y) / viewport_size.y;

    let projection = camera.projection();
    let view = camera.view();

    let inverse_vp = (projection * view).try_inverse()?;
    let near_point = inverse_vp * glm::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let far_point = inverse_vp * glm::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    let ray_origin = near_point.xyz() / near_point.w;
    let ray_end = far_point.xyz() / far_point.w;
    let ray_direction = (ray_end - ray_origin).normalize();

    if ray_direction.z.abs() < 1e-5 {
        return None;
    }

    let t = -ray_origin.z / ray_direction.z;

    if t < 0.0 {
        return None;
    }

    Some(ray_origin + ray_direction * t)
}
