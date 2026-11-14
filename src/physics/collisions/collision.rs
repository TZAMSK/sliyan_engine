use crate::physics::collisions::collider::Collider;

struct WorldCollider<'a> {
    collider: &'a dyn Collider,
    position: glm::Vec3,
    rotation: glm::Quat,
}

impl<'a> WorldCollider<'a> {
    pub fn new(collider: &'a dyn Collider) -> Self {
        Self {
            collider,

            // t = (0, 0, 0)
            //
            // Initial world-space translation
            position: glm::Vec3::zeros(),

            // q_identity = (w: 1, x: 0, y: 0, z: 0)
            //
            // Initial world-space rotation
            //
            // Reference:
            // https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
            rotation: glm::quat_identity(),
        }
    }

    /// Returns the collider's support point in the world space
    ///
    /// Complete equation
    ///
    /// S_world(d) = t + R * S_local(R⁻¹ * d)
    ///
    /// Where:
    /// - d       = world-space search direction
    /// - t       = world-space position
    /// - R       = world-space rotation
    /// - S_local = local-space support function
    ///
    /// Reference:
    /// https://en.wikipedia.org/wiki/Support_function
    /// https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
    pub fn support(&self, world_diretion: &glm::Vec3) -> glm::Vec3 {
        // q⁻¹ = conjugate(q) / ||q||²
        //
        // For a unit quaterion
        //
        // q⁻¹ = conjugate(q)
        //
        // Calculate the inverse rotation so that a vector
        // can be transformed from world space into a local space
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Quaternion#Conjugation,_the_norm,_and_reciprocal
        let inverse_rotation = glm::quat_inverse(&self.rotation);

        // d_local = R⁻¹ * d_world
        //
        // Quaternion representation
        //
        // d_local = R⁻¹ * d_world * q
        //
        // Transform the search direction into the collider's
        // local coordinate system
        //
        // The vector is interepreted as a pure quaterion
        // during quaterion multiplication
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Quaternions_and_spatial_rotation
        let local_direction = glm::quat_rotate_vec3(&inverse_rotation, world_diretion);

        // p_local = S_local(d_local)
        //
        // Find the furthest point of the collider along
        // the local-space search direction
        //
        // S_local(d) = argmax(p · d), for p ∈ C
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Support_function
        let local_support = self.collider.support(local_direction);

        // p_world = t + R * p_local
        //
        // Quaternion representation:
        //
        // p_world = t + R * p_local * q⁻¹
        //
        // Rotate the local support point into world space
        // then translate it by the collider's world position
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Rigid_transformation
        self.position + glm::quat_rotate_vec3(&self.rotation, &local_support)
    }
}

struct Contact {
    // world-space contact position
    point: glm::Vec3,

    // Unit-length collision normal
    //
    // ||n|| = 1
    //
    // Its orientation depends on the collision convention
    // such as pointing from collider A toward collider B
    normal: glm::Vec3,

    // penetration depth between collider
    //
    // depth >= 0
    //
    // Used during collision resolution to separate
    // interesting bodies
    depth: f32,
}

struct Vertex {
    // p = p_A - p_B
    //
    // Support point in the minkowski difference
    minkowski_different_point: glm::Vec3,

    // Original world-space support point on collider A
    single_point_shape_a: glm::Vec3,

    // Original world-space support point on collider B
    single_point_shape_b: glm::Vec3,
}

/// Returns a support vertex of the minkowski difference A - B
///
/// minkowski difference:
///
/// A - B = { a - b | a ∈ A, b ∈ B }
///
/// Support mapping:
///
/// S_(A-B)(d) = S_A(d) - S_B(d)
///
/// Reference:
/// https://en.wikipedia.org/wiki/Minkowski_addition
/// https://en.wikipedia.org/wiki/Gilbert%E2%80%93Johnson%E2%80%93Keerthi_distance_algorithm
fn minkowski(
    collider_a: &WorldCollider<'_>,
    collider_b: &WorldCollider<'_>,
    search_direction: glm::Vec3,
) -> Vertex {
    // p_A = S_A(d)
    //
    // Find the furthest point of the collider A along
    // the search_direction
    let point_a = collider_a.support(&search_direction);

    // p_B = S_B(-d)
    //
    // Find the furthest point of the collider B along
    // the opposite search_direction
    //
    // This is equivalent to finding the point of B
    // with the smallest projection along d
    let point_b = collider_b.support(&-search_direction);

    // p = p_A - p_B
    //
    // Calculate the support point of the minkowski difference
    //
    // Mathematical derivation
    //
    // max((a-b) · d)
    //     = max(a · d - b · d)
    //     = max(a · d) + max(b · (-d))
    //
    // Therefore
    //
    // S_(A-B)(d) = S_A(d) - S_B(-d)
    //
    // GJK uses this point to constrct a simplex and
    // determine whether the minkowski difference contains
    // the origin
    //
    // A ∩ B ≠ ∅  ⇔  0 ∈ (A - B)
    //
    // The original support points are preserved so EPA
    // can later reconstruct world-space contact positions
    //
    // Reference:
    // https://en.wikipedia.org/wiki/Gilbert%E2%80%93Johnson%E2%80%93Keerthi_distance_algorithm
    Vertex {
        minkowski_different_point: point_a - point_b,
        single_point_shape_a: point_a,
        single_point_shape_b: point_b,
    }
}
