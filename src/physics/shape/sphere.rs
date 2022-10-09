use super::*;

use crate::math::*;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub radius: Float,
}

impl Sphere {
    pub fn new(radius: Float) -> Self {
        Self { radius }
    }
}

impl Shape for Sphere {
    fn get_center_of_mass(&self) -> Point3 {
        na::Point3::origin()
    }
    fn get_inertia_tensor(&self) -> Matrix3 {
        let mut tensor = Matrix3::zeros();
        tensor[(0, 0)] = 2.0 * self.radius * self.radius / 5.0;
        tensor[(1, 1)] = 2.0 * self.radius * self.radius / 5.0;
        tensor[(2, 2)] = 2.0 * self.radius * self.radius / 5.0;
        tensor
    }
    fn get_inv_inertia_tensor(&self) -> Matrix3 {
        let mut tensor = Matrix3::zeros();
        tensor[(0, 0)] = 5.0 / (2.0 * self.radius * self.radius);
        tensor[(1, 1)] = 5.0 / (2.0 * self.radius * self.radius);
        tensor[(2, 2)] = 5.0 / (2.0 * self.radius * self.radius);
        tensor
    }
    fn build_aabb(&self, pos: &Isometry3) -> AABB {
        let radii = Vector3::new(self.radius, self.radius, self.radius);
        let center = pos.translation.vector.into();
        AABB { center, radii }
    }
}
