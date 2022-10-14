use crate::math::*;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Isometry3,
    pub near: Float,
    pub far: Float,
    pub aspect: Float,
    pub fovy: Float,
}

impl Camera {

    #[rustfmt::skip]
    const OPENGL_TO_WGPU_MATRIX: Matrix4 = 
        Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

    pub fn new() -> Self {
        let position = Isometry3::identity();
        let near = 0.1;
        let far = 100.0;
        let aspect = 1920.0 / 1080.0;
        let fovy = 90.0;

        Self {
            position,
            near,
            far,
            aspect,
            fovy,
        }
    }

    pub fn build_veiw_matrix(&self) -> Matrix4 {
        self.position.inverse().to_homogeneous()
    }

    pub fn build_camera_matrix(&self) -> Matrix4{
        self.position.to_homogeneous()
    }

    pub fn build_projection_matrix(&self) -> Matrix4 {
        perspective_projection(self.fovy, self.aspect, self.near, self.far)
    }
    pub fn build_rev_projection_matrix(&self) -> Matrix4 {
        perspective_projection(self.fovy, self.aspect, self.far, self.near)
    }

    pub fn top_left_at_depth(&self, d: Float) -> Point3{
        let z = d;
        let y = z * (self.fovy * 0.5).tan();
        let x = y * self.aspect;
        Point3::new(x, y, z)
    }

    pub fn get_vertex_local(&self, index: usize) -> Point3{
        assert!(index < 8);
        // near plane
        let mut y = self.near * (self.fovy * 0.5).tan();
        let mut x = y * self.aspect;
        let mut z = self.near;
        if (index & 2) == 2 {
            x = -x;
        }
        if index % 4 == 0 || index % 4 == 3{
            y = -y;
        }

        let mut v = Vector3::new(x, y, z);
        if index >= 4{
            v = self.far / self.near * v;
        }
        v.into()
    }
}

#[rustfmt::skip]
fn perspective_projection(fovy: Float, aspect: Float, near: Float, far: Float) -> Matrix4{
    let g = 1.0 / (fovy * 0.5).tan();
    let k = far / (far - near);
    return Matrix4::new(
        g/aspect, 0.0, 0.0, 0.0,
        0.0, g, 0.0, 0.0,
        0.0, 0.0, k, -near * k,
        0.0, 0.0, 1.0, 0.0
        )
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn camera_vertex_test(){
        let camera = Camera{
            position: Isometry3::identity(),
            near: 1.0,
            far: 1000.0,
            aspect: 2.0,
            fovy: PI / 2.0,
        };
        assert_eq!(camera.get_vertex_local(0), Point3::new(2.0, -1.0, 1.0));
        assert_eq!(camera.get_vertex_local(1), Point3::new(2.0, 1.0, 1.0));
        assert_eq!(camera.get_vertex_local(2), Point3::new(-2.0, 1.0, 1.0));
        assert_eq!(camera.get_vertex_local(3), Point3::new(-2.0, -1.0, 1.0));

        assert_eq!(camera.get_vertex_local(4), Point3::new(2000.0, -1000.0, 1000.0));
        assert_eq!(camera.get_vertex_local(5), Point3::new(2000.0, 1000.0, 1000.0));
        assert_eq!(camera.get_vertex_local(6), Point3::new(-2000.0, 1000.0, 1000.0));
        assert_eq!(camera.get_vertex_local(7), Point3::new(-2000.0, -1000.0, 1000.0));

        assert_eq!(camera.top_left_at_depth(1.0), Point3::new(2.0, 1.0, 1.0));
        assert_eq!(camera.top_left_at_depth(500.0), Point3::new(1000.0, 500.0, 500.0));
        assert_eq!(camera.top_left_at_depth(1000.0), Point3::new(2000.0, 1000.0, 1000.0));
    }
}
