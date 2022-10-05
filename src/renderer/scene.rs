use super::*;
use camera::*;
use entity::Entity;
use generational_arena as ga;
use light::*;

pub type EntityIndex = (ga::Index,);

#[derive(Debug)]
pub struct Scene {
    entities: ga::Arena<Entity>,
    camera: Camera,
    lights: Lights,
}

impl Scene {
    /// Creates a new empty scene.
    pub fn new() -> Self {
        let entities = ga::Arena::new();
        let camera = Camera::new();
        let lights = Lights::new();
        Self {
            entities,
            camera,
            lights,
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> EntityIndex {
        (self.entities.insert(entity),)
    }

    pub fn remove_entity(&mut self, index: EntityIndex) {
        self.entities.remove(index.0);
    }

    pub fn iter_entities(&self) -> ga::Iter<Entity> {
        self.entities.iter()
    }

    pub fn iter_mut_entities(&mut self) -> ga::IterMut<Entity> {
        self.entities.iter_mut()
    }

    pub fn get_entity(&self, index: EntityIndex) -> Option<&Entity> {
        self.entities.get(index.0)
    }

    pub fn get_entity_mut(&mut self, index: EntityIndex) -> Option<&mut Entity> {
        self.entities.get_mut(index.0)
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
    pub fn get_lights(&self) -> &Lights {
        &self.lights
    }
    pub fn get_lights_mut(&mut self) -> &mut Lights {
        &mut self.lights
    }
}
