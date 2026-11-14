use crate::{
    physics::collisions::collider::{BoxCollider, Collider, ConvexHullCollider},
    scene::shape::{Shape, ShapeType},
};

const CUBE_MESH_HALF_EXTENT: f32 = 1.0;
const FLAT_HALF_THICKNESS: f32 = 0.05;

struct ColliderPart {
    collider: Box<dyn Collider>,
    local_position: glm::Vec3,
    local_rotation: glm::Quat,
}

impl ColliderPart {
    pub fn new<C: Collider + 'static>(collider: C) -> Self {
        Self {
            collider: Box::new(collider),

            // t_local = (0, 0, 0)
            //
            // The collider is initially centered at the
            // rigid body's local origin
            local_position: glm::Vec3::zeros(),

            // q_identity = (w: 1, x: 0, y: 0, z: 0)
            //
            // The identity quaterion represents no rotation
            //
            // Reference:
            // https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
            local_rotation: glm::quat_identity(),
        }
    }
}

pub fn extract_vertex_points(shape: &Shape) -> Vec<glm::Vec3> {
    let data = shape.mesh.vertex_data();
    let count = shape.mesh.vertex_count();
    if data.is_empty() || count == 0 {
        return Vec::new();
    }

    // Copy the mesh's local-space vertex positions
    let mut points: Vec<glm::Vec3> = vec![];
    data[..count].to_vec();

    // Lexicographical ordering
    //
    // Compare x first, then y, then z
    //
    // Sorting places identical vertices next to each other
    // so they can be removed efficiently
    //
    // Reference:
    // https://en.wikipedia.org/wiki/Lexicographic_order
    points.sort_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then_with(|| a.y.total_cmp(&b.y))
            .then_with(|| a.z.total_cmp(&b.z))
    });

    // Remove consecutive identical vertices
    //
    // A mesh may contain duplicate positions because multiple
    // triangles can reference the same geometric point
    //
    // Duplicate points are unnecessary for support mapping
    points.dedup();

    points
}

pub fn thicken_if_flat(points: &mut Vec<glm::Vec3>) {
    if points.is_empty() {
        return;
    }

    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;

    for p in points.iter() {
        // z_min = min(z_min, p.z)
        // z_max = max(z_max, p.z)
        //
        // Find the minimum and maximum Z coordinates
        // of the geometry
        lo = lo.min(p.z);
        hi = hi.max(p.z);
    }

    // thicknes = z_max - z_min
    //
    // If the geometry already has sufficient thickness
    // along the Z axis, no modification is necessary
    if (hi - lo) > 1e-4f32 {
        return;
    }

    let mut out = Vec::with_capacity(points.len() * 2);

    for p in points.iter() {
        // p_top = p + (0, 0, h)
        // p_bottom = p - (0, 0, h)
        //
        // Extrude each vertex in both directions along Z
        //
        // For a perfectly flat shape
        //
        // total_thicknes = 2h
        //
        // This transforms planar geometry into a thin
        // three-dimensional volume
        //
        // Reference
        // https://files.openscad.org/documentation/manual/2D_to_3D_Extrusion.html
        out.push(glm::vec3(p.x, p.y, p.z + FLAT_HALF_THICKNESS));
        out.push(glm::vec3(p.x, p.y, p.z - FLAT_HALF_THICKNESS));
    }

    // Replace the original planar vertices with the
    // extruded vertices
    //
    // This helps avoid degenerate zero-volume simplices
    // during 3D GJK and EPA collision detection

    *points = out;
}

pub fn make_collider_parts(shape: &Shape) -> Vec<ColliderPart> {
    let scale = shape.transform.scale;

    match shape.shape_type {
        ShapeType::Cube => {
            // h = h_mesh * |s|
            //
            // Calculate the scaled half extents of the cube
            //
            // hx = h_mesh * |sx|
            // hy = h_mesh * |sy|
            // hz = h_mesh * |sz|
            //
            // Absolute values ensure that negative scaling
            // does not produce negative half extents
            //
            // Reference:
            // https://en.wikipedia.org/wiki/Scaling_(geometry)
            let half = scale.abs() * CUBE_MESH_HALF_EXTENT;
            vec![ColliderPart::new(BoxCollider::new(half))]
        }
        _ => {
            let mut points = extract_vertex_points(&shape);

            if points.is_empty() {
                return vec![];
            }

            for p in &mut points {
                // p_scaled = S * p
                //
                //     | sx  0   0  | * | px |
                // p = | 0   sy  0  | * | py |
                //     | 0   0   sz | * | pz |
                //
                // Equivalent component-wise calculation
                //
                // p_scaled = (px*sx, py*sy, pz*sz)
                //
                // Bake the shape's scale into its collider geometry
                //
                // Reference:
                // https://en.wikipedia.org/wiki/Scaling_(geometry)
                *p = p.component_mul(&scale);
            }

            // Give planar geometry sufficient volume
            // for three-dimensional collision detection
            thicken_if_flat(&mut points);

            // Construct a support-mapped convex collider
            // from the scaled vertex positions
            //
            // C = conv({p_1, p_2, ..., p_n})
            //
            // The support function of the convex hull can
            // be evaluated directly using the input vertices
            // Explicit hull faces are not required for GJK
            //
            // Reference:
            // https://en.wikipedia.org/wiki/Convex_hull
            vec![ColliderPart::new(ConvexHullCollider::new(points))]
        }
    }
}
