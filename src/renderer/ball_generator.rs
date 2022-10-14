use crate::math::*;
use crate::renderer::ModelVertex;
pub fn generate_ball() -> (Vec<ModelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    for i in 0..12 {
        let phi = 0.0 + i as f32 * PI / 11.0;
        let y = phi.cos();
        for j in 0..16 {
            let theta = 0.0 + j as f32 * 2.0 * PI / 15.0;
            let x = phi.sin() * theta.cos();
            let z = phi.sin() * theta.sin();
            let tex_coords = [j as f32 / 15.0, i as f32 / 10.0];
            let pos = [x, y, z];
            let normal = pos;
            let vertex = ModelVertex {
                pos,
                tex_coords,
                normal,
            };
            vertices.push(vertex);
        }
    }
    let mut indices = Vec::new();
    for i in 0..11 {
        for j in 0..15 {
            let a = 16 * i + j;
            let b = a + 16;
            let c = b + 1;
            let d = a + 1;
            indices.push(a);
            indices.push(c);
            indices.push(b);
            indices.push(d);
            indices.push(c);
            indices.push(a);
        }
    }
    (vertices, indices)
}
