pub mod sphere;
pub use sphere::*;

use crate::math::*;
use crate::physics::*;

#[derive(Debug, Clone)]
pub enum ShapeType {
    Sphere(Sphere),
}

impl ShapeType {
    pub fn get_center_of_mass(&self) -> Point3 {
        self.as_shape().get_center_of_mass()
    }

    pub fn get_inertia_tensor(&self) -> Matrix3 {
        self.as_shape().get_inertia_tensor()
    }
    pub fn get_inv_inertia_tensor(&self) -> Matrix3 {
        self.as_shape().get_inv_inertia_tensor()
    }

    pub fn contact(&self, other: &Self, pos12: &Isometry3) -> Option<Contact> {
        match (self, other) {
            (Self::Sphere(s1), Self::Sphere(s2)) => contact_sphere_sphere(s1, s2, pos12),
            _ => None,
        }
    }

    pub fn build_aabb(&self, pos: &Isometry3) -> AABB {
        self.as_shape().build_aabb(pos)
    }
    pub fn build_bounding_sphere(&self, pos: &Isometry3) -> BoundingSphere {
        self.as_shape().build_bounding_sphere(pos)
    }

    pub fn as_shape(&self) -> &dyn Shape {
        match self {
            Self::Sphere(s) => s,
        }
    }
}

pub trait Shape {
    fn get_center_of_mass(&self) -> Point3;
    fn get_inertia_tensor(&self) -> Matrix3;
    fn get_inv_inertia_tensor(&self) -> Matrix3 {
        self.get_inertia_tensor().try_inverse().unwrap()
    }
    fn build_aabb(&self, pos: &Isometry3) -> AABB;
    fn build_bounding_sphere(&self, pos: &Isometry3) -> BoundingSphere;
}
