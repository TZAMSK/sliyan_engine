pub mod camera;
pub mod shape;

use std::{cell::RefCell, rc::Rc};

use crate::{
    scene::shape::{
        three_dimensions::{cube::Cube, sphere::Sphere},
        two_dimensions::{circle::Circle, rectangle::Rectangle, triangle::Triangle},
        Shape,
    },
    selection::{PlacementMode, SelectionState},
};
pub type ShapeRef = Rc<RefCell<Shape>>;

pub struct Scene {
    pub shapes: Vec<ShapeRef>,
}

impl Scene {
    pub fn add_shape(&mut self, shape_selected: &SelectionState, position: &glm::Vec3) {
        let shape = match shape_selected.placement_mode {
            PlacementMode::Triangle => {
                Triangle::new("Triangle", *position, glm::Vec4::new(0.5, 1.0, 1.0, 1.0)).shape
            }
            PlacementMode::Rectangle => {
                Rectangle::new("Rectangle", *position, glm::Vec4::new(1.0, 0.5, 1.0, 1.0)).shape
            }

            PlacementMode::Circle => {
                Circle::new("Circle", *position, glm::Vec4::new(1.0, 1.0, 0.5, 1.0)).shape
            }
            PlacementMode::Cube => {
                Cube::new("Cube", *position, glm::Vec4::new(1.0, 0.5, 0.5, 1.0)).shape
            }
            PlacementMode::Sphere => {
                Sphere::new("Sphere", *position, glm::Vec4::new(0.5, 0.5, 1.0, 1.0)).shape
            }
            _ => return,
        };

        self.shapes.push(Rc::new(RefCell::new(shape)));
    }

    pub fn remove_shape(&mut self, id: u64) -> bool {
        let Some(index) = self.shapes.iter().position(|shape| shape.borrow().id == id) else {
            return false;
        };

        let shape = self.shapes.remove(index);

        let (parent_id, children) = {
            let mut shape = shape.borrow_mut();
            (shape.parent.take(), std::mem::take(&mut shape.children))
        };

        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.find_shape_by_id(parent_id) {
                parent
                    .borrow_mut()
                    .children
                    .retain(|child| child.borrow().id != id);
            }
        }

        for child in children {
            child.borrow_mut().parent = None;
        }

        true
    }

    pub fn find_shape_by_id(&self, id: u64) -> Option<ShapeRef> {
        self.shapes
            .iter()
            .find(|shape| shape.borrow().id == id)
            .cloned()
    }

    pub fn find_shape_by_id_mut(&mut self, id: u64) -> Option<ShapeRef> {
        self.find_shape_by_id(id)
    }
}
