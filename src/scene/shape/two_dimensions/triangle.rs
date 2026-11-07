use crate::scene::shape::ShapeBehavior;
use crate::scene::shape::{Shape, ShapeType};

pub struct Triangle {
    pub shape: Shape,
}

impl Triangle {
    pub fn new(name: &str, position: glm::Vec3, color: glm::Vec4) -> Self {
        let mut triangle = Self {
            shape: Shape::new(name.to_string(), position, color),
        };
        triangle.rebuild_mesh();
        triangle
    }

    pub fn rebuild(shape: &mut Shape) {
        let vertices = &mut shape.mesh.vertices;

        vertices.clear();

        vertices.push(glm::Vec3::new(0.0, -1.0, 0.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, 0.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 0.0));
    }
}

impl ShapeBehavior for Triangle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Triangle
    }

    fn rebuild_mesh(&mut self) {
        Self::rebuild(&mut self.shape);
    }
}
