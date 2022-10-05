pub mod sphere;

use super::*;
use bvh::*;
pub use sphere::*;

use nalgebra as na;
use std::{any::Any, fmt::Debug};

#[derive(Debug, Clone)]
pub enum ShapeType {
    Sphere(Sphere),
}

impl ShapeType {
    pub fn get_center_of_mass(&self) -> na::Point3<f32> {
        self.as_shape().get_center_of_mass()
    }

    pub fn get_inertia_tensor(&self) -> na::Matrix3<f32> {
        self.as_shape().get_inertia_tensor()
    }
    pub fn get_inv_inertia_tensor(&self) -> na::Matrix3<f32> {
        self.as_shape().get_inv_inertia_tensor()
    }

    pub fn contact(&self, other: &Self, pos12: &na::Isometry3<f32>) -> Option<Contact> {
        match (self, other) {
            (Self::Sphere(s1), Self::Sphere(s2)) => contact_sphere_sphere(s1, s2, pos12),
            _ => None,
        }
    }

    pub fn build_aabb(&self, pos: &na::Isometry3<f32>) -> AABB {
        self.as_shape().build_aabb(pos)
    }

    pub fn as_shape(&self) -> &dyn Shape {
        match self {
            Self::Sphere(s) => s,
        }
    }
}

pub trait Shape {
    fn get_center_of_mass(&self) -> na::Point3<f32>;
    fn get_inertia_tensor(&self) -> na::Matrix3<f32>;
    fn get_inv_inertia_tensor(&self) -> na::Matrix3<f32> {
        self.get_inertia_tensor().try_inverse().unwrap()
    }
    fn build_aabb(&self, pos: &na::Isometry3<f32>) -> AABB;
}
