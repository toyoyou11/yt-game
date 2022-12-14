use crate::math::*;

use crate::physics::*;
pub fn intersect_sphere_sphere(s1: &Sphere, s2: &Sphere, pos_12: &Isometry3) -> bool {
    let d2 = pos_12.translation.vector.magnitude_squared();
    let r = s1.radius + s2.radius;
    d2 >= r * r
}
