pub mod app_state;
mod input;

use std::sync::Arc;

use transform_gizmo_egui::Gizmo;
use wgpu::hal::SurfaceError;
use winit::{application::ApplicationHandler, window::Window};

use crate::{
    app::{
        app_state::{AppState, ViewportState},
        input::{Drag, Input},
    },
    renderer::Renderer,
    scene::{camera::CameraManager, shape::three_dimensions::cube::Cube, Scene},
    selection::PlacementMode,
    ui::UIManager,
};

pub struct Application {
    scene: Option<Scene>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    state: Option<AppState>,
    pub ui: Option<UIManager>,
    camera: Option<CameraManager>,
    input: Input,
    gizmo: Gizmo,
}

impl Application {
    pub async fn new() -> Self {
        let cube = Cube::new(
            "Cube",
            glm::Vec3::new(10.0, 0.0, 0.0),
            glm::Vec4::new(0.50, 1.0, 1.0, 1.0),
        );

        let scene = Scene {
            shapes: vec![cube.shape],
        };

        Self {
            scene: Some(scene),
            window: None,
            renderer: None,
            state: None,
            ui: None,
            camera: None,
            input: Input::default(),
            gizmo: Gizmo::default(),
        }
    }

    pub async fn init(&mut self, window: Window) -> anyhow::Result<()> {
        let window = Arc::new(window);
        let scene = self.scene.as_mut().expect("Scene must be set before init");

        let mut renderer = Renderer::new(&window, &mut scene.shapes).await?;

        let camera = CameraManager::new(
            &renderer.device,
            window.inner_size().width,
            window.inner_size().height,
        );

        let state = AppState::new(&renderer.device);

        renderer.build_pipeline(&camera);

        let ui = UIManager::new(
            &window,
            &renderer.device,
            renderer.surface_format(),
            renderer.viewport_view(),
        );

        self.state = Some(state);
        self.renderer = Some(renderer);
        self.window = Some(window);
        self.camera = Some(camera);
        self.ui = Some(ui);

        Ok(())
    }

    pub fn on_viewport_clicked(scene: &mut Scene, state: &mut AppState, world_point: &glm::Vec3) {
        if let Some(selection) = state.selection.as_mut() {
            scene.add_shape(selection, world_point);
            selection.disarm_placement();
        }

        state.log_shape_added(&world_point);
    }

    fn redraw(&mut self) {
        let (Some(window), Some(renderer), Some(ui), Some(camera), Some(state), Some(scene)) = (
            self.window.as_ref(),
            self.renderer.as_mut(),
            self.ui.as_mut(),
            self.camera.as_mut(),
            self.state.as_mut(),
            self.scene.as_mut(),
        ) else {
            return;
        };

        state.update();

        let (w, h) = (state.viewport.width, state.viewport.height);
        if w > 0 && h > 0 && (w, h) != renderer.viewport_size() {
            renderer.resize_viewport(w, h);
            camera.camera.set_aspect(w as f32 / h as f32);
            ui.update_viewport_texture(&renderer.device, renderer.viewport_view());
        }

        self.input.update_drag(state.viewport.hovered);

        if self.input.drag() == Drag::Orbit {
            camera.camera.move_camera(self.input.mouse_delta(), 0.25);
        }

        if state.viewport.hovered {
            camera.camera.zoom_camera(self.input.scroll());
        }

        left_mouse_actions(&mut self.input, state, scene, renderer, camera);

        right_mouse_actions(&mut self.input, state, renderer, ui);

        camera.update(&renderer.queue);

        state.camera.position = camera.camera.position().into();
        state.camera.target = camera.camera.target().into();

        renderer.sync_shapes(&mut scene.shapes);

        match renderer.render(ui, state, window, camera, scene, &mut self.gizmo) {
            Ok(_) => {}
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
            }
            Err(e) => log::error!("Failed to render: {}", e),
        }

        self.input.end_frame();

        window.request_redraw();
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attr = Window::default_attributes()
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        let window = event_loop.create_window(attr).unwrap();

        pollster::block_on(self.init(window)).unwrap();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let (Some(window), Some(renderer), Some(ui)) = (
            self.window.as_ref(),
            self.renderer.as_mut(),
            self.ui.as_mut(),
        ) else {
            return;
        };

        let consumed = ui.handle_event(window, &event);

        //Window events handled even if egui consumes first
        if matches!(&event, winit::event::WindowEvent::CloseRequested) {
            event_loop.exit();
            return;
        }

        //Keyboard inputs allowed when egui not consuming it
        if !consumed && self.input.handle_keyboard(window, &event) {
            event_loop.exit();
            return;
        }

        self.input.handle_mouse(window, &event);

        match event {
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
            }
            winit::event::WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }
}

fn left_mouse_actions(
    input: &mut Input,
    state: &mut AppState,
    scene: &mut Scene,
    renderer: &Renderer,
    camera: &CameraManager,
) {
    let Some(mouse) = input.take_left_click() else {
        return;
    };

    let Some(local) = state.viewport.local_position(mouse) else {
        return;
    };

    let is_placement = state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.placement_mode != PlacementMode::None);

    if is_placement {
        if let Some(point) =
            input::mouse_local_to_world_space(&state.viewport, &camera.camera, mouse.x, mouse.y)
        {
            Application::on_viewport_clicked(scene, state, &point);
        }

        return;
    }

    if let Some(selection) = state.selection.as_mut() {
        let id = selection.picker.pick(
            &renderer.device,
            &renderer.queue,
            &renderer.viewport.id_texture,
            renderer.viewport_size(),
            local.x as u32,
            local.y as u32,
        );

        selection.picker.set_selected_shape_id(id);

        if id == 0 {
            state.console.append("Selection cleared");
        } else {
            state.console.append(&format!("Selected shape {id}"));
        }
    }
}

fn right_mouse_actions(
    input: &mut Input,
    state: &mut AppState,
    renderer: &Renderer,
    ui: &mut UIManager,
) {
    let Some(mouse) = input.take_right_click() else {
        return;
    };

    let Some(local) = state.viewport.local_position(mouse) else {
        return;
    };

    if let Some(selection) = state.selection.as_mut() {
        let id = selection.picker.pick(
            &renderer.device,
            &renderer.queue,
            &renderer.viewport.id_texture,
            renderer.viewport_size(),
            local.x as u32,
            local.y as u32,
        );

        if id != 0 {
            ui.open_shape_context_menu(id, mouse);
        }
    }
}
