use nalgebra as na;
use std::sync::Arc;

use super::model::Model;
#[derive(Debug)]
pub struct Entity {
    pub name: String,
    pub position: na::Isometry3<f32>,
    pub scale: na::Scale3<f32>,
    pub model: Arc<Model>,
}

impl Entity {
    pub fn new(model: Arc<Model>) -> Self {
        let name = model.name.clone();
        let position = na::Isometry3::identity();
        let scale = na::Scale3::identity();
        Self {
            name,
            position,
            scale,
            model,
        }
    }
}
