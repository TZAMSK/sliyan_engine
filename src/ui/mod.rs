mod dimensions;
mod menu;
mod panels;

use egui_wgpu::wgpu::{
    CommandEncoder, Device, LoadOp, Operations, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};
use egui_wgpu::Renderer as EguiRenderer;
use egui_wgpu::RendererOptions;
use egui_winit::egui::{Context, PlatformOutput, RawInput};
use egui_winit::winit::window::Theme;
use egui_winit::winit::window::Window;
use egui_winit::State as EguiWinitState;
use transform_gizmo_egui::Gizmo;

use crate::editor::gizmo::GizmoApp;
use crate::scene::camera::CameraManager;
use crate::{app::app_state::AppState, scene::Scene};

pub struct UIManager {
    ctx: Context,
    platform: EguiWinitState,
    components: UIComponents,
    egui_renderer: EguiRenderer,
    viewport_texture_id: Option<egui::TextureId>,
    gizmo_app: GizmoApp,
}

struct UIComponents {
    viewport: panels::ViewportPanel,
    inspector: panels::InspectorPanel,
    scene: panels::ScenePanel,
    console: panels::ConsolePanel,
    menu_bar: menu::menu_bar::MenuPanel,
    context_menu: menu::context::ContextMenu,
    gizmo_settings_menu: menu::gizmo_settings::GizmoSettingsMenu,
    toolbar: panels::ToolbarPanel,
}

impl UIManager {
    pub fn new(
        window: &Window,
        device: &Device,
        surface_format: TextureFormat,
        viewport_view: &wgpu::TextureView,
    ) -> Self {
        let ctx = Context::default();
        let platform = EguiWinitState::new(
            ctx.clone(),
            egui_winit::egui::ViewportId::ROOT,
            window,
            None,
            Some(Theme::Dark),
            None,
        );

        let mut egui_renderer = EguiRenderer::new(
            device,
            surface_format,
            RendererOptions {
                ..Default::default()
            },
        );

        let viewport_texture_id = Some(egui_renderer.register_native_texture(
            device,
            viewport_view,
            wgpu::FilterMode::Linear,
        ));

        let gizmo_app = GizmoApp::new();

        Self {
            ctx,
            platform,
            components: UIComponents {
                viewport: panels::ViewportPanel::new(),
                inspector: panels::InspectorPanel::new(),
                scene: panels::ScenePanel::new(),
                console: panels::ConsolePanel::new(),
                menu_bar: menu::menu_bar::MenuPanel::new(),
                toolbar: panels::ToolbarPanel::new(),
                context_menu: menu::context::ContextMenu::new(),
                gizmo_settings_menu: menu::gizmo_settings::GizmoSettingsMenu::new(),
            },
            egui_renderer,
            viewport_texture_id,
            gizmo_app,
        }
    }

    pub fn open_shape_context_menu(&mut self, id: u32, mouse: glm::Vec2) {
        self.components
            .context_menu
            .open(id, egui::pos2(mouse.x, mouse.y));
    }

    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        self.platform.on_window_event(window, event).consumed
    }

    pub fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        state: &mut AppState,
        scene: &mut Scene,
        camera: &mut CameraManager,
        gizmo: &mut Gizmo,
    ) -> bool {
        self.components.menu_bar.show(ui, state);
        self.components.toolbar.show(ui, state, &mut camera.camera);
        self.components.console.show(ui, state);
        self.components.scene.show(ui, state);
        self.components.inspector.show(ui, state, scene);

        let mut transform_changed = false;

        if let Some(texture_id) = self.viewport_texture_id {
            transform_changed = self.components.viewport.show(
                ui,
                Some(texture_id),
                state,
                scene,
                camera,
                gizmo,
                &mut self.gizmo_app,
            );
        }

        let ppp = ui.ctx().pixels_per_point();
        let viewport_position = egui::pos2(
            state.viewport.position.x / ppp,
            state.viewport.position.y / ppp,
        );

        let over_gizmo_menu = self.components.gizmo_settings_menu.show(
            ui,
            viewport_position,
            &mut self.gizmo_app,
            &mut state.console,
        );

        let over_context_menu = self
            .components
            .context_menu
            .show(ui, scene, &mut state.console);

        let over_overlay = over_gizmo_menu || over_context_menu;
        state.viewport.pointer_over_overlay = over_overlay;

        if over_gizmo_menu {
            state.viewport.hovered = false;
            state.viewport.clicked = false;
        }

        transform_changed
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        texture_view: &TextureView,
        state: &mut AppState,
        scene: &mut Scene,
        camera: &mut CameraManager,
        gizmo: &mut Gizmo,
    ) -> bool {
        // Get raw input and run egui
        let input = self.take_input(window);
        let ctx = self.ctx().clone();

        let mut transform_changed = false;

        let mut full_output = ctx.run_ui(input, |ui| {
            transform_changed = self.render_ui(ui, state, scene, camera, gizmo);
        });

        // Handle platform output
        self.handle_platform_output(window, full_output.platform_output);

        // Tessellate paint jobs
        let paint_jobs = ctx.tessellate(full_output.shapes, 4.9);

        // Prepare screen descriptor
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in full_output.textures_delta.set.drain() {
            match image_delta.first() {
                Some(x) => self.egui_renderer.update_texture(device, queue, id, x),
                None => println!("Image failed to changed"),
            }
        }

        self.egui_renderer
            .update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let render_pass_static: &mut RenderPass<'static> =
            unsafe { std::mem::transmute(&mut render_pass) };

        // Render the UI
        self.egui_renderer
            .render(render_pass_static, &paint_jobs, &screen_descriptor);

        transform_changed
    }

    pub fn handle_platform_output(&mut self, window: &Window, platform_output: PlatformOutput) {
        self.platform
            .handle_platform_output(window, platform_output);
    }

    pub fn take_input(&mut self, window: &Window) -> RawInput {
        self.platform.take_egui_input(window)
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    pub fn set_viewport_texture(&mut self, texture_id: egui::TextureId) {
        self.viewport_texture_id = Some(texture_id)
    }

    pub fn update_viewport_texture(&mut self, device: &Device, view: &wgpu::TextureView) {
        if let Some(id) = self.viewport_texture_id {
            self.egui_renderer.update_egui_texture_from_wgpu_texture(
                device,
                view,
                wgpu::FilterMode::Linear,
                id,
            );
        }
    }
}
