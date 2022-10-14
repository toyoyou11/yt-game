use crate::math::*;
use generational_arena as ga;

#[derive(Debug, Clone, Copy)]
pub struct PointLightId {
    inner: ga::Index,
}

#[derive(Debug)]
pub struct Lights {
    pub ambient_light: AmbientLight,
    pub directional_light: DirectionalLight,
    pub point_lights: ga::Arena<PointLight>,
}

impl Lights {
    pub fn new() -> Self {
        let ambient_light = AmbientLight::new([0.1, 0.1, 0.1]);
        let directional_light = DirectionalLight::new(
            [0.7, 0.7, 0.7],
            UnitVector3::new_normalize(Vector3::new(1.0, 1.0, 1.0)),
        );
        let point_lights = ga::Arena::new();
        Self {
            ambient_light,
            directional_light,
            point_lights,
        }
    }

    pub fn insert_point_light(&mut self, p: PointLight) -> PointLightId {
        PointLightId {
            inner: self.point_lights.insert(p),
        }
    }

    pub fn get_point_light(&self, id: PointLightId) -> Option<&PointLight> {
        self.point_lights.get(id.inner)
    }
    pub fn get_point_light_mut(&mut self, id: PointLightId) -> Option<&mut PointLight> {
        self.point_lights.get_mut(id.inner)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AmbientLight {
    pub color: [Float; 3],
}

impl AmbientLight {
    pub fn new(color: [Float; 3]) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub color: [Float; 3],
    pub direction: UnitVector3,
}

impl DirectionalLight {
    pub fn new(color: [Float; 3], direction: UnitVector3) -> Self {
        Self { color, direction }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub color: [Float; 3],
    pub radius: Float,
    pub point: Point3,
}

impl PointLight {
    pub fn new(color: [Float; 3], radius: Float, point: Point3) -> Self {
        Self {
            color,
            radius,
            point,
        }
    }
}
