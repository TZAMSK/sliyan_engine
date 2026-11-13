pub mod camera;
pub mod shape;

use crate::{
    scene::shape::{
        three_dimensions::{cube::Cube, sphere::Sphere},
        two_dimensions::{circle::Circle, rectangle::Rectangle, triangle::Triangle},
        Shape, ShapeId, Shapes,
    },
    selection::{PlacementMode, SelectionState},
};

pub struct Scene {
    pub shapes: Shapes,
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

        self.shapes.insert(shape.id, shape);
    }

    pub fn remove_shape(&mut self, id: u64) -> bool {
        let Some(shape) = self.shapes.remove(&id) else {
            return false;
        };

        if let Some(parent) = shape.parent.and_then(|p| self.shapes.get_mut(&p)) {
            parent.children.retain(|c| *c != id);
        }

        for child_id in shape.children {
            if let Some(child) = self.shapes.get_mut(&child_id) {
                child.parent = None;
            }
        }

        true
    }

    pub fn find_shape_by_id(&self, id: u64) -> Option<&Shape> {
        self.shapes.get(&id)
    }

    pub fn find_shape_by_id_mut(&mut self, id: u64) -> Option<&mut Shape> {
        self.shapes.get_mut(&id)
    }

    fn detach(&mut self, id: ShapeId) {
        let old_parent = self.shapes.get_mut(&id).and_then(|s| s.parent.take());
        if let Some(p) = old_parent.and_then(|p| self.shapes.get_mut(&p)) {
            p.children.retain(|c| *c != id);
        }
    }

    fn is_descendant_of(&self, id: ShapeId, ancestor: ShapeId) -> bool {
        let mut current = self.shapes.get(&id).and_then(|s| s.parent);
        while let Some(p) = current {
            if p == ancestor {
                return true;
            }
            current = self.shapes.get(&p).and_then(|s| s.parent);
        }

        false
    }

    pub fn add_child(&mut self, parent_id: ShapeId, child_id: ShapeId) -> bool {
        if parent_id == child_id
            || !self.shapes.contains_key(&parent_id)
            || !self.shapes.contains_key(&child_id)
            || self.is_descendant_of(parent_id, child_id)
        {
            return false;
        }

        self.detach(child_id);
        self.shapes.get_mut(&child_id).unwrap().parent = Some(parent_id);
        self.shapes
            .get_mut(&parent_id)
            .unwrap()
            .children
            .push(child_id);

        true
    }
}
