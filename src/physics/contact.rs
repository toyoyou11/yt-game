mod contact_cube_cube;
mod contact_cube_sphere;
mod contact_sphere_sphere;
use super::*;
use crate::math::*;

pub use self::{contact_cube_cube::*, contact_cube_sphere::*, contact_sphere_sphere::*};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    pub point1: Point3,
    pub point2: Point3,
    pub normal1: UnitVector3,
    pub normal2: UnitVector3,
    pub separation_distance: Float,
    pub toi: Float,
}

impl Contact {
    pub fn new(
        point1: Point3,
        point2: Point3,
        normal1: UnitVector3,
        normal2: UnitVector3,
        separation_distance: Float,
        toi: Float,
    ) -> Self {
        Self {
            point1,
            point2,
            normal1,
            normal2,
            separation_distance,
            toi,
        }
    }
    pub fn flip(&self) -> Self {
        Self {
            point1: self.point2,
            point2: self.point1,
            normal1: self.normal2,
            normal2: self.normal1,
            ..*self
        }
    }
}

pub fn resolve_contact(b1: &mut RigidBody, b2: &mut RigidBody, contact: &Contact) {
    if contact.separation_distance >= 0.0 {
        return;
    }
    let inv_mass1 = b1.get_inv_mass();
    let inv_mass2 = b2.get_inv_mass();
    if inv_mass1 == 0.0 && inv_mass2 == 0.0 {
        return;
    }
    let pt1 = b1.local_to_world_point(&contact.point1);
    let pt2 = b2.local_to_world_point(&contact.point2);
    let elasticity = b1.get_elasticity() * b2.get_elasticity();

    let inv_mass1 = b1.get_inv_mass();
    let inv_mass2 = b2.get_inv_mass();

    // Calculate collision impulse.

    let inv_inertia_world1 = b1.get_inv_inertia_tensor_world();
    let inv_inertia_world2 = b2.get_inv_inertia_tensor_world();

    let normal_world = b1.local_to_world_vector(&contact.normal1);

    let relative1 = pt1 - b1.get_center_of_mass_world();
    let relative2 = pt2 - b2.get_center_of_mass_world();

    let angularj1 = (inv_inertia_world1 * relative1.cross(&normal_world)).cross(&relative1);
    let angularj2 = (inv_inertia_world2 * relative2.cross(&normal_world)).cross(&relative2);
    let angular_factor = (angularj1 + angularj2).dot(&normal_world);

    let vel1 = b1.get_linear_velocity() + b1.get_angular_velocity().cross(&relative1);
    let vel2 = b2.get_linear_velocity() + b2.get_angular_velocity().cross(&relative2);

    let v12 = vel1 - vel2;
    let impulsej =
        (1.0 + elasticity) * v12.dot(&normal_world) / (inv_mass1 + inv_mass2 + angular_factor);
    let impulse = normal_world * impulsej;

    b1.apply_impulse_point_world(&(-impulse), &pt1);
    b2.apply_impulse_point_world(&impulse, &pt2);

    // Calculate friction impulse.
    let vnorm = normal_world * normal_world.dot(&v12);
    let vtang = v12 - vnorm;
    if vtang.magnitude_squared() > 0.00001 {
        let relative_vtang = vtang.normalize();
        let friction1 = b1.get_friction();
        let friction2 = b2.get_friction();
        let friction = friction1 * friction2;

        let inertia1 = (inv_inertia_world1 * relative1.cross(&relative_vtang)).cross(&relative1);
        let inertia2 = (inv_inertia_world2 * relative2.cross(&relative_vtang)).cross(&relative2);
        let inv_inertia = (inertia1 + inertia2).dot(&relative_vtang);

        let reduced_mass = 1.0 / (inv_mass1 + inv_mass2 + inv_inertia);
        let impulse_friction = vtang * reduced_mass * friction;

        b1.apply_impulse_point_world(&(-impulse_friction), &pt1);
        b2.apply_impulse_point_world(&impulse_friction, &pt2);
    }

    // Resolve penetration.
    let denom = 1.0 / (inv_mass1 + inv_mass2);
    let t1 = inv_mass1 * denom;
    let t2 = inv_mass2 * denom;
    let d = pt2 - pt1;
    b1.append_translation(&Translation3::from(t1 * d));
    b2.append_translation(&Translation3::from(-t2 * d));
}
