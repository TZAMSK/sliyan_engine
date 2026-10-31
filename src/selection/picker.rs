use wgpu::{Buffer, CommandEncoderDescriptor, Device, Queue, Texture};

#[derive(Debug)]
pub struct Picker {
    pub selected_shape_id: u32,
    pick_buffer: Buffer,
}

impl Picker {
    pub fn new(device: &Device) -> Self {
        let pick_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback selection buffer"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        Self {
            selected_shape_id: 0,
            pick_buffer,
        }
    }

    pub fn set_selected_shape_id(&mut self, id: u32) {
        self.selected_shape_id = id
    }

    pub fn pick(
        &self,
        device: &Device,
        queue: &Queue,
        id_texture: &Texture,
        viewport_size: (u32, u32),
        x: u32,
        y: u32,
    ) -> u32 {
        let (w, h) = viewport_size;
        if x >= w || y >= h {
            return 0;
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Pick encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: id_texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.pick_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let slice = self.pick_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });

        let _ = device.poll(wgpu::PollType::wait_indefinitely());

        if !matches!(rx.recv(), Ok(Ok(()))) {
            return 0;
        }

        let id = {
            let Ok(data) = slice.get_mapped_range() else {
                self.pick_buffer.unmap();
                return 0;
            };
            u32::from_ne_bytes(data[0..4].try_into().unwrap())
        };

        self.pick_buffer.unmap();
        id
    }
}
