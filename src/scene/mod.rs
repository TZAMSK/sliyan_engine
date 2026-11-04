pub mod camera;
pub mod shape;

use crate::{
    scene::shape::{
        three_dimensions::{cube::Cube, sphere::Sphere},
        two_dimensions::{circle::Circle, rectangle::Rectangle, triangle::Triangle},
        Shape,
    },
    selection::{PlacementMode, SelectionState},
};

pub struct Scene {
    pub shapes: Vec<Shape>,
}

impl Scene {
    pub fn add_shape(&mut self, shape_selected: &SelectionState, position: &glm::Vec3) {
        match shape_selected.placement_mode {
            PlacementMode::Triangle => self.shapes.push(
                Triangle::new("Triangle", *position, glm::Vec4::new(0.5, 1.0, 1.0, 1.0)).shape,
            ),
            PlacementMode::Rectangle => self.shapes.push(
                Rectangle::new("Rectangle", *position, glm::Vec4::new(1.0, 0.5, 1.0, 1.0)).shape,
            ),
            PlacementMode::Circle => self
                .shapes
                .push(Circle::new("Circle", *position, glm::Vec4::new(1.0, 1.0, 0.5, 1.0)).shape),
            PlacementMode::Cube => self
                .shapes
                .push(Cube::new("Cube", *position, glm::Vec4::new(1.0, 0.5, 0.5, 1.0)).shape),
            PlacementMode::Sphere => self
                .shapes
                .push(Sphere::new("Sphere", *position, glm::Vec4::new(0.5, 0.5, 1.0, 1.0)).shape),
            _ => {}
        }
    }

    pub fn remove_shape(&mut self, id: u64) -> bool {
        fn remove(shapes: &mut Vec<Shape>, id: u64) -> bool {
            if let Some(index) = shapes.iter().position(|shape| shape.id == id) {
                shapes.remove(index);
                return true;
            }

            for shape in shapes {
                if remove(&mut shape.children, id) {
                    return true;
                }
            }

            false
        }

        remove(&mut self.shapes, id)
    }

    pub fn find_shape_by_id_mut(&mut self, id: u64) -> Option<&mut Shape> {
        fn find(shapes: &mut [Shape], id: u64) -> Option<&mut Shape> {
            for shape in shapes.iter_mut() {
                if shape.id == id {
                    return Some(shape);
                }

                if let Some(found) = find(&mut shape.children, id) {
                    return Some(found);
                }
            }

            None
        }

        find(&mut self.shapes, id)
    }

    pub fn find_shape_by_id(&self, id: u64) -> Option<&Shape> {
        fn find(shapes: &[Shape], id: u64) -> Option<&Shape> {
            for shape in shapes {
                if shape.id == id {
                    return Some(shape);
                }

                if let Some(found) = find(&shape.children, id) {
                    return Some(found);
                }
            }

            None
        }

        find(&self.shapes, id)
    }
}
