use super::*;
use generational_arena as ga;

pub type RigidBodyIndex = (ga::Index,);
#[derive(Debug)]
pub struct Scene {
    bodies: ga::Arena<RigidBody>,
}

impl Scene {
    pub fn new() -> Self {
        let bodies = ga::Arena::new();
        Self { bodies }
    }

    pub fn update(&mut self, delta_time: f32) {
        for (_, b) in &mut self.bodies {
            b.update(delta_time);
        }
        let mut indices = Vec::with_capacity(self.bodies.len());
        for (i, _) in &self.bodies {
            indices.push(i);
        }

        for i in 0..indices.len() {
            for j in i + 1..indices.len() {
                let (i, j) = (indices[i], indices[j]);
                if let (Some(b1), Some(b2)) = self.bodies.get2_mut(i, j) {
                    if let Some(contact) = b1.contact(b2) {
                        resolve_contact(b1, b2, &contact);
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, rigid_body: RigidBody) -> RigidBodyIndex {
        (self.bodies.insert(rigid_body),)
    }

    pub fn remove(&mut self, index: RigidBodyIndex) -> Option<RigidBody> {
        self.bodies.remove(index.0)
    }

    pub fn get(&self, index: RigidBodyIndex) -> Option<&RigidBody> {
        self.bodies.get(index.0)
    }
    pub fn get_mut(&mut self, index: RigidBodyIndex) -> Option<&mut RigidBody> {
        self.bodies.get_mut(index.0)
    }
}
