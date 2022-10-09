use super::*;
use crate::math::*;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub center: Point3,
    pub radii: Vector3,
}

impl AABB {
    pub fn new(center: Point3, radii: Vector3) -> Self {
        Self { center, radii }
    }

    pub fn new_min_max(min: Point3, max: Point3) -> Self {
        let center = ((min.coords + max.coords) * 0.5).into();
        let radii = (max - min) * 0.5;
        Self { center, radii }
    }
}

impl BoundingVolume for AABB {
    fn intersect(&self, other: &Self) -> bool {
        let offset = self.center - other.center;
        let r = self.radii + other.radii;
        if offset.x.abs() >= r.x || offset.y.abs() >= r.y || offset.z.abs() >= r.z {
            false
        } else {
            true
        }
    }
    fn expand_mut(&mut self, p: &Point3) {
        let offset = p - self.center;
        let offset_abs = offset.abs();
        if offset_abs.x > self.radii.x {
            self.center.x += offset.x * (offset_abs.x - self.radii.x) * 0.5 / offset_abs.x;
            self.radii.x = (offset_abs.x + self.radii.x) * 0.5;
        }
        if offset_abs.y > self.radii.y {
            self.center.y += offset.y * (offset_abs.y - self.radii.y) * 0.5 / offset_abs.y;
            self.radii.y = (offset_abs.y + self.radii.y) * 0.5;
        }
        if offset_abs.z > self.radii.z {
            self.center.z += offset.z * (offset_abs.z - self.radii.z) * 0.5 / offset_abs.z;
            self.radii.z = (offset_abs.z + self.radii.z) * 0.5;
        }
    }

    fn merge(&self, other: &Self) -> Self {
        let offset = other.center - self.center;
        let offset_abs = offset.abs();
        let rdiff = other.radii - self.radii;

        let mut ret = *self;

        // if larger box encloses small one along x.
        if offset_abs.x <= rdiff.x.abs() {
            if rdiff.x > 0.0 {
                ret.center.x = other.center.x;
                ret.radii.x = other.radii.x;
            }
        } else {
            ret.radii.x = (offset_abs.x + self.radii.x + other.radii.x) * 0.5;

            if offset_abs.x > 0.0 {
                ret.center.x += offset.x * ((ret.radii.x - self.radii.x) / offset_abs.x);
            }
        }
        // Same for y axis
        if offset_abs.y <= rdiff.y.abs() {
            if rdiff.y > 0.0 {
                ret.center.y = other.center.y;
                ret.radii.y = other.radii.y;
            }
        } else {
            ret.radii.y = (offset_abs.y + self.radii.y + other.radii.y) * 0.5;

            if offset_abs.y > 0.0 {
                ret.center.y += offset.y * ((ret.radii.y - self.radii.y) / offset_abs.y);
            }
        }
        // Same for z axis
        if offset_abs.z <= rdiff.z.abs() {
            if rdiff.z > 0.0 {
                ret.center.z = other.center.z;
                ret.radii.z = other.radii.z;
            }
        } else {
            ret.radii.z = (offset_abs.z + self.radii.z + other.radii.z) * 0.5;

            if offset_abs.z > 0.0 {
                ret.center.z += offset.z * ((ret.radii.z - self.radii.z) / offset_abs.z);
            }
        }
        ret
    }

    fn volume(&self) -> f32 {
        self.radii.x * self.radii.y * self.radii.z
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aabb_intersect() {
        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = b1;
        let expected = true;
        assert_eq!(b1.intersect(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(1.5, 1.5, 1.5), Vector3::new(0.5, 0.5, 0.5));
        let expected = false;
        assert_eq!(b1.intersect(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(1.5, 1.5, 1.5), Vector3::new(0.6, 0.6, 0.6));
        let expected = true;
        assert_eq!(b1.intersect(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(2.0, 0.0, 0.0), Vector3::new(0.5, 0.5, 0.5));
        let expected = false;
        assert_eq!(b1.intersect(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(0.0, 2.0, 0.0), Vector3::new(0.5, 0.5, 0.5));
        let expected = false;
        assert_eq!(b1.intersect(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(0.0, 0.0, 3.0), Vector3::new(0.5, 0.5, 0.5));
        let expected = false;
        assert_eq!(b1.intersect(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(0.0, 0.0, -3.0), Vector3::new(0.5, 0.5, 0.5));
        let expected = false;
        assert_eq!(b1.intersect(&b2), expected);
    }
    #[test]
    fn test_aabb_merge() {
        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = b1;
        let expected = b1;
        assert_eq!(b1.merge(&b2), expected);

        let b1 = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let b2 = AABB::new(Point3::new(1.5, 1.5, 1.5), Vector3::new(0.5, 0.5, 0.5));
        let expected = AABB::new(Point3::new(0.5, 0.5, 0.5), Vector3::new(1.5, 1.5, 1.5));
        assert_eq!(b1.merge(&b2), expected);

        let b1 = AABB::new(Point3::new(1.0, 2.0, 3.0), Vector3::new(10.0, 8.0, 9.0));
        let b2 = AABB::new(Point3::new(4.0, 3.0, 1.0), Vector3::new(1.0, 2.0, 1.0));
        let expected = b1;
        assert_eq!(b1.merge(&b2), expected);
        assert_eq!(b2.merge(&b1), expected);
    }
    #[test]
    fn test_aabb_expand() {
        let b = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let p = Point3::origin();
        let expected = b;
        assert_eq!(b.expand(&p), expected);

        let b = AABB::new(Point3::origin(), Vector3::new(1.0, 1.0, 1.0));
        let p = Point3::new(2.0, 1.5, -1.5);
        let expected = AABB::new_min_max(Point3::new(-1.0, -1.0, -1.5), Point3::new(2.0, 1.5, 1.0));
        assert_eq!(b.expand(&p), expected);
    }
}
