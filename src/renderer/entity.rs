use crate::math::*;
use std::sync::Arc;

use super::model::Model;
#[derive(Debug)]
pub struct Entity {
    pub name: String,
    pub position: Isometry3,
    pub scale: Scale3,
    pub model: Arc<Model>,
}

impl Entity {
    pub fn new(model: Arc<Model>) -> Self {
        let name = model.name.clone();
        let position = Isometry3::identity();
        let scale = Scale3::identity();
        Self {
            name,
            position,
            scale,
            model,
        }
    }
}
