use crate::scene::shape::ShapeBehavior;
use crate::scene::shape::{Shape, ShapeType};

pub struct Rectangle {
    pub shape: Shape,
}

impl Rectangle {
    pub fn new(name: &str, position: glm::Vec3, color: glm::Vec4) -> Self {
        let mut rectangle = Self {
            shape: Shape::new(name.to_string(), position, color),
        };
        rectangle.rebuild_mesh();
        rectangle
    }

    pub fn rebuild(shape: &mut Shape) {
        let vertices = &mut shape.mesh.vertices;

        vertices.clear();

        vertices.push(glm::Vec3::new(-2.0, -1.0, 0.0));
        vertices.push(glm::Vec3::new(2.0, -1.0, 0.0));
        vertices.push(glm::Vec3::new(2.0, 1.0, 0.0));

        vertices.push(glm::Vec3::new(-2.0, -1.0, 0.0));
        vertices.push(glm::Vec3::new(2.0, 1.0, 0.0));
        vertices.push(glm::Vec3::new(-2.0, 1.0, 0.0));
    }
}

impl ShapeBehavior for Rectangle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Rectangle
    }

    fn rebuild_mesh(&mut self) {
        Self::rebuild(&mut self.shape);
    }
}
