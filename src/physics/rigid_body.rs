use super::*;
use crate::math::*;
#[derive(Debug, Clone)]
pub struct RigidBody {
    position: Isometry3,
    linear_velocity: Vector3,
    angular_velocity: Vector3,
    inv_mass: Float,
    elasticity: Float,
    damping: Float,
    anguar_damping: Float,
    friction: Float,
    shape: ShapeType,

    force_accum: Vector3,
    torque_accum: Vector3,
    position_modified: bool,
}

impl RigidBody {
    pub fn new(shape: ShapeType, inv_mass: Float) -> Self {
        Self {
            position: Isometry3::identity(),
            linear_velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            force_accum: Vector3::zeros(),
            torque_accum: Vector3::zeros(),
            shape,
            inv_mass,
            damping: 0.99,
            anguar_damping: 0.99,
            elasticity: 0.8,
            friction: 0.3,
            position_modified: false,
        }
    }

    /// Calculate new position. Returns true if its position gets modified.
    pub fn update(&mut self, delta_time: Float) -> bool {
        if self.inv_mass == 0.0 {
            let modified = self.position_modified;
            self.position_modified = false;
            return modified;
        }
        let acc = self.inv_mass * self.force_accum;
        self.linear_velocity += acc * delta_time;
        self.linear_velocity *= self.damping.powf(delta_time);
        let delta = self.linear_velocity * delta_time;
        self.position_modified = delta != Vector3::zeros();
        self.position
            .append_translation_mut(&Translation3::from(delta));

        // Update angular velocity.
        let rotation = self.position.rotation.to_rotation_matrix().into_inner();
        let inertia_tensor = rotation * self.shape.get_inertia_tensor() * rotation.transpose();
        let alpha = inertia_tensor.try_inverse().unwrap()
            * (self
                .angular_velocity
                .cross(&(inertia_tensor * self.angular_velocity)));
        self.angular_velocity += alpha * delta_time;
        self.angular_velocity *= self.anguar_damping.powf(delta_time);
        let delta = self.angular_velocity * delta_time;
        self.position_modified = delta != Vector3::zeros();
        self.position.append_rotation_wrt_point_mut(
            &UnitQuaternion::from_scaled_axis(delta),
            &self.get_center_of_mass_world(),
        );
        self.clear_force();

        let modified = self.position_modified;
        self.position_modified = false;
        modified
    }

    pub fn apply_force_point_local(&mut self, force: &Vector3, point: &Point3) {
        let (force, point) = (
            self.position.inverse_transform_vector(force),
            self.position.inverse_transform_point(point),
        );
        self.apply_force_point_world(&force, &point);
    }
    pub fn apply_force_point_world(&mut self, force: &Vector3, point: &Point3) {
        self.force_accum += force;
        let center = self.get_center_of_mass_world();
        let relative_pos = point - center;
        self.torque_accum += relative_pos.cross(force);
    }
    pub fn apply_impulse_point_local(&mut self, impulse: &Vector3, point: &Point3) {
        let (force, point) = (
            self.position.inverse_transform_vector(impulse),
            self.position.inverse_transform_point(point),
        );
        self.apply_impulse_point_world(&force, &point);
    }

    pub fn apply_impulse_point_world(&mut self, impulse: &Vector3, point: &Point3) {
        let center = self.get_center_of_mass_world();
        let relative_pos = point - center;
        let angular = relative_pos.cross(impulse);
        self.apply_impulse_world(impulse);
        self.apply_angular_impulse_world(&angular);
    }
    pub fn apply_force_world(&mut self, force: &Vector3) {
        self.force_accum += force;
    }
    pub fn apply_force_local(&mut self, force: &Vector3) {
        let w = self.position.transform_vector(force);
        self.force_accum += w;
    }

    pub fn apply_torque_world(&mut self, force: &Vector3) {
        self.torque_accum += force;
    }
    pub fn apply_torque_local(&mut self, force: &Vector3) {
        let force = self.position.transform_vector(force);
        self.torque_accum += force;
    }

