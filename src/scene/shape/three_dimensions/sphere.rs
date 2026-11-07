use crate::scene::shape::round::RoundProperties;
use crate::scene::shape::{Properties, ShapeBehavior};
use crate::scene::shape::{Shape, ShapeType};

pub struct Sphere {
    pub shape: Shape,
}

impl Sphere {
    pub fn new(name: &str, position: glm::Vec3, color: glm::Vec4) -> Self {
        let mut shape = Shape::new(name.to_string(), position, color);

        shape.properties = Properties::Round(RoundProperties {
            radius: 1.0,
            segments: 16,
        });
        shape.shape_type = ShapeType::Sphere;

        let mut sphere = Self { shape };

        sphere.rebuild_mesh();
        sphere
    }

    pub fn rebuild(shape: &mut Shape) {
        let vertices = &mut shape.mesh.vertices;

        let Properties::Round(properties) = &shape.properties else {
            return;
        };

        let radius = properties.radius;
        let segments = properties.segments;

        vertices.clear();
        vertices.reserve(segments * segments * 6);

        for y in 0..segments {
            for x in 0..segments {
                let vert_at = |xi: usize, yi: usize| -> glm::Vec3 {
                    let u = xi as f32 / segments as f32;
                    let v = yi as f32 / segments as f32;
                    let phi = v * std::f32::consts::PI;
                    let theta = u * 2.0 * std::f32::consts::PI;

                    glm::Vec3::new(
                        radius * phi.sin() * theta.cos(),
                        radius * phi.cos(),
                        radius * phi.sin() * theta.sin(),
                    )
                };

                let tl = vert_at(x, y);
                let tr = vert_at(x + 1, y);
                let bl = vert_at(x, y + 1);
                let br = vert_at(x + 1, y + 1);

                vertices.push(tl);
                vertices.push(bl);
                vertices.push(tr);

                vertices.push(tr);
                vertices.push(bl);
                vertices.push(br);
            }
        }
    }
}

impl ShapeBehavior for Sphere {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Sphere
    }

    fn rebuild_mesh(&mut self) {
        Self::rebuild(&mut self.shape);
    }
}
