pub mod aabb;
pub mod bounding_sphere;

use nalgebra as na;

pub use self::{aabb::*, bounding_sphere::*};

trait BoundingVolume: Sized {
    fn intersect(&self, other: &Self) -> bool;
    fn expand(&self, p: &na::Point3<f32>) -> Self;
    fn expand_self(&mut self, p: &na::Point3<f32>) {
        *self = self.expand(p);
    }
}
