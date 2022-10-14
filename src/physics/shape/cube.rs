use crate::math::*;
use crate::physics::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cube {
    pub half_extents: Vector3,
}

impl Cube {
    pub fn new(half_extents: Vector3) -> Self {
        Self { half_extents }
    }

    pub fn corners(&self) -> [Point3; 8] {
        [
            self.half_extents.into(),
            [
                self.half_extents.x,
                self.half_extents.y,
                -self.half_extents.z,
            ]
            .into(),
            [
                self.half_extents.x,
                -self.half_extents.y,
                self.half_extents.z,
            ]
            .into(),
            [
                self.half_extents.x,
                -self.half_extents.y,
                -self.half_extents.z,
            ]
            .into(),
            [
                -self.half_extents.x,
                self.half_extents.y,
                self.half_extents.z,
            ]
            .into(),
            [
                -self.half_extents.x,
                self.half_extents.y,
                -self.half_extents.z,
            ]
            .into(),
            [
                -self.half_extents.x,
                -self.half_extents.y,
                self.half_extents.z,
            ]
            .into(),
            [
                -self.half_extents.x,
                -self.half_extents.y,
                -self.half_extents.z,
            ]
            .into(),
        ]
    }

    pub fn corners_world(&self, position: &Isometry3) -> [Point3; 8] {
        self.corners()
            .map(|corner| position.transform_point(&corner))
    }
}

impl Shape for Cube {
    fn supporting_point(&self, dir: &UnitVector3, bias: Float) -> Point3 {
        let x = if dir.x > 0.0 {
            self.half_extents.x
        } else {
            -self.half_extents.x
        };
        let y = if dir.y > 0.0 {
            self.half_extents.y
        } else {
            -self.half_extents.y
        };
        let z = if dir.z > 0.0 {
            self.half_extents.z
        } else {
            -self.half_extents.z
        };
        Point3::new(x, y, z) + self.half_extents.normalize() * bias
    }

    fn get_center_of_mass(&self) -> Point3 {
        Point3::origin()
    }
    fn get_inertia_tensor(&self) -> Matrix3 {
        let dx2 = self.half_extents.x * self.half_extents.x;
        let dy2 = self.half_extents.y * self.half_extents.y;
        let dz2 = self.half_extents.z * self.half_extents.z;
        let tensor = Matrix3::new(
            (dy2 + dz2) / 3.0,
            0.0,
            0.0,
            0.0,
            (dx2 + dz2) / 3.0,
            0.0,
            0.0,
            0.0,
            (dx2 + dy2) / 3.0,
        );

        tensor
    }

    fn get_inv_inertia_tensor(&self) -> Matrix3 {
        let dx2 = self.half_extents.x * self.half_extents.x;
        let dy2 = self.half_extents.y * self.half_extents.y;
        let dz2 = self.half_extents.z * self.half_extents.z;
        let inv_tensor = Matrix3::new(
            3.0 / (dy2 + dz2),
            0.0,
            0.0,
            0.0,
            3.0 / (dx2 + dz2),
            0.0,
            0.0,
            0.0,
            3.0 / (dx2 + dy2),
        );

        inv_tensor
    }

    fn build_aabb(&self, pos: &Isometry3) -> AABB {
        let mut aabb = AABB::new(pos.translation.vector.into(), Vector3::zeros());
        for corner in &self.corners_world(pos) {
            aabb.expand_mut(corner);
        }
        aabb
    }

    fn build_bounding_sphere(&self, pos: &Isometry3) -> BoundingSphere {
        let radius = self.half_extents.magnitude();
        let center = pos.translation.vector.into();
        BoundingSphere { radius, center }
    }
}
