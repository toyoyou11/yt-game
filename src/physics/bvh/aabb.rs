use super::*;
use nalgebra as na;
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: na::Point3<f32>,
    pub max: na::Point3<f32>,
}

impl BoundingVolume for AABB {
    fn intersect(&self, other: &Self) -> bool {
        if self.max.x < other.min.x || self.max.y < other.min.y || self.max.z < other.min.y {
            return false;
        }
        if self.min.x > other.max.x || self.min.y > other.max.y || self.min.z > other.max.z {
            return false;
        }

        true
    }

    fn expand(&self, p: &na::Point3<f32>) -> Self {
        let mut result = *self;
        if self.min.x > p.x {
            result.min.x = p.x;
        }
        if self.min.y > p.y {
            result.min.y = p.y;
        }
        if self.min.z > p.z {
            result.min.z = p.z;
        }

        if self.max.x < p.x {
            result.max.x = p.x;
        }
        if self.max.y < p.y {
            result.max.y = p.y;
        }
        if self.max.z < p.z {
            result.max.z = p.z;
        }
        result
    }
    fn expand_self(&mut self, p: &na::Point3<f32>) {
        if self.min.x > p.x {
            self.min.x = p.x;
        }
        if self.min.y > p.y {
            self.min.y = p.y;
        }
        if self.min.z > p.z {
            self.min.z = p.z;
        }

        if self.max.x < p.x {
            self.max.x = p.x;
        }
        if self.max.y < p.y {
            self.max.y = p.y;
        }
        if self.max.z < p.z {
            self.max.z = p.z;
        }
    }
}
