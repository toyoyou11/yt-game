use super::*;
use crate::math::*;

pub fn contact_sphere_sphere(
    s1: &shape::Sphere,
    s2: &shape::Sphere,
    pos12: &Isometry3,
) -> Option<Contact> {
    // Returns None if objects are apart.
    let d2 = pos12.translation.vector.magnitude_squared();
    let r = s1.radius + s2.radius;
    if d2 >= r * r {
        return None;
    }

    // Calculate contact point.
    let normal1 = UnitVector3::new_normalize(pos12.translation.vector);
    let normal2 = pos12.inverse_transform_unit_vector(&-normal1);
    let point1 = (s1.radius * normal1.into_inner()).into();
    let point2 = (s2.radius * normal2.into_inner()).into();
    let separation_distance = d2.sqrt() - r;

    Some(Contact::new(
        point1,
        point2,
        normal1,
        normal2,
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
        let pos12 = Isometry3::translation(0.0, -100.5, 0.0);
        let contact = contact_sphere_sphere(&s1, &s2, &pos12);

        let expected = Contact {
            point1: Point3::new(0.0, -1.0, 0.0),
            point2: Point3::new(0.0, 100.0, 0.0),
            normal1: UnitVector3::new_normalize(Vector3::new(0.0, -1.0, 0.0)),
            normal2: UnitVector3::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
            separation_distance: -0.5,
            toi: 0.0,
        };

        assert_eq!(contact.unwrap(), expected);
    }
}
