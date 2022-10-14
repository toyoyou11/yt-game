use crate::math::*;
use crate::physics::*;
/// Returns contact information if they contact.
/// pos12 is sphere's position relative to cube.
pub fn contact_cube_sphere(cube: &Cube, sphere: &Sphere, pos12: &Isometry3) -> Option<Contact> {
    // calculate point on cube closest to sphere's center.
    let translation = &pos12.translation;
    let closest_point = Point3::new(
        if translation.x > cube.half_extents.x {
            cube.half_extents.x
        } else if translation.x < -cube.half_extents.x {
            -cube.half_extents.x
        } else {
            translation.x
        },
        if translation.y > cube.half_extents.y {
            cube.half_extents.y
        } else if translation.y < -cube.half_extents.y {
            -cube.half_extents.y
        } else {
            translation.y
        },
        if translation.z > cube.half_extents.z {
            cube.half_extents.z
        } else if translation.z < -cube.half_extents.z {
            -cube.half_extents.z
        } else {
            translation.z
        },
    );

    let distance_sqared = (closest_point.coords - pos12.translation.vector).magnitude_squared();
    if distance_sqared >= sphere.radius * sphere.radius || distance_sqared == 0.0 {
        return None;
    }

    let point1 = closest_point;
    let normal1 = UnitVector3::new_normalize(translation.vector - closest_point.coords);
    let normal2 = pos12.inverse_transform_unit_vector(&(-normal1));
    let point2 = (sphere.radius * normal2.into_inner()).into();
    let separation_distance = distance_sqared.sqrt() - sphere.radius;
    let toi = 0.0;
    Some(Contact {
        point1,
        point2,
        normal1,
        normal2,
        separation_distance,
        toi,
    })
}
