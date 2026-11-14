use wgpu::{
    Device, Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

pub struct ViewportRenderer {
    #[allow(dead_code)]
    pub texture: Texture,
    pub view: TextureView,
    #[allow(dead_code)]
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    pub id_texture: Texture,
    pub id_view: TextureView,
    #[allow(dead_code)]
    pub mask_texture: Texture,
    pub mask_view: TextureView,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl ViewportRenderer {
    pub fn new(device: &Device, format: TextureFormat, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Viewport"),
            size: Extent3d {
                depth_or_array_layers: 1,
                width,
                height,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("Viewport Depth"),
            size: Extent3d {
                depth_or_array_layers: 1,
                width,
                height,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

        let id_texture = device.create_texture(&TextureDescriptor {
            label: Some("Viewport Id"),
            size: Extent3d {
                depth_or_array_layers: 1,
                width,
                height,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R32Uint,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let id_view = id_texture.create_view(&TextureViewDescriptor::default());

        let mask_texture = device.create_texture(&TextureDescriptor {
            label: Some("Selection texture"),
            size: Extent3d {
                depth_or_array_layers: 1,
                width,
                height,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let mask_view = mask_texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view,
            depth_texture,
            depth_view,
            id_texture,
            id_view,
            format,
            width,
            height,
            mask_texture,
            mask_view,
        }
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        *self = Self::new(device, self.format, width, height)
    }
}
