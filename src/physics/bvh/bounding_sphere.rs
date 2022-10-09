use crate::math::*;
use crate::physics::bvh::BoundingVolume;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    pub center: Point3,
    pub radius: Float,
}

impl BoundingSphere {
    pub fn new(center: Point3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl BoundingVolume for BoundingSphere {
    fn intersect(&self, other: &Self) -> bool {
        let r = self.radius + other.radius;
        let d2 = (self.center - other.center).magnitude_squared();
        d2 < r * r
    }

    fn expand_mut(&mut self, p: &Point3) {
        let offset = p - self.center;
        let distance2 = offset.magnitude_squared();
        if distance2 <= self.radius * self.radius {
            return;
        }
        let distance = distance2.sqrt();
        self.center = self.center + offset * (distance - self.radius) * 0.5 / distance;
        self.radius = (distance + self.radius) * 0.5;
    }

    fn merge(&self, other: &Self) -> Self {
        let p12 = other.center - self.center;
        let d2 = p12.magnitude_squared();
        let rdiff = other.radius - self.radius;

        // if larger sphere encloses small one
        if rdiff * rdiff >= d2 {
            if rdiff > 0.0 {
                *other
            } else {
                *self
            }
        } else {
            let d = d2.sqrt();
            let radius = (d + self.radius + other.radius) * 0.5;

            let mut center = self.center;
            if d > 0.0 {
                center += p12 * ((radius - self.radius) / d);
            }
            Self { radius, center }
        }
    }

    fn volume(&self) -> Float {
        self.radius
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Point3;
    #[test]
    fn test_sphere_intersect() {
        let tests = [
            (
                BoundingSphere::new(Point3::origin(), 1.0),
                BoundingSphere::new(Point3::origin(), 1.0),
                true,
            ),
            (
                BoundingSphere::new(Point3::origin(), 1.0),
                BoundingSphere::new(Point3::new(2.0, 0.0, 0.0), 1.0),
                false,
            ),
            (
                BoundingSphere::new(Point3::origin(), 1.0),
                BoundingSphere::new(Point3::new(1.0, 0.0, 0.0), 1.0),
                true,
            ),
            (
                BoundingSphere::new(Point3::new(1.0, 1.0, 1.0), 0.5),
                BoundingSphere::new(Point3::new(0.0, 0.0, 0.0), 0.1),
                false,
            ),
        ];

        for t in tests {
            assert_eq!(t.0.intersect(&t.1), t.2);
        }
    }

    #[test]
    fn test_sphere_expand() {
        let s = BoundingSphere::new(Point3::origin(), 1.0);
        let p = Point3::new(1.0, 0.0, 0.0);
        let expected = s;
        assert_eq!(s.expand(&p), expected);
        let p = Point3::new(2.0, 0.0, 0.0);
        let expected = BoundingSphere::new(Point3::new(0.5, 0.0, 0.0), 1.5);
        assert_eq!(s.expand(&p), expected);
        let s = BoundingSphere::new(Point3::new(3.0, 0.0, 0.0), 1.0);
        let p = Point3::new(-1.0, 0.0, 0.0);
        let expected = BoundingSphere::new(Point3::new(1.5, 0.0, 0.0), 2.5);
        assert_eq!(s.expand(&p), expected);
    }

    #[test]
    fn test_sphere_merge() {
        let s1 = BoundingSphere::new(Point3::origin(), 1.0);
        let s2 = s1;
        let expected = s1;
        assert_eq!(s1.merge(&s2), expected);

        let s1 = BoundingSphere::new(Point3::origin(), 1.0);
        let s2 = BoundingSphere::new(Point3::new(1.0, 0.0, 0.0), 1.0);
        let expected = BoundingSphere::new(Point3::new(0.5, 0.0, 0.0), 1.5);
        assert_eq!(s1.merge(&s2), expected);

        let s1 = BoundingSphere::new(Point3::new(2.0, 0.0, 0.0), 0.5);
        let s2 = BoundingSphere::new(Point3::new(1.0, 0.0, 0.0), 1.0);
        let expected = BoundingSphere::new(Point3::new(1.25, 0.0, 0.0), 1.25);
        assert_eq!(s1.merge(&s2), expected);
    }
}
