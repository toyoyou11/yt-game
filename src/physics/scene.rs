use super::*;
use generational_arena as ga;

pub type RigidBodyId = (ga::Index,);
#[derive(Debug)]
pub struct Scene {
    bodies: ga::Arena<(RigidBody, BVHLeafId)>,
    bvh: BVH<AABB, RigidBodyId>,
}

impl Scene {
    pub fn new() -> Self {
        let bodies = ga::Arena::new();
        let bvh = BVH::new();
        Self { bodies, bvh }
    }

    pub fn update(&mut self, delta_time: f32) {
        for (body_id, (body, bounding_id)) in &mut self.bodies {
            if body.update(delta_time) {
                self.bvh.remove(*bounding_id);
                let new_bounding = body.get_aabb();
                let new_bounding_id = self.bvh.insert((body_id,), new_bounding);
                *bounding_id = new_bounding_id;
            }
        }
        let mut indices = Vec::with_capacity(self.bodies.len());
        for (i, _) in &self.bodies {
            indices.push(i);
        }

        let mut overlaps = Vec::new();
        self.bvh.get_overlaps(&mut overlaps);

        for (body_id1, body_id2) in &overlaps {
            if let (Some((body1, _)), Some((body2, _))) =
                self.bodies.get2_mut(body_id1.0, body_id2.0)
            {
                if let Some(contact) = body1.contact(body2) {
                    resolve_contact(body1, body2, &contact);
                }
            }
        }
    }

    pub fn insert(&mut self, rigid_body: RigidBody) -> RigidBodyId {
        let bounding = rigid_body.get_aabb();
        let rigid_id = ((self
            .bodies
            .insert((rigid_body, (ga::Index::from_raw_parts(0, 0),)))),);

        let bounding_id = self.bvh.insert(rigid_id, bounding);
        self.bodies.get_mut(rigid_id.0).unwrap().1 = bounding_id;
        rigid_id
    }

    pub fn remove(&mut self, id: RigidBodyId) -> Option<RigidBody> {
        self.bodies.remove(id.0).map(|b| b.0)
    }

    pub fn get(&self, id: RigidBodyId) -> Option<&RigidBody> {
        self.bodies.get(id.0).map(|b| &b.0)
    }
    pub fn get_mut(&mut self, id: RigidBodyId) -> Option<&mut RigidBody> {
        self.bodies.get_mut(id.0).map(|b| &mut b.0)
    }
}
