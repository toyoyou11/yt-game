use super::*;
use nalgebra as na;

pub fn contact_sphere_sphere(
    s1: &shape::Sphere,
    s2: &shape::Sphere,
    pos12: &na::Isometry3<f32>,
) -> Option<Contact> {
    // Returns None if objects are apart.
    let d2 = pos12.translation.vector.magnitude_squared();
    let r = s1.radius + s2.radius;
    if d2 >= r * r {
        return None;
    }

    // Calculate contact point.
    let normal = na::Unit::new_normalize(pos12.translation.vector);
    let point1 = (s1.radius * normal.into_inner()).into();
    let point2 = (s2.radius * pos12.inverse_transform_vector(&-normal)).into();
    let separation_distance = d2.sqrt() - r;

    Some(Contact::new(
        point1,
        point2,
        normal,
        separation_distance,
        0.0,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_contact_sphere_sphere() {
        let s1 = Sphere::new(1.0);
        let s2 = Sphere::new(100.0);
        let pos12 = na::Isometry3::translation(0.0, -100.5, 0.0);
        let contact = contact_sphere_sphere(&s1, &s2, &pos12);

        let expected = Contact {
            point1: na::Point3::new(0.0, -1.0, 0.0),
            point2: na::Point3::new(0.0, 100.0, 0.0),
            normal: na::UnitVector3::new_normalize(na::Vector3::new(0.0, -1.0, 0.0)),
            separation_distance: -0.5,
            toi: 0.0,
        };

        assert_eq!(contact.unwrap(), expected);
    }
}
