use crate::scene::shape::ShapeBehavior;
use crate::scene::shape::{Shape, ShapeType};

pub struct Cube {
    pub shape: Shape,
}

impl Cube {
    pub fn new(name: &str, position: glm::Vec3, color: glm::Vec4) -> Self {
        let mut cube = Self {
            shape: Shape::new(name.to_string(), position, color),
        };
        cube.rebuild_mesh();
        cube
    }

    pub fn rebuild(shape: &mut Shape) {
        let vertices = &mut shape.mesh.vertices;

        vertices.clear();

        // -Z
        vertices.push(glm::Vec3::new(-1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, -1.0));

        vertices.push(glm::Vec3::new(-1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, -1.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, -1.0));

        // +Z
        vertices.push(glm::Vec3::new(-1.0, -1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, -1.0, 1.0));

        vertices.push(glm::Vec3::new(-1.0, -1.0, 1.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 1.0));

        // -Y
        vertices.push(glm::Vec3::new(-1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(-1.0, -1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, -1.0, 1.0));

        vertices.push(glm::Vec3::new(-1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, -1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, -1.0, -1.0));

        // +Y
        vertices.push(glm::Vec3::new(-1.0, 1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 1.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, 1.0));

        vertices.push(glm::Vec3::new(-1.0, 1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 1.0));

        // -X
        vertices.push(glm::Vec3::new(-1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, -1.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, 1.0));

        vertices.push(glm::Vec3::new(-1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(-1.0, 1.0, 1.0));
        vertices.push(glm::Vec3::new(-1.0, -1.0, 1.0));

        // +X
        vertices.push(glm::Vec3::new(1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, -1.0));

        vertices.push(glm::Vec3::new(1.0, -1.0, -1.0));
        vertices.push(glm::Vec3::new(1.0, -1.0, 1.0));
        vertices.push(glm::Vec3::new(1.0, 1.0, 1.0));
    }
}

impl ShapeBehavior for Cube {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Cube
    }

    fn rebuild_mesh(&mut self) {
        Self::rebuild(&mut self.shape);
    }
}
