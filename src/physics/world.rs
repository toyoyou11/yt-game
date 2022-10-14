use super::*;
use crate::math::*;
use generational_arena as ga;

#[derive(Debug, Clone, Copy)]
pub struct RigidBodyId {
    inner: ga::Index,
}

impl RigidBodyId {
    fn new(inner: ga::Index) -> Self {
        Self { inner }
    }
}

#[derive(Debug)]
pub struct PhysicsWorld {
    bodies: ga::Arena<(RigidBody, BVHLeafId)>,
    bvh: BVH<AABB, RigidBodyId>,
    gravity: Vector3,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        let bodies = ga::Arena::new();
        let bvh = BVH::new();
        let gravity = Vector3::new(0.0, -9.8, 0.0);
        Self {
            bodies,
            bvh,
            gravity,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for (body_id, (body, bounding_id)) in &mut self.bodies {
            let body_id = RigidBodyId::new(body_id);
            if body.get_inv_mass() != 0.0 {
                body.apply_force_world(&(self.gravity / body.get_inv_mass()));
            }
            if body.update(delta_time) {
                self.bvh.remove(*bounding_id);
                let new_bounding = body.get_aabb();
                let new_bounding_id = self.bvh.insert(body_id, new_bounding);
                *bounding_id = new_bounding_id;
            }
        }
        let mut overlaps = Vec::new();
        self.bvh.get_overlaps(&mut overlaps);

        for (body_id1, body_id2) in &overlaps {
            if let (Some((body1, _)), Some((body2, _))) =
                self.bodies.get2_mut(body_id1.inner, body_id2.inner)
            {
                if let Some(contact) = body1.contact(body2) {
                    resolve_contact(body1, body2, &contact);
                }
            } else {
                panic!();
            }
        }

        println!("{:?}", self.bvh);
    }

    pub fn insert(&mut self, rigid_body: RigidBody) -> RigidBodyId {
        let bounding = rigid_body.get_aabb();
        let rigid_id = RigidBodyId::new(
            self.bodies
                .insert((rigid_body, (ga::Index::from_raw_parts(0, 0),))),
        );

        let bounding_id = self.bvh.insert(rigid_id, bounding);
        self.bodies.get_mut(rigid_id.inner).unwrap().1 = bounding_id;
        rigid_id
    }

    pub fn remove(&mut self, id: RigidBodyId) -> Option<RigidBody> {
        self.bodies.remove(id.inner).map(|b| b.0)
    }

    pub fn get(&self, id: RigidBodyId) -> Option<&RigidBody> {
        self.bodies.get(id.inner).map(|b| &b.0)
    }
    pub fn get_mut(&mut self, id: RigidBodyId) -> Option<&mut RigidBody> {
        self.bodies.get_mut(id.inner).map(|b| &mut b.0)
    }
}
