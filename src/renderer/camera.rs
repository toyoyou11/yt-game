use nalgebra as na;

#[derive(Debug)]
pub struct Camera {
    pub position: na::Isometry3<f32>,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    pub fovy: f32,
}

impl Camera {

    #[rustfmt::skip]
    const OPENGL_TO_WGPU_MATRIX: na::Matrix4<f32> = 
        na::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

    pub fn new() -> Self {
        let position = na::Isometry3::identity();
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

    pub fn build_veiw_matrix(&self) -> na::Matrix4<f32> {
        self.position.inverse().into()
    }

    pub fn build_projection_matrix(&self) -> na::Matrix4<f32> {
        perspective_projection(self.fovy, self.aspect, self.near, self.far)
    }
    pub fn build_rev_projection_matrix(&self) -> na::Matrix4<f32> {
        perspective_projection(self.fovy, self.aspect, self.far, self.near)
    }
}

#[rustfmt::skip]
fn perspective_projection(fovy: f32, aspect: f32, near: f32, far: f32) -> na::Matrix4<f32>{
    let g = 1.0 / (fovy * 0.5).tan();
    let k = far / (far - near);
    return na::Matrix4::new(
        g/aspect, 0.0, 0.0, 0.0,
        0.0, g, 0.0, 0.0,
        0.0, 0.0, k, -near * k,
        0.0, 0.0, 1.0, 0.0
        )
}
