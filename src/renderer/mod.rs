mod grid_axis_renderer;
mod outline_renderer;
mod pipeline;
mod viewport_renderer;

use std::sync::Arc;

use transform_gizmo_egui::Gizmo;
use wgpu::{
    Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Instance, LoadOp, Operations,
    PowerPreference, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RequestAdapterOptions, StoreOp, Surface, SurfaceColorSpace,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, hal::SurfaceError,
};
use winit::window::Window;

use crate::{
    app::app_state::AppState,
    renderer::{
        grid_axis_renderer::GridAxisRenderer, outline_renderer::OutlineRenderer,
        pipeline::RenderPipelineBuilder, viewport_renderer::ViewportRenderer,
    },
    scene::{
        Scene,
        camera::CameraManager,
        shape::{Mesh, Shape},
    },
    ui::UIManager,
};

pub struct Renderer {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    pipeline: Option<RenderPipeline>,
    pub viewport: ViewportRenderer,
    grid_pipeline: Option<RenderPipeline>,
    grid: GridAxisRenderer,
    outline_pipeline: Option<RenderPipeline>,
    outline: OutlineRenderer,
}

fn upload_shapes(shapes: &mut [Shape], device: &Device, queue: &Queue) {
    for shape in shapes {
        if !shape.mesh.vertices.is_empty() {
            shape
                .mesh
                .upload_to_gpu(device, &shape.transform, &shape.material.color, shape.id);

            if let Some(buffer) = &shape.mesh.vertex_buffer {
                queue.write_buffer(buffer, 0, &shape.mesh.vertex_data);
            }
        }

        upload_shapes(&mut shape.children, device, &queue);
    }
}

impl Renderer {
    pub async fn new(window: &Arc<Window>, shapes: &mut [Shape]) -> anyhow::Result<Self> {
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find adapter");

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        surface.configure(&device, &config);

        upload_shapes(shapes, &device, &queue);

        let viewport = ViewportRenderer::new(&device, surface_format, config.width, config.height);

        let grid = GridAxisRenderer::new(&device);

        let outline = OutlineRenderer::new(&device);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline: None,
            viewport,
            grid_pipeline: None,
            grid,
            outline_pipeline: None,
            outline,
        })
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(
        &mut self,
        ui: &mut UIManager,
        state: &mut AppState,
        window: &Window,
        camera: &mut CameraManager,
        scene: &mut Scene,
        gizmo: &mut Gizmo,
    ) -> Result<(), SurfaceError> {
        let (Some(pipeline), Some(grid_pipeline), Some(outline_pipeline)) =
            (&self.pipeline, &self.grid_pipeline, &self.outline_pipeline)
        else {
            return Ok(());
        };

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                return Err(SurfaceError::Lost);
            }
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Selected shape
        let selected_shape = state
            .selection
            .as_ref()
            .map(|selection| selection.picker.selected_shape_id)
            .filter(|&id| id != 0)
            .and_then(|id| scene.find_shape_by_id(id as u64));

        // Outline uniform update
        if let Some(shape) = selected_shape {
            let center = shape.transform.position;

            self.outline
                .update(&self.queue, [center.x, center.y, center.z], 1.05);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("3D Render Pass"),
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: &self.viewport.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.04,
                                g: 0.04,
                                b: 0.04,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                    Some(RenderPassColorAttachment {
                        view: &self.viewport.id_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0,
                            }),
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.viewport.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Grid
            render_pass.set_pipeline(grid_pipeline);
            render_pass.set_bind_group(0, &camera.camera_bind_group, &[]);
            self.grid.draw(&mut render_pass);

            // Outline
            if let Some(shape) = selected_shape {
                if let (Some(buffer), num_vertices) =
                    (&shape.mesh.vertex_buffer, &shape.mesh.vertex_count())
                {
                    render_pass.set_pipeline(outline_pipeline);
                    render_pass.set_bind_group(0, &camera.camera_bind_group, &[]);
                    render_pass.set_bind_group(1, &self.outline.bind_group, &[]);
                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                    render_pass.draw(0..*num_vertices as u32, 0..1);
                }
            }

            // Normal shapes
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &camera.camera_bind_group, &[]);

            for shape in &scene.shapes {
                if let (Some(buf), num_vertices) =
                    (&shape.mesh.vertex_buffer, &shape.mesh.vertex_count())
                {
                    render_pass.set_vertex_buffer(0, buf.slice(..));
                    render_pass.draw(0..*num_vertices as u32, 0..1);
                }
            }
        }

        let tranform_changed = ui.render(
            &self.device,
            &self.queue,
            &mut encoder,
            window,
            &view,
            state,
            scene,
            camera,
            gizmo,
        );

        if tranform_changed {
            self.sync_shapes(&mut scene.shapes);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }

    pub fn build_pipeline(&mut self, camera: &CameraManager) {
        // Normal expected shader
        self.pipeline = Some(
            RenderPipelineBuilder::new(&self.device, self.config.format)
                .with_shader(include_str!("shaders/shader.wgsl"))
                .with_vertex_layout(Mesh::vertex_layout())
                .with_binding_group_layout(&camera.camera_bind_group_layout)
                .with_id_format()
                .build()
                .expect("Couldn't create pipeline"),
        );

        // Grid shader
        self.grid_pipeline = Some(
            RenderPipelineBuilder::new(&self.device, self.config.format)
                .with_shader(include_str!("shaders/grid_shader.wgsl"))
                .with_vertex_layout(GridAxisRenderer::vertex_layout())
                .with_binding_group_layout(&camera.camera_bind_group_layout)
                .with_topology(wgpu::PrimitiveTopology::LineList)
                .with_id_format()
                .build()
                .expect("Couldn't create grid_pipeline"),
        );

        // Grid shader
        self.outline_pipeline = Some(
            RenderPipelineBuilder::new(&self.device, self.config.format)
                .with_shader(include_str!("shaders/outline_shader.wgsl"))
                .with_vertex_layout(Mesh::vertex_layout())
                .with_binding_group_layout(&camera.camera_bind_group_layout)
                .with_binding_group_layout(&self.outline.bind_group_layout)
                .with_cull_mode(wgpu::Face::Front)
                .with_depth_compare(wgpu::CompareFunction::LessEqual)
                .without_depth_write()
                .without_id_write()
                .with_id_format()
                .build()
                .expect("Couldn't create outline_pipeline"),
        );
    }

    pub fn sync_shapes(&mut self, shapes: &mut [Shape]) {
        upload_shapes(shapes, &self.device, &self.queue);
    }

    pub fn viewport_view(&self) -> &wgpu::TextureView {
        &self.viewport.view
    }

    pub fn resize_viewport(&mut self, width: u32, height: u32) {
        self.viewport.resize(&self.device, width, height)
    }

    pub fn viewport_size(&self) -> (u32, u32) {
        (self.viewport.width, self.viewport.height)
    }
}
