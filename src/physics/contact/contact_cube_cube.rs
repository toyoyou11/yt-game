use crate::math::*;
use crate::physics::*;

/// Return the overlap along the axis.
/// Negative value indicates separation.
fn penetration_on_axis(cube1: &Cube, cube2: &Cube, pos12: &Isometry3, axis: &Vector3) -> Float {
    let axis = UnitVector3::new_normalize(*axis);
    let support1 = cube1.supporting_point(&axis, 0.0).coords;
    let len1 = support1.dot(&axis);
    let transformed_axis = pos12.inverse_transform_unit_vector(&axis);
    let support2 = cube2.supporting_point(&transformed_axis, 0.0).coords;
    let len2 = support2.dot(&transformed_axis);

    let distance = pos12.translation.vector.dot(&axis).abs();

    len1 + len2 - distance
}

fn get_contact_point(
    axis1: &Vector3,
    axis2: &Vector3,
    pt_on_edge1: &Point3,
    pt_on_edge2: &Point3,
) -> (Point3, Point3) {
    let to_st = pt_on_edge1 - pt_on_edge2;
    let dp_sta1 = axis1.dot(&to_st);
    let dp_sta2 = axis2.dot(&to_st);

    let sm1 = axis1.magnitude_squared();
    let sm2 = axis2.magnitude_squared();
    let dot_axes = axis1.dot(axis2);
    let denom = sm1 * sm2 - dot_axes * dot_axes;
    let (a, b) = if denom > 0.00001 {
        let denom = 1.0 / denom;
        (
            (dot_axes * dp_sta2 - sm2 * dp_sta1) * denom,
            (sm1 * dp_sta2 - dot_axes * dp_sta1) * denom,
        )
    } else {
        (0.0, 0.0)
    };
    (pt_on_edge1 + axis1 * a, pt_on_edge2 + axis2 * b)
}

pub fn contact_cube_cube(cube1: &Cube, cube2: &Cube, pos12: &Isometry3) -> Option<Contact> {
    let axes1 = Matrix3::identity();
    let axes2 = pos12.rotation.to_rotation_matrix().into_inner();
    let mut axes: [Vector3; 15] = [
        // face axes for cube1.
        axes1.column(0).into(),
        axes1.column(1).into(),
        axes1.column(2).into(),
        // face axes for cube2.
        axes2.column(0).into(),
        axes2.column(1).into(),
        axes2.column(2).into(),
        // edge-edge axes.
        axes1.column(0).cross(&axes2.column(0)),
        axes1.column(0).cross(&axes2.column(1)),
        axes1.column(0).cross(&axes2.column(2)),
        axes1.column(1).cross(&axes2.column(0)),
        axes1.column(1).cross(&axes2.column(1)),
        axes1.column(1).cross(&axes2.column(2)),
        axes1.column(2).cross(&axes2.column(0)),
        axes1.column(2).cross(&axes2.column(1)),
        axes1.column(2).cross(&axes2.column(2)),
    ];
    let mut best_overlap = FLOAT_MAX;
    let mut best_index = 0;

    for i in 0..15 {
        let axis = &mut axes[i];

        if axis.magnitude_squared() < 0.001 {
            continue;
        }
        axis.normalize_mut();

        let overlap = penetration_on_axis(cube1, cube2, pos12, axis);
        if overlap < 0.0 {
            return None;
        }
        if overlap < best_overlap {
            best_overlap = overlap;
            best_index = i;
        }
    }

    let mut axis = axes[best_index];
    if axis.dot(&pos12.translation.vector) < 0.0 {
        axis = -axis;
    }

    // vertex-face contact
    let contact = if best_index < 3 {
        // contact vertex in cube2's coordinates.
        let mut point2: Point3 = cube2.half_extents.into();
        if axes2.column(0).dot(&axis) > 0.0 {
            point2.x = -point2.x;
        }
        if axes2.column(1).dot(&axis) > 0.0 {
            point2.y = -point2.y;
        }
        if axes2.column(2).dot(&axis) > 0.0 {
            point2.z = -point2.z;
        }

        let normal1 = UnitVector3::new_normalize(axis);
        let point1 = pos12.transform_point(&point2) + best_overlap * normal1.into_inner();
        let normal2 = pos12.inverse_transform_unit_vector(&(-normal1));
        let separation_distance = -best_overlap;
        let toi = 0.0;

        let contact = Contact {
            point1,
            point2,
            normal1,
            normal2,
            separation_distance,
            toi,
        };
        Some(contact)
    } else if best_index < 6 {
        // contact vertex in cube1's coordinates.
        let mut point1: Point3 = cube1.half_extents.into();
        if axes1.column(0).dot(&axis) < 0.0 {
            point1.x = -point1.x;
        }
        if axes1.column(1).dot(&axis) < 0.0 {
            point1.y = -point1.y;
        }
        if axes1.column(2).dot(&axis) < 0.0 {
            point1.z = -point1.z;
        }

        let normal1 = UnitVector3::new_normalize(axis);
        let normal2 = pos12.inverse_transform_unit_vector(&(-normal1));
        let point2 = pos12.inverse_transform_point(&point1) + best_overlap * normal2.into_inner();
        let separation_distance = -best_overlap;
        let toi = 0.0;
        let contact = Contact {
            point1,
            point2,
            normal1,
            normal2,
            separation_distance,
            toi,
        };

        Some(contact)
    } else {
        let mut pt_on_edge1: Point3 = cube1.half_extents.into();
        let mut pt_on_edge2: Point3 = cube2.half_extents.into();
        let axis_index1 = (best_index - 6) / 3;
        let axis_index2 = (best_index - 6) % 3;
        let axis1 = axes1.column(axis_index1).into();
        let axis2 = axes2.column(axis_index2).into();
        for i in 0..3 {
            if i == axis_index1 {
                pt_on_edge1[i] = 0.0;
            } else if axes1.column(i).dot(&axis) < 0.0 {
                pt_on_edge1[i] = -pt_on_edge1[i];
            }

            if i == axis_index2 {
                pt_on_edge2[i] = 0.0;
            } else if axes2.column(i).dot(&axis) > 0.0 {
                pt_on_edge2[i] = -pt_on_edge2[i];
            }
        }

        pt_on_edge2 = pos12.transform_point(&pt_on_edge2);
        let (point1, mut point2) = get_contact_point(&axis1, &axis2, &pt_on_edge1, &pt_on_edge2);
        point2 = pos12.inverse_transform_point(&point2);
        let normal1 = UnitVector3::new_normalize(axis);
        let normal2 = pos12.inverse_transform_unit_vector(&(-normal1));
        let separation_distance = -best_overlap;
        let toi = 0.0;

        let contact = Contact {
            point1,
            point2,
            normal1,
            normal2,
            separation_distance,
            toi,
        };

        Some(contact)
    };
    contact
}
