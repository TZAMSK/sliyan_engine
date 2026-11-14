use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use wgpu::{BufferAddress, VertexBufferLayout, VertexStepMode};

use crate::scene::shape::{
    round::RoundProperties,
    three_dimensions::{cube::Cube, sphere::Sphere},
    two_dimensions::{circle::Circle, rectangle::Rectangle, triangle::Triangle},
};

mod round;
pub mod three_dimensions;
pub mod two_dimensions;

#[derive(Default)]
pub enum ShapeType {
    #[default]
    Triangle,
    Rectangle,
    Circle,
    Cube,
    Sphere,
}

pub trait ShapeBehavior {
    #[allow(dead_code)]
    fn shape_type(&self) -> ShapeType;
    fn rebuild_mesh(&mut self);
}

#[derive(Default)]
pub enum Properties {
    #[default]
    None,
    Round(RoundProperties),
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
pub type ShapeId = u64;
pub type Shapes = HashMap<ShapeId, Shape>;

#[derive(Default)]
pub struct Shape {
    pub id: ShapeId,
    pub name: String,
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,
    pub physics_body: PhysicsBody,
    pub properties: Properties,
    pub shape_type: ShapeType,

    pub children: Vec<ShapeId>,
    pub parent: Option<ShapeId>,
}

#[derive(Clone, Copy)]
pub enum TransformationType {
    Position,
    Rotation,
    Scale,
}

pub struct Transform {
    pub position: glm::Vec3,
    pub rotation: glm::Vec3,
    pub scale: glm::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glm::Vec3::new(0.0, 0.0, 0.0),
            rotation: glm::Vec3::new(0.0, 0.0, 0.0),
            scale: glm::Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

const VERTEX_ATTRIBS: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Uint32];

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<glm::Vec3>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_data: Vec<u8>,
}

impl Mesh {
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn vertex_data(&self) -> Vec<u8> {
        self.vertex_data.clone()
    }

    pub fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: (8 * std::mem::size_of::<f32>()) as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &VERTEX_ATTRIBS,
        }
    }

    pub fn upload_to_gpu(
        &mut self,
        device: &wgpu::Device,
        transform: &Transform,
        color: &glm::Vec4,
        id: u64,
    ) {
        if self.vertices.is_empty() {
            self.vertex_buffer = None;
            self.vertex_data.clear();
            return;
        }

        debug_assert!(id <= u32::MAX as u64);

        let translation = glm::translation(&transform.position);

        let rotation_x = glm::rotation(transform.rotation.x, &glm::vec3(1.0, 0.0, 0.0));
        let rotation_y = glm::rotation(transform.rotation.y, &glm::vec3(0.0, 1.0, 0.0));
        let rotation_z = glm::rotation(transform.rotation.z, &glm::vec3(0.0, 0.0, 1.0));

        let scaling = glm::scaling(&transform.scale);

        let model = translation * rotation_z * rotation_y * rotation_x * scaling;

        let mut raw: Vec<f32> = Vec::with_capacity(self.vertices.len() * 8);

        for v in &self.vertices {
            let position = model * glm::vec4(v.x, v.y, v.z, 1.0);

            raw.extend_from_slice(&[
                position.x,
                position.y,
                position.z,
                color.x,
                color.y,
                color.z,
                color.w,
                f32::from_bits(id as u32),
            ]);
        }
        self.vertex_data = bytemuck::cast_slice(&raw).to_vec();

        self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex buffer"),
            size: self.vertex_data.len() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }
}

pub struct Material {
    pub color: glm::Vec4,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: glm::Vec4::new(0.608, 0.608, 0.608, 1.0),
        }
    }
}

#[derive(Default)]
pub struct PhysicsBody {
    pub volume: u32,
    pub following_parent: bool,
    pub has_physics_added: bool,
}

impl PhysicsBody {
    pub fn calculate_volume(&self, mesh: &Mesh) -> f32 {
        let count = mesh.vertex_count();
        let vertices = &mesh.vertices;
        let mut volume = 0.0;

        if count < 3 || count % 3 != 0 {
            return 0.0;
        }

        for triangle in vertices.chunks_exact(3) {
            let a = triangle[0];
            let b = triangle[1];
            let c = triangle[2];

            volume += glm::dot(&a, &glm::cross(&b, &c));
        }

        (volume / 6.0).abs()
    }
}

impl Shape {
    fn new(name: String, position: glm::Vec3, color: glm::Vec4) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            name,
            transform: Transform {
                position,
                ..Default::default()
            },
            material: Material { color },
            ..Default::default()
        }
    }

    pub fn set_transformation(
        id: ShapeId,
        shapes: &mut Shapes,
        value: glm::Vec3,
        ty: TransformationType,
    ) {
        let (children, change) = {
            let Some(s) = shapes.get_mut(&id) else {
                return;
            };

            let change = match ty {
                TransformationType::Position => {
                    let delta = value - s.transform.position;

                    s.transform.position = value;
                    Some(delta)
                }
                TransformationType::Rotation => {
                    let delta = value - s.transform.rotation;

                    s.transform.rotation = value;
                    Some(delta)
                }
                TransformationType::Scale => {
                    let old_scale = s.transform.scale;
                    s.transform.scale = value;

                    if old_scale.x.abs() < f32::EPSILON
                        || old_scale.y.abs() < f32::EPSILON
                        || old_scale.z.abs() < f32::EPSILON
                    {
                        None
                    } else {
                        let delta = value.component_div(&old_scale);
                        Some(delta)
                    }
                }
            };

            (s.children.clone(), change)
        };

        let Some(change) = change else {
            return;
        };

        for child_id in children {
            let Some(child) = shapes.get(&child_id) else {
                continue;
            };
            if !child.physics_body.following_parent {
                continue;
            }

            let new_values = match ty {
                TransformationType::Position => child.transform.position + change,
                TransformationType::Rotation => child.transform.rotation + change,
                TransformationType::Scale => child.transform.scale.component_mul(&change),
            };

            Shape::set_transformation(child_id, shapes, new_values, ty);
        }
    }

    pub fn rebuild_mesh(&mut self) {
        match self.shape_type {
            ShapeType::Triangle => Triangle::rebuild(self),
            ShapeType::Rectangle => Rectangle::rebuild(self),
            ShapeType::Circle => Circle::rebuild(self),
            ShapeType::Cube => Cube::rebuild(self),
            ShapeType::Sphere => Sphere::rebuild(self),
        }
    }
}
