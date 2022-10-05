use nalgebra as na;

#[derive(Debug)]
pub struct Lights {
    pub ambient_light: AmbientLight,
    pub directional_light: DirectionalLight,
    pub point_lights: Vec<PointLight>,
}

impl Lights {
    pub fn new() -> Self {
        let ambient_light = AmbientLight::new([0.2, 0.2, 0.2]);
        let directional_light = DirectionalLight::new(
            [0.7, 0.7, 0.7],
            na::UnitVector3::new_normalize(na::Vector3::new(1.0, 1.0, 1.0)),
        );
        let point_lights = Vec::new();
        Self {
            ambient_light,
            directional_light,
            point_lights,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AmbientLight {
    pub color: [f32; 3],
}

impl AmbientLight {
    pub fn new(color: [f32; 3]) -> Self {
        Self { color }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub color: [f32; 3],
    pub direction: na::UnitVector3<f32>,
}

impl DirectionalLight {
    pub fn new(color: [f32; 3], direction: na::UnitVector3<f32>) -> Self {
        Self { color, direction }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub color: [f32; 3],
    pub radius: f32,
    pub point: na::Point3<f32>,
}

impl PointLight {
    pub fn new(color: [f32; 3], radius: f32, point: na::Point3<f32>) -> Self {
        Self {
            color,
            radius,
            point,
        }
    }
}
