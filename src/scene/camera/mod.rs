use wgpu::util::DeviceExt;
pub mod view_mode;

pub struct CameraManager {
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

pub struct Camera {
    position: glm::Vec3,
    target: glm::Vec3,
    up: glm::Vec3,
    aspect: f32,
    fov: f32,
    znear: f32,
    zfar: f32,
    projection: glm::Mat4,
    view: glm::Mat4,
    ortho_scale: f32,
    camera_movement: CameraMovement,
}

pub struct CameraMovement {
    yaw: f32,
    pitch: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraManager {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let mut camera = Camera {
            position: glm::Vec3::new(5.0, -8.0, 4.0),
            target: glm::Vec3::zeros(),
            up: glm::Vec3::new(0.0, 0.0, 1.0),
            aspect: width as f32 / height as f32,
            fov: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 1000.0,
            ortho_scale: 12.0,
            projection: glm::Mat4::zeros(),
            view: glm::Mat4::zeros(),
            camera_movement: CameraMovement {
                yaw: 0.0,
                pitch: 0.0,
            },
        };

        camera.init_angle();

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&mut camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.camera_uniform.update_view_proj(&mut self.camera);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    } 
}

impl Camera {
    fn build_view_projection_matrix(&mut self) -> glm::Mat4 {
        let view = glm::look_at(&self.position, &self.target, &self.up);
        let proj = glm::perspective(self.aspect, self.fov, self.znear, self.zfar);

        self.projection = proj;
        self.view = view;

        opengl_to_wgpu_matrix() * proj * view
    }

    pub fn position(&self) -> glm::Vec3 {
        self.position
    }

    pub fn target(&self) -> glm::Vec3 {
        self.target
    }

    pub fn projection(&self) -> glm::Mat4 {
        self.projection
    }

    pub fn view(&self) -> glm::Mat4 {
        self.view
    }

    pub fn zoom_camera(&mut self, scroll: f32) {
        //let orthoZoom = self.ortho_scale - scroll_offset;
        let offset = self.position - self.target;
        let dist = (glm::length(&offset) * (1.0 - scroll * 0.1)).clamp(5.0, 500.0);
        self.position = self.target + glm::normalize(&offset) * dist;
    }

    pub fn move_camera(
        &mut self,
        mouse_delta: glm::Vec2,
        sensitivity: f32,
    ) {
        self.camera_movement.yaw -= mouse_delta.x * sensitivity;
        self.camera_movement.pitch += mouse_delta.y * sensitivity;
        self.camera_movement.pitch = self.camera_movement.pitch.clamp(-89.0, 89.0);

        let radius = (self.position - self.target).magnitude();

        let yaw_rad = self.camera_movement.yaw.to_radians();
        let pitch_rad = self.camera_movement.pitch.to_radians();

        self.position.x = self.target.x + radius * pitch_rad.cos() * yaw_rad.cos();
        self.position.y = self.target.y + radius * pitch_rad.cos() * yaw_rad.sin();
        self.position.z = self.target.z + radius * pitch_rad.sin();
        
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect
    }

    pub fn init_angle(&mut self) {
        let offset = self.position - self.target;
        let radius = offset.magnitude();

        self.camera_movement.yaw = offset.y.atan2(offset.x).to_degrees();
        self.camera_movement.pitch = (offset.z / radius).asin().to_degrees();
    }

    pub fn set_view(&mut self, positition: glm::Vec3, target: glm::Vec3, up: glm::Vec3) {
        self.position = positition;
        self.target = target;
        self.up = up
    }
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: glm::Mat4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &mut Camera) {
        self.view_proj = camera.build_view_projection_matrix().into()
    }
}

#[rustfmt::skip]
pub fn opengl_to_wgpu_matrix() -> glm::Mat4{ 
    glm::Mat4::from_columns(&[
        glm::vec4(1.0, 0.0, 0.0, 0.0),
        glm::vec4(0.0, 1.0, 0.0, 0.0),
        glm::vec4(0.0, 0.0, 0.5, 0.0),
        glm::vec4(0.0, 0.0, 0.5, 1.0),
    ])
}
