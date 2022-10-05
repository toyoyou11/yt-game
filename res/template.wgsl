struct CameraUniform{
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct InstanceInput{
    @location(5) model_x: vec4<f32>,
    @location(6) model_y: vec4<f32>,
    @location(7) model_z: vec4<f32>,
    @location(8) model_w: vec4<f32>,
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput{
    var out: VertexOutput;
    let model_matrix = mat4x4<f32>(instance.model_x, instance.model_y, instance.model_z, instance.model_w);
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.proj * camera.view * model_matrix * vec4<f32>(model.position, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out: vec4<f32>;
    out = compute_color(in);
    return out;
}
