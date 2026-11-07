use crate::renderer::viewport_renderer::ViewportRenderer;

pub struct OutlineRenderer {
    pub composite_bind_group: wgpu::BindGroup,
    pub composite_bind_group_layout: wgpu::BindGroupLayout,
}

impl OutlineRenderer {
    pub fn new(device: &wgpu::Device, viewport: &ViewportRenderer) -> Self {
        let composite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Outline Composite Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Outline composite bind group"),
            layout: &composite_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&viewport.mask_view),
            }],
        });

        Self {
            composite_bind_group,
            composite_bind_group_layout,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, viewport: &ViewportRenderer) {
        self.composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Outline composite bind group"),
            layout: &self.composite_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&viewport.mask_view),
            }],
        });
    }
}
