use crate::scene::shape::round::RoundProperties;
use crate::scene::shape::{Properties, ShapeBehavior};
use crate::scene::shape::{Shape, ShapeType};

pub struct Circle {
    pub shape: Shape,
}

impl Circle {
    pub fn new(name: &str, position: glm::Vec3, color: glm::Vec4) -> Self {
        let mut shape = Shape::new(name.to_string(), position, color);

        shape.properties = Properties::Round(RoundProperties {
            radius: 1.0,
            segments: 6,
        });
        shape.shape_type = ShapeType::Circle;

        let mut circle = Self { shape };

        circle.rebuild_mesh();
        circle
    }

    pub fn rebuild(shape: &mut Shape) {
        let vertices = &mut shape.mesh.vertices;

        let Properties::Round(properties) = &shape.properties else {
            return;
        };

        let radius = properties.radius;
        let segments = properties.segments;

        vertices.clear();
        vertices.reserve(segments * 3);

        let pi = std::f32::consts::PI;

        for i in 0..segments {
            let angle1 = (i as f32 / (segments) as f32) * 2.0 * pi;
            let angle2 = ((i + 1) as f32 / (segments) as f32) * 2.0 * pi;

            vertices.push(glm::Vec3::new(0.0, 0.0, 0.0));
            vertices.push(glm::Vec3::new(angle1.cos() * radius, angle1.sin(), 0.0));
            vertices.push(glm::Vec3::new(angle2.cos() * radius, angle2.sin(), 0.0));
        }
    }
}

impl ShapeBehavior for Circle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn rebuild_mesh(&mut self) {
        Self::rebuild(&mut self.shape);
    }
}
