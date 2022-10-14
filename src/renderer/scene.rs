use crate::renderer::*;
use entity::Entity;
use generational_arena as ga;
use light::*;

#[derive(Debug, Clone, Copy)]
pub struct EntityId {
    inner: ga::Index,
}

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

    pub fn add_entity(&mut self, entity: Entity) -> EntityId {
        EntityId {
            inner: self.entities.insert(entity),
        }
    }

    pub fn remove_entity(&mut self, index: EntityId) {
        self.entities.remove(index.inner);
    }

    pub fn iter_entities(&self) -> ga::Iter<Entity> {
        self.entities.iter()
    }

    pub fn iter_mut_entities(&mut self) -> ga::IterMut<Entity> {
        self.entities.iter_mut()
    }

    pub fn get_entity(&self, index: EntityId) -> Option<&Entity> {
        self.entities.get(index.inner)
    }

    pub fn get_entity_mut(&mut self, index: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(index.inner)
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
    pub fn set_ambient_light(&mut self, ambient_light: AmbientLight) {
        self.lights.ambient_light = ambient_light;
    }
    pub fn get_ambient_light(&self) -> &AmbientLight {
        &self.lights.ambient_light
    }
    pub fn set_directional_light(&mut self, directional_light: DirectionalLight) {
        self.lights.directional_light = directional_light;
    }
    pub fn get_directional_light(&self) -> &DirectionalLight {
        &self.lights.directional_light
    }
    pub fn get_directional_light_mut(&mut self) -> &mut DirectionalLight {
        &mut self.lights.directional_light
    }

    pub fn insert_point_light(&mut self, p: PointLight) -> PointLightId {
        self.lights.insert_point_light(p)
    }

    pub fn iter_point_lights(&self) -> ga::Iter<PointLight> {
        self.lights.point_lights.iter()
    }

    pub fn get_point_light(&self, id: PointLightId) -> Option<&PointLight> {
        self.lights.get_point_light(id)
    }
    pub fn get_point_light_mut(&mut self, id: PointLightId) -> Option<&mut PointLight> {
        self.lights.get_point_light_mut(id)
    }
}
