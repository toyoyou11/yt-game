use crate::math::*;
use crate::renderer::*;

#[derive(Debug)]
pub struct CascadeShadowPass {
    shadow_data: CascadedShadow,
}

#[derive(Debug, Clone, Copy)]
pub struct CascadedShadow {
    resolution: Float,
    a: [Float; 4],
    b: [Float; 4],
    cascade_size: [Float; 4],
    shadow_matrix: Matrix4,
    cascade_scale: [Vector3; 3],
    cascade_offset: [Vector3; 3],
}

impl CascadedShadow {
    const OVERLAP: Float = 0.1;
    pub fn new(camera: &Camera, light_direction: &Vector3, resolution: Float) -> Self {
        let b = [
            camera.far / 16.0,
            camera.far / 4.0,
            camera.far / 2.0,
            camera.far,
        ];
        let a = [
            0.0,
            b[0] + Self::OVERLAP * b[0],
            b[1] + Self::OVERLAP * b[1],
            b[2] + Self::OVERLAP * b[2],
        ];

        let mut cascade_size = [0.0; 4];
        let mut min = [Vector3::zeros(); 4];
        let mut max = [Vector3::zeros(); 4];
        let mut cascade_positions = vec![Point3::origin(); 4];
        for i in 0..4 {
            cascade_size[i] = calculate_cascade_size(camera, a[i], b[i]);
            (min[i], max[i]) = calculate_min_max(camera, light_direction, a[i], b[i]);

            let texel_size = cascade_size[i] / resolution;
            let half_inv_texel_size = 0.5 * texel_size;
            cascade_positions[i] = Point3::new(
                ((min[i].x + max[i].x) * half_inv_texel_size).floor() * texel_size,
                ((min[i].y + max[i].y) * half_inv_texel_size).floor() * texel_size,
                min[i].z,
            );
        }

        let inv_light_mat =
            UnitQuaternion::face_towards(light_direction, &Vector3::new(1.0, 0.0, 0.0))
                .inverse()
                .to_homogeneous();
        let mut inv_cascade_mat = inv_light_mat;
        inv_cascade_mat[(0, 3)] = -cascade_positions[0].x;
        inv_cascade_mat[(1, 3)] = -cascade_positions[0].y;
        inv_cascade_mat[(2, 3)] = -cascade_positions[0].z;
        inv_cascade_mat[(3, 3)] = 1.0;
        let shadow_projection = Matrix4::new(
            1.0 / cascade_size[0],
            0.0,
            0.0,
            0.5,
            0.0,
            1.0 / cascade_size[0],
            0.0,
            0.5,
            0.0,
            0.0,
            1.0 / (max[0].z - min[0].z),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        let shadow_matrix = shadow_projection * inv_cascade_mat;
        let mut cascade_scale = [Vector3::zeros(); 3];
        let mut cascade_offset = [Vector3::zeros(); 3];

        for i in 0..3 {
            let k = i + 1;
            cascade_scale[i] = Vector3::new(
                cascade_size[0] / cascade_size[k],
                cascade_size[0] / cascade_size[k],
                (max[0].z - min[0].z) / (max[k].z - min[k].z),
            );

            let pos_diff = cascade_positions[0] - cascade_positions[k];
            let offset = (-cascade_size[0] / cascade_size[k] + 1.0) * 0.5;
            cascade_offset[i] = Vector3::new(
                pos_diff.x / cascade_size[k] + offset,
                pos_diff.y / cascade_size[k] + offset,
                pos_diff.z / (max[k].z - min[k].z),
            );
        }

        Self {
            resolution,
            a,
            b,
            cascade_size,
            shadow_matrix,
            cascade_scale,
            cascade_offset,
        }
    }
}

fn calculate_cascade_size(camera: &Camera, zmin: Float, zmax: Float) -> Float {
    let v0 = camera.top_left_at_depth(zmin);
    let v1 = camera.top_left_at_depth(zmax);
    let v2 = Point3::new(-v1.x, -v1.y, v1.z);

    (v0 - v2).magnitude().max((v1 - v2).magnitude()).ceil()
}

fn calculate_min_max(
    camera: &Camera,
    light_direction: &Vector3,
    zmin: Float,
    zmax: Float,
) -> (Vector3, Vector3) {
    let near = camera.top_left_at_depth(zmin);
    let far = camera.top_left_at_depth(zmax);
    let vertices = [
        Point3::new(near.x, -near.y, near.z),
        near,
        Point3::new(-near.x, near.y, near.z),
        Point3::new(-near.x, -near.y, near.z),
        Point3::new(far.x, -far.y, far.z),
        far,
        Point3::new(-far.x, far.y, far.z),
        Point3::new(-far.x, -far.y, far.z),
    ];

    let mut min = Vector3::new(FLOAT_MAX, FLOAT_MAX, FLOAT_MAX);
    let mut max = Vector3::new(FLOAT_MIN, FLOAT_MIN, FLOAT_MIN);
    let camera_mat = camera.build_camera_matrix();
    let inv_light_mat = UnitQuaternion::face_towards(light_direction, &Vector3::new(1.0, 0.0, 0.0))
        .inverse()
        .to_homogeneous();
    let camera_to_light = inv_light_mat * camera_mat;
    for v in &vertices {
        let transformed = &camera_to_light * &v.to_homogeneous();
        min.x = min.x.min(transformed.x);
        max.x = max.x.max(transformed.x);
        min.y = min.y.min(transformed.y);
        max.y = max.y.max(transformed.y);
        min.z = min.z.min(transformed.z);
        max.z = max.z.max(transformed.z);
    }

    (min, max)
}

pub fn calculate_cascade_position(
    camera: &Camera,
    light_direction: &Vector3,
    cascade_size: Float,
    resolution: u32,
    zmin: Float,
    zmax: Float,
) -> Point3 {
    let near = camera.top_left_at_depth(zmin);
    let far = camera.top_left_at_depth(zmax);
    let vertices = [
        Point3::new(near.x, -near.y, near.z),
        near,
        Point3::new(-near.x, near.y, near.z),
        Point3::new(-near.x, -near.y, near.z),
        Point3::new(far.x, -far.y, far.z),
        far,
        Point3::new(-far.x, far.y, far.z),
        Point3::new(-far.x, -far.y, far.z),
    ];

    let mut xmin = FLOAT_MAX;
    let mut xmax = FLOAT_MIN;
    let mut ymin = FLOAT_MAX;
    let mut ymax = FLOAT_MIN;
    let mut zmin = FLOAT_MAX;
    let mut zmax = FLOAT_MIN;

    let camera_mat = camera.build_camera_matrix();
    let inv_light_mat = UnitQuaternion::face_towards(light_direction, &Vector3::new(1.0, 0.0, 0.0))
        .inverse()
        .to_homogeneous();
    let camera_to_light = inv_light_mat * camera_mat;
    for v in &vertices {
        let transformed = &camera_to_light * &v.to_homogeneous();
        xmin = xmin.min(transformed.x);
        xmax = xmax.max(transformed.x);

        ymin = xmin.min(transformed.y);
        ymax = xmax.max(transformed.y);

        zmin = xmin.min(transformed.z);
        zmax = xmax.max(transformed.z);
    }
    let texel_size = cascade_size / resolution as Float;
    let half_inv_texel_size = resolution as Float / cascade_size;
    let x = ((xmin + xmax) * half_inv_texel_size).floor() * texel_size;
    let y = ((ymin + ymax) * half_inv_texel_size).floor() * texel_size;
    let z = zmin;

    Point3::new(x, y, z)
}