    pub fn apply_impulse_world(&mut self, impulse: &Vector3) {
        self.linear_velocity += self.inv_mass * impulse;
    }
    pub fn apply_impulse_local(&mut self, impulse: &Vector3) {
        let w = self.position.transform_vector(impulse);
        self.linear_velocity += self.inv_mass * w;
    }
    pub fn apply_angular_impulse_world(&mut self, impulse: &Vector3) {
        self.angular_velocity += self.get_inv_inertia_tensor_world() * impulse;
    }
    pub fn apply_angular_impulse_local(&mut self, impulse: &Vector3) {
        let impulse = self.position.transform_vector(impulse);
        self.angular_velocity += self.get_inv_inertia_tensor_world() * impulse;
    }

    fn clear_force(&mut self) {
        self.force_accum = Vector3::zeros();
        self.torque_accum = Vector3::zeros();
    }

    pub fn get_inv_inertia_tensor_local(&self) -> Matrix3 {
        self.inv_mass * self.shape.get_inv_inertia_tensor()
    }
    pub fn get_inv_inertia_tensor_world(&self) -> Matrix3 {
        let rotation = self.position.rotation.to_rotation_matrix().into_inner();
        rotation * self.inv_mass * self.shape.get_inv_inertia_tensor() * rotation.transpose()
    }

    pub fn get_center_of_mass_local(&self) -> Point3 {
        self.shape.get_center_of_mass()
    }

    pub fn get_center_of_mass_world(&self) -> Point3 {
        self.position
            .transform_point(&self.shape.get_center_of_mass())
    }

    pub fn get_linear_velocity(&self) -> &Vector3 {
        &self.linear_velocity
    }
    pub fn set_linear_velocity(&mut self, v: &Vector3) {
        self.linear_velocity = *v;
    }
    pub fn get_angular_velocity(&self) -> &Vector3 {
        &self.angular_velocity
    }
    pub fn set_angular_velocity(&mut self, v: &Vector3) {
        self.angular_velocity = *v;
    }
    pub fn get_position(&self) -> &Isometry3 {
        &self.position
    }
    pub fn set_position(&mut self, iso: &Isometry3) {
        self.position_modified = true;
        self.position = *iso;
    }
    pub fn append_translation(&mut self, translation: &Translation3) {
        self.position.append_translation_mut(translation);
    }
    pub fn get_inv_mass(&self) -> Float {
        self.inv_mass
    }
    pub fn set_inv_mass(&mut self, inv_mass: Float) {
        self.inv_mass = inv_mass;
    }
    pub fn get_elasticity(&self) -> Float {
        self.elasticity
    }
    pub fn set_elasticity(&mut self, elasticity: Float) {
        self.elasticity = elasticity;
    }

    pub fn get_damping(&self) -> Float {
        self.damping
    }

    pub fn set_damping(&mut self, damping: Float) {
        self.damping = damping;
    }

    pub fn get_angular_damping(&self) -> Float {
        self.anguar_damping
    }

    pub fn set_angular_damping(&mut self, damping: Float) {
        self.anguar_damping = damping
    }

    pub fn get_friction(&self) -> Float {
        self.friction
    }

    pub fn set_friction(&mut self, friction: Float) {
        self.friction = friction
    }

    pub fn world_to_local_vector(&self, v: &Vector3) -> Vector3 {
        self.position.inverse_transform_vector(v)
    }
    pub fn local_to_world_vector(&self, v: &Vector3) -> Vector3 {
        self.position.transform_vector(v)
    }
    pub fn world_to_local_point(&self, p: &Point3) -> Point3 {
        self.position.inverse_transform_point(p)
    }
    pub fn local_to_world_point(&self, p: &Point3) -> Point3 {
        self.position.transform_point(p)
    }
    pub fn contact(&self, other: &Self) -> Option<Contact> {
        let pos = self.position.inverse() * other.position;
        self.shape.contact(&other.shape, &pos)
    }
}
