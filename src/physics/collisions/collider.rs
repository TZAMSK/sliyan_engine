pub trait Collider {
    /// Returns the furthest point of the collider in a given direction.
    ///
    /// S(d) = argmax(p · d), for p ∈ C
    ///
    /// Reference:
    /// https://en.wikipedia.org/wiki/Support_function
    fn support(&self, direction: glm::Vec3) -> glm::Vec3;

    /// Returns the intertia tensor of the collider for a given mass
    ///
    /// Reference:
    /// https://en.wikipedia.org/wiki/Moment_of_inertia#Inertia_tensor
    fn intertia(&self, mass: f32) -> glm::Mat3;

    /// Returns the radius of a sphere centered at the collider's
    /// local origin that encloses its entire geometry
    fn bounding_radius(&self) -> f32;
}

struct SphereCollider {
    radius: f32,
}

impl SphereCollider {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Collider for SphereCollider {
    fn support(&self, direction: glm::Vec3) -> glm::Vec3 {
        // ||d|| = sqrt(dx² + dy² + dz²)
        //
        // Calculate the Euclidean length of the search direction
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Euclidean_norm
        let len = glm::length(&direction);

        // S(d) = r * d / ||d||
        //
        // Normalize the search direction and multiply it by
        // the sphere's radius to obtain a point on its surface
        //
        // The resulting point maximizes the dot product p · d
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Support_function
        return if len > 1e-8f32 {
            direction * { self.radius / len }
        } else {
            // Avoid division by zero for a degenerate direction
            glm::Vec3::zeros()
        };
    }

    fn intertia(&self, mass: f32) -> glm::Mat3 {
        // I = (2/5) * m * r²
        //
        // Moment of inertia of a uniform solid sphere around
        // any axis passing through its center of mass
        //
        // Reference:
        // https://en.wikipedia.org/wiki/List_of_moments_of_inertia
        let i = 0.4 * mass * self.radius.powi(2);

        // I_tensor = I * Identity(3)
        //
        //     | I 0 0 |
        // I = | 0 I 0 |
        //     | 0 0 I |
        //
        // A sphere has the same moment of intertia around
        // every axis passing through its center
        glm::Mat3::identity() * i
    }

    fn bounding_radius(&self) -> f32 {
        // R = r
        //
        // The bounding sphere is identical to the sphere collider
        self.radius
    }
}

pub struct BoxCollider {
    half: glm::Vec3,
}

impl BoxCollider {
    pub fn new(half: glm::Vec3) -> Self {
        Self { half }
    }
}

impl Collider for BoxCollider {
    fn support(&self, direction: glm::Vec3) -> glm::Vec3 {
        // S(d) = (
        //     hx * sign(dx),
        //     hy * sign(dy),
        //     hz * sign(dz)
        // )
        //
        // Select the box corner that maximizes p · d
        // Each coordinate is selected independently accoring
        // to the sign of the corresponding direction component
        //
        // Here, a zero direction component selects +half
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Support_function

        let x = if direction.x >= 0.0 {
            self.half.x
        } else {
            -self.half.x
        };

        let y = if direction.y >= 0.0 {
            self.half.y
        } else {
            -self.half.y
        };

        let z = if direction.z >= 0.0 {
            self.half.z
        } else {
            -self.half.z
        };

        glm::vec3(x, y, z)
    }

    fn intertia(&self, mass: f32) -> glm::Mat3 {
        // Standard rectangular cuboid inertia:
        //
        // Ixx = (m/12) * (height² + depth²)
        // Iyy = (m/12) * (width² + width²)
        // Izz = (m/12) * (width² + height²)
        //
        // Since full dimensions are twice the half extents
        //
        // width = 2hx
        // height = 2hy
        // depth = 2hz
        //
        // Substitution gives the equations below
        //
        // Reference:
        // https://en.wikipedia.org/wiki/List_of_moments_of_inertia

        // Ixx = (m/3) * (hy² + hz²)
        //
        // Resistance to rotation around the X axis
        let m11 = mass / 3.0 * (self.half.y.powi(2) + self.half.z.powi(2));

        // Iyy = (m/3) * (hx² + hz²)
        //
        // Resistance to rotation around the Y axis
        let m22 = mass / 3.0 * (self.half.x.powi(2) + self.half.z.powi(2));

        // Ixx = (m/3) * (hx² + hy²)
        //
        // Resistance to rotation around the Z axis
        let m33 = mass / 3.0 * (self.half.x.powi(2) + self.half.y.powi(2));

        //     | Ixx   0    0  |
        // I = |  0   Iyy   0  |
        //     |  0    0   Izz |
        //
        // Construct the intertia tensor in the box's local
        // principal-axis coordinate system
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Moment_of_inertia#Inertia_tensor
        glm::mat3(m11, 0.0, 0.0, 0.0, m22, 0.0, 0.0, 0.0, m33)
    }

