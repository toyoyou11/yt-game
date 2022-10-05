use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Shape for Sphere {
    fn get_center_of_mass(&self) -> na::Point3<f32> {
        na::Point3::origin()
    }
    fn get_inertia_tensor(&self) -> na::Matrix3<f32> {
        let mut tensor = na::Matrix3::zeros();
        tensor[(0, 0)] = 2.0 * self.radius * self.radius / 5.0;
        tensor[(1, 1)] = 2.0 * self.radius * self.radius / 5.0;
        tensor[(2, 2)] = 2.0 * self.radius * self.radius / 5.0;
        tensor
    }
    fn get_inv_inertia_tensor(&self) -> na::Matrix3<f32> {
        let mut tensor = na::Matrix3::zeros();
        tensor[(0, 0)] = 5.0 / (2.0 * self.radius * self.radius);
        tensor[(1, 1)] = 5.0 / (2.0 * self.radius * self.radius);
        tensor[(2, 2)] = 5.0 / (2.0 * self.radius * self.radius);
        tensor
    }
    fn build_aabb(&self, pos: &na::Isometry3<f32>) -> AABB {
        let max_local = na::Point3::new(self.radius, self.radius, self.radius);
        let max = pos.translation.transform_point(&max_local);
        let min = pos.translation.transform_point(&(-max_local));
        AABB { min, max }
    }
}
