use nalgebra as na;

pub type Float = f32;
pub const PI: Float = std::f32::consts::PI;
pub const E: Float = std::f32::consts::E;
pub const FLOAT_MAX: Float = f32::MAX;
pub const FLOAT_MIN: Float = f32::MIN;
pub const FLOAT_MIN_POSITIVE: Float = f32::MIN_POSITIVE;

pub type Point2 = na::Point2<Float>;
pub type Point3 = na::Point3<Float>;

pub type Isometry2 = na::Isometry2<Float>;
pub type Isometry3 = na::Isometry3<Float>;

pub type Vector2 = na::Vector2<Float>;
pub type Vector3 = na::Vector3<Float>;
pub type Vector4 = na::Vector4<Float>;

pub type UnitVector2 = na::UnitVector2<Float>;
pub type UnitVector3 = na::UnitVector3<Float>;
pub type UnitVector4 = na::UnitVector4<Float>;

pub type Matrix2 = na::Matrix2<Float>;
pub type Matrix3 = na::Matrix3<Float>;
pub type Matrix4 = na::Matrix4<Float>;

pub type Translation2 = na::Translation2<Float>;
pub type Translation3 = na::Translation3<Float>;

pub type Rotation2 = na::Rotation2<Float>;
pub type Rotation3 = na::Rotation3<Float>;

pub type Quaternion = na::Quaternion<Float>;
pub type UnitQuaternion = na::UnitQuaternion<Float>;
pub type DualQuaternion = na::DualQuaternion<Float>;

pub type Scale2 = na::Scale2<Float>;
pub type Scale3 = na::Scale3<Float>;

pub fn project_vector_on_axis(v: &Vector3, axis: &Vector3) -> Vector3 {
    v.dot(&axis) * axis / axis.magnitude_squared()
}