    fn bounding_radius(&self) -> f32 {
        // ||d|| = sqrt(dx² + dy² + dz²)
        //
        // Calculate the distance from the center of the box
        // to one of its corners using the Euclidean norm
        //
        // This is the radius of the enclosing bounding sphere
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Euclidean_norm

        glm::length(&self.half)
    }
}

pub struct ConvexHullCollider {
    radius: f32,
    points: Vec<glm::Vec3>,
    half: glm::Vec3,
}

impl ConvexHullCollider {
    pub fn new(points: Vec<glm::Vec3>) -> Self {
        let mut lo = glm::Vec3::repeat(f32::INFINITY);
        let mut hi = glm::Vec3::repeat(f32::NEG_INFINITY);
        let mut radius: f32 = 0.0;

        for p in &points {
            // lo = componentwise_min(lo, p)
            // hi = componentwise_max(hi, p)
            //
            // Find the minimum and maximum coordinates
            // of the axis-aligned bounding box (AABB)
            //
            // Reference:
            // https://en.wikipedia.org/wiki/Minimum_bounding_box
            lo = lo.inf(p);
            hi = hi.sup(p);

            // R = max (||p_i||), for every vertex p_i
            //
            // Find the maximum distance between the local
            // origin and any vertex of the convex hull
            //
            // This produces an origin-centered bounding sphere
            //
            // Reference:
            // https://en.wikipedia.org/wiki/Bounding_sphere
            radius = radius.max(glm::length(p));
        }

        // h = (hi - lo) / 2
        //
        // Calculate the half extents of the AABB
        // The difference hi - lo gives its full dimensions
        let half = if points.is_empty() {
            glm::Vec3::zeros()
        } else {
            (hi - lo) * 0.5
        };

        Self {
            radius,
            points,
            half,
        }
    }
}

impl Collider for ConvexHullCollider {
    fn support(&self, direction: glm::Vec3) -> glm::Vec3 {
        // S(d) = argmax(p_i · d)
        //
        // Find the vertex with the largest projection
        // along the given search direction
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Support_function
        let Some(mut best_point) = self.points.first() else {
            return glm::Vec3::zeros();
        };

        // p · d = px*dx + py*dy + pz*dz
        //
        // The dot product measures how far a point lies
        // along the search direction, up to the direction's length
        //
        // Reference:
        // https://en.wikipedia.org/wiki/Dot_product
        let mut best_dot = glm::dot(&best_point, &direction);

        for p in self.points.iter().skip(1) {
            let v = glm::dot(p, &direction);

            // Update the support point whenever a vertex
            // has a larger projection along the direction
            if v > best_dot {
                best_dot = v;
                best_point = p;
            }
        }

        *best_point
    }

    fn intertia(&self, mass: f32) -> glm::Mat3 {
        // I_hull ≈ I_box
        //
        // Approximate the convex hull's inertia tensor using
        // a uniform solid box with the same bounding dimensions
        //
        // This is an approximation, not the exact interta
        // tensor of an arbitrary convex polyhedron
        //
        // Reference for exact polyhedral mass propoerties:
        // https://www.geometrictools.com/Documentation/PolyhedralMassProperties.pdf
        BoxCollider::new(self.half).intertia(mass)
    }

    fn bounding_radius(&self) -> f32 {
        // R = max(||p_i||)
        //
        // Return the enclosing radius calculated during construction
        self.radius
    }
}
