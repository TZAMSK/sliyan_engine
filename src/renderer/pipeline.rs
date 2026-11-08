use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, Device,
    FragmentState, FrontFace, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StencilState, TextureFormat,
    VertexBufferLayout, VertexState,
};

pub struct RenderPipelineBuilder<'a> {
    device: &'a Device,
    format: TextureFormat,
    shader: Option<String>,
    vertex_layout: Option<VertexBufferLayout<'static>>,
    binding_group_layout: Vec<wgpu::BindGroupLayout>,
    topology: PrimitiveTopology,
    id_format: Option<TextureFormat>,
    cull_mode: Option<wgpu::Face>,
    depth_enabled: bool,
    depth_write_enabled: bool,
    id_write_enabled: bool,
    depth_compare: CompareFunction,
    depth_only: bool,
    front_face: FrontFace,
    blend: Option<BlendState>,
    depth_bias: DepthBiasState,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(device: &'a Device, format: TextureFormat) -> Self {
        Self {
            device,
            format,
            shader: None,
            vertex_layout: None,
            binding_group_layout: Vec::new(),
            topology: PrimitiveTopology::TriangleList,
            id_format: None,
            cull_mode: None,
            depth_enabled: true,
            depth_write_enabled: true,
            depth_only: false,
            id_write_enabled: true,
            depth_compare: CompareFunction::Less,
            front_face: FrontFace::Ccw,
            blend: None,
            depth_bias: DepthBiasState::default(),
        }
    }

    pub fn with_shader(mut self, shader_source: &str) -> Self {
        self.shader = Some(shader_source.to_string());
        self
    }

    pub fn with_vertex_layout(mut self, layout: VertexBufferLayout<'static>) -> Self {
        self.vertex_layout = Some(layout);
        self
    }

    pub fn with_binding_group_layout(mut self, layout: &wgpu::BindGroupLayout) -> Self {
        self.binding_group_layout.push(layout.clone());
        self
    }

    pub fn with_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    pub fn with_id_format(mut self) -> Self {
        self.id_format = Some(TextureFormat::R32Uint);
        self
    }

    pub fn with_cull_mode(mut self, mode: wgpu::Face) -> Self {
        self.cull_mode = Some(mode);
        self
    }

    pub fn without_depth(mut self) -> Self {
        self.depth_enabled = false;
        self
    }

    pub fn without_depth_write(mut self) -> Self {
        self.depth_write_enabled = false;
        self
    }

    pub fn with_depth_only(mut self) -> Self {
        self.depth_only = true;
        self
    }

    pub fn without_id_write(mut self) -> Self {
        self.id_write_enabled = false;
        self
    }

    pub fn with_depth_compare(mut self, compare: CompareFunction) -> Self {
        self.depth_compare = compare;
        self
    }

    pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
        self.front_face = front_face;
        self
    }

    pub fn with_blend(mut self, blend: BlendState) -> Self {
        self.blend = Some(blend);
        self
    }

    pub fn with_depth_bias(mut self, constant: i32, slope_scale: f32) -> Self {
        self.depth_bias = DepthBiasState {
            constant,
            slope_scale,
            clamp: 0.0,
        };
        self
    }

    pub fn build(self) -> Result<RenderPipeline, PipelineError> {
        let shader = self.shader.ok_or(PipelineError::MissingShader)?;
        let vertex_layouts: Vec<VertexBufferLayout<'static>> =
            self.vertex_layout.into_iter().collect();

        let shader_module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(shader.into()),
        });

        let layout_refs: Vec<Option<&wgpu::BindGroupLayout>> =
            self.binding_group_layout.iter().map(Some).collect();

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &layout_refs,
                immediate_size: 0,
            });

        let mut targets = Vec::new();

        if !self.depth_only {
            targets.push(Some(ColorTargetState {
                format: self.format,
                blend: self.blend,
                write_mask: ColorWrites::ALL,
            }));

            if let Some(id_format) = self.id_format {
                targets.push(Some(ColorTargetState {
                    format: id_format,
                    blend: None,
                    write_mask: if self.id_write_enabled {
                        ColorWrites::ALL
                    } else {
                        ColorWrites::empty()
                    },
                }));
            }
        }

        let fragment = if self.depth_only {
            None
        } else {
            Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &targets,
                compilation_options: PipelineCompilationOptions::default(),
            })
        };

        Ok(self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: Some("vs_main"),
                    buffers: &vertex_layouts.iter().cloned().map(Some).collect::<Vec<_>>(),
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment,
                primitive: PrimitiveState {
                    topology: self.topology,
                    strip_index_format: None,
                    front_face: self.front_face,
                    cull_mode: self.cull_mode,
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: if self.depth_enabled {
                    Some(wgpu::DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: Some(self.depth_write_enabled),
                        depth_compare: Some(self.depth_compare),
                        stencil: StencilState::default(),
                        bias: self.depth_bias,
                    })
                } else {
                    None
                },
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            }))
    }
}

#[derive(Debug)]
pub enum PipelineError {
    MissingShader,
    MissingVertexLayout,
}
