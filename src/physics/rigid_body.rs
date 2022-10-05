use super::*;
use nalgebra as na;
#[derive(Debug, Clone)]
pub struct RigidBody {
    position: na::Isometry3<f32>,
    linear_velocity: na::Vector3<f32>,
    angular_velocity: na::Vector3<f32>,
    force_accum: na::Vector3<f32>,
    torque_accum: na::Vector3<f32>,
    inv_mass: f32,
    elasticity: f32,
    damping: f32,
    anguar_damping: f32,
    friction: f32,
    shape: ShapeType,
}

impl RigidBody {
    pub fn new(shape: ShapeType, inv_mass: f32) -> Self {
        Self {
            position: na::Isometry3::identity(),
            linear_velocity: na::Vector3::zeros(),
            angular_velocity: na::Vector3::zeros(),
            force_accum: na::Vector3::zeros(),
            torque_accum: na::Vector3::zeros(),
            shape,
            inv_mass,
            damping: 0.99,
            anguar_damping: 0.99,
            elasticity: 0.8,
            friction: 0.1,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        let acc = self.inv_mass * self.force_accum;
        self.linear_velocity += acc * delta_time;
        self.linear_velocity *= self.damping.powf(delta_time);
        let delta = self.linear_velocity * delta_time;
        self.position
            .append_translation_mut(&na::Translation3::from(delta));

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
        self.position.append_rotation_wrt_point_mut(
            &na::UnitQuaternion::from_scaled_axis(delta),
            &self.get_center_of_mass_world(),
        );
        self.clear_force();
    }

    pub fn apply_force_point_local(&mut self, force: &na::Vector3<f32>, point: &na::Point3<f32>) {
        let (force, point) = (
            self.position.inverse_transform_vector(force),
            self.position.inverse_transform_point(point),
        );
        self.apply_force_point_world(&force, &point);
    }
    pub fn apply_force_point_world(&mut self, force: &na::Vector3<f32>, point: &na::Point3<f32>) {
        self.force_accum += force;
        let center = self.get_center_of_mass_world();
        let relative_pos = point - center;
        self.torque_accum += relative_pos.cross(force);
    }
    pub fn apply_impulse_point_local(
        &mut self,
        impulse: &na::Vector3<f32>,
        point: &na::Point3<f32>,
    ) {
        let (force, point) = (
            self.position.inverse_transform_vector(impulse),
            self.position.inverse_transform_point(point),
        );
        self.apply_impulse_point_world(&force, &point);
    }

    pub fn apply_impulse_point_world(
        &mut self,
        impulse: &na::Vector3<f32>,
        point: &na::Point3<f32>,
    ) {
        let center = self.get_center_of_mass_world();
        let relative_pos = point - center;
        let angular = relative_pos.cross(impulse);
        self.apply_impulse_world(impulse);
        self.apply_angular_impulse_world(&angular);
    }
    pub fn apply_force_world(&mut self, force: &na::Vector3<f32>) {
        self.force_accum += force;
    }
    pub fn apply_force_local(&mut self, force: &na::Vector3<f32>) {
        let w = self.position.transform_vector(force);
        self.force_accum += w;
    }

    pub fn apply_torque_world(&mut self, force: &na::Vector3<f32>) {
        self.torque_accum += force;
    }
    pub fn apply_torque_local(&mut self, force: &na::Vector3<f32>) {
        let force = self.position.transform_vector(force);
        self.torque_accum += force;
    }

    pub fn apply_impulse_world(&mut self, impulse: &na::Vector3<f32>) {
        self.linear_velocity += self.inv_mass * impulse;
    }
    pub fn apply_impulse_local(&mut self, impulse: &na::Vector3<f32>) {
        let w = self.position.transform_vector(impulse);
        self.linear_velocity += self.inv_mass * w;
    }
    pub fn apply_angular_impulse_world(&mut self, impulse: &na::Vector3<f32>) {
        self.angular_velocity += self.get_inv_inertia_tensor_world() * impulse;
    }
    pub fn apply_angular_impulse_local(&mut self, impulse: &na::Vector3<f32>) {
        let impulse = self.position.transform_vector(impulse);
        self.angular_velocity += self.get_inv_inertia_tensor_world() * impulse;
    }

    fn clear_force(&mut self) {
        self.force_accum = na::Vector3::zeros();
        self.torque_accum = na::Vector3::zeros();
    }

    pub fn get_inv_inertia_tensor_local(&self) -> na::Matrix3<f32> {
        self.inv_mass * self.shape.get_inv_inertia_tensor()
    }
    pub fn get_inv_inertia_tensor_world(&self) -> na::Matrix3<f32> {
        let rotation = self.position.rotation.to_rotation_matrix().into_inner();
        rotation * self.inv_mass * self.shape.get_inv_inertia_tensor() * rotation.transpose()
    }

    pub fn get_center_of_mass_local(&self) -> na::Point3<f32> {
        self.shape.get_center_of_mass()
    }

    pub fn get_center_of_mass_world(&self) -> na::Point3<f32> {
        self.position
            .transform_point(&self.shape.get_center_of_mass())
    }

    pub fn get_linear_velocity(&self) -> &na::Vector3<f32> {
        &self.linear_velocity
    }
    pub fn set_linear_velocity(&mut self, v: &na::Vector3<f32>) {
        self.linear_velocity = *v;
    }
    pub fn get_angular_velocity(&self) -> &na::Vector3<f32> {
        &self.angular_velocity
    }
    pub fn set_angular_velocity(&mut self, v: &na::Vector3<f32>) {
        self.angular_velocity = *v;
    }
    pub fn get_position(&self) -> &na::Isometry3<f32> {
        &self.position
    }
    pub fn get_position_mut(&mut self) -> &mut na::Isometry3<f32> {
        &mut self.position
    }
    pub fn set_position(&mut self, iso: &na::Isometry3<f32>) {
        self.position = *iso;
    }
    pub fn get_inv_mass(&self) -> f32 {
        self.inv_mass
    }
    pub fn set_inv_mass(&mut self, inv_mass: f32) {
        self.inv_mass = inv_mass;
    }
    pub fn get_elasticity(&self) -> f32 {
        self.elasticity
    }
    pub fn set_elasticity(&mut self, elasticity: f32) {
        self.elasticity = elasticity;
    }

    pub fn get_damping(&self) -> f32 {
        self.damping
    }

    pub fn set_damping(&mut self, damping: f32) {
        self.damping = damping;
    }

    pub fn get_angular_damping(&self) -> f32 {
        self.anguar_damping
    }

    pub fn set_angular_damping(&mut self, damping: f32) {
        self.anguar_damping = damping
    }

    pub fn get_friction(&self) -> f32 {
        self.friction
    }

    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction
    }

    pub fn world_to_local_vector(&self, v: &na::Vector3<f32>) -> na::Vector3<f32> {
        self.position.inverse_transform_vector(v)
    }
    pub fn local_to_world_vector(&self, v: &na::Vector3<f32>) -> na::Vector3<f32> {
        self.position.transform_vector(v)
    }
    pub fn world_to_local_point(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
        self.position.inverse_transform_point(p)
    }
    pub fn local_to_world_point(&self, p: &na::Point3<f32>) -> na::Point3<f32> {
        self.position.transform_point(p)
    }
    pub fn contact(&self, other: &Self) -> Option<Contact> {
        let pos = self.position.inverse() * other.position;
        self.shape.contact(&other.shape, &pos)
    }
}
