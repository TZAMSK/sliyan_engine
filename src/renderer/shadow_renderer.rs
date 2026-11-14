use wgpu::util::DeviceExt;

use crate::scene::camera::opengl_to_wgpu_matrix;

pub struct ShadowRenderer {
    #[allow(dead_code)]
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,

    #[allow(dead_code)]
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,

    pub shadow_bind_group: wgpu::BindGroup,
    pub shadow_bind_group_layout: wgpu::BindGroupLayout,

    #[allow(dead_code)]
    pub sampler: wgpu::Sampler,
}

const SHADOW_SIZE: u32 = 2048;
const SHADOW_EXTENT: f32 = 30.0;
const LIGHT_NEAR: f32 = 5.0;
const LIGHT_FAR: f32 = 120.0;

impl ShadowRenderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Depth Texture"),
            size: wgpu::Extent3d {
                depth_or_array_layers: 1,
                width: SHADOW_SIZE,
                height: SHADOW_SIZE,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let eye = glm::vec3(25.0, 30.0, 40.0);
        let target = glm::vec3(0.0, 0.0, 0.0);
        let up = glm::vec3(0.0, 0.0, 1.0);

        let view = glm::look_at(&eye, &target, &up);

        let projection = glm::ortho(
            -SHADOW_EXTENT,
            SHADOW_EXTENT,
            -SHADOW_EXTENT,
            SHADOW_EXTENT,
            LIGHT_NEAR,
            LIGHT_FAR,
        );

        let light_view_proj = opengl_to_wgpu_matrix() * projection * view;

        let to_light = glm::normalize(&(eye - target));

        let mut light_data: Vec<f32> = light_view_proj.as_slice().to_vec();
        light_data.extend_from_slice(&[to_light.x, to_light.y, to_light.z, 0.0]);

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniform Buffer"),
            contents: &bytemuck::cast_slice(&light_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light bind group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow comparison Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow sampling Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow sampling bind group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            depth_texture,
            depth_view,
            light_buffer,
            light_bind_group,
            light_bind_group_layout,
            shadow_bind_group,
            shadow_bind_group_layout,
            sampler,
        }
    }
}
