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
    @location(9) normal_x: vec3<f32>,
    @location(10) normal_y: vec3<f32>,
    @location(11) normal_z: vec3<f32>,
};

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_normal: vec3<f32>,
    @location(2) view_pos: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput{
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(instance.model_x, instance.model_y, instance.model_z, instance.model_w);
    let normal_matrix = mat3x3<f32>(instance.normal_x, instance.normal_y, instance.normal_z);
    let view_position = camera.view * model_matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    out.view_pos = view_position.xyz;
    out.clip_position = camera.proj * view_position;
    out.view_normal = normalize(camera.view * vec4<f32>(normal_matrix * model.normal, 0.0)).xyz;

    return out;
}

struct MaterialUniforms{
    diffuse: vec4<f32>,
    specular: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> material_uniforms: MaterialUniforms;
@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var t_specular: texture_2d<f32>;
@group(0) @binding(3)
var s_diffuse: sampler;


struct DirectionalLight{
    color: vec3<f32>,
    direction: vec3<f32>,
};
struct PointLight{
    color: vec3<f32>,
    pos: vec3<f32>,
    radius: f32,
};
struct PointLights{
    num_point_lights: u32,
    lights: array<PointLight>,
};

@group(2) @binding(0)
var<uniform> ambient_light: vec4<f32>;
@group(2) @binding(1)
var<uniform> directional_light: DirectionalLight;
@group(2) @binding(2)
var<storage> point_lights: PointLights;

fn calculate_ambient_reflection(diffuse_color: vec3<f32>) -> vec3<f32>{
    return ambient_light.rgb * diffuse_color;
}

fn calculate_diffuse_reflection(normal: vec3<f32>, light_dir: vec3<f32>, light_color: vec3<f32>, diffuse_color: vec3<f32>) -> vec3<f32>{
    let direct_color = light_color * max(0.0, dot(normal, light_dir));
    return direct_color * diffuse_color;
}

fn calculate_specular_reflection(nh: f32, nl: f32, light_color: vec3<f32>, specular_color: vec3<f32>, shininess: f32) -> vec3<f32>{
    let highlight = pow(nh, shininess) * f32(nl > 0.0);
    return light_color * specular_color * highlight;
}

fn calculate_directional_light_effect(pos: vec3<f32>, normal: vec3<f32>, diffuse_color: vec3<f32>, specular_color: vec3<f32>, shininess: f32) -> vec3<f32>{
    let v = normalize(-pos);
    let l = -normalize(camera.view * vec4<f32>(directional_light.direction, 0.0)).xyz;
    let h = normalize(v + l);
    let diffuse_reflection = calculate_diffuse_reflection(normal, l, directional_light.color, diffuse_color);
    let nh = max(0.0, dot(normal, h));
    let nl = dot(normal, l);
    let specular_reflection = calculate_specular_reflection(nh, nl, directional_light.color, specular_color, shininess);

    return diffuse_reflection + specular_reflection;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = in.view_pos;
    let diffuse_color = (textureSample(t_diffuse, s_diffuse, in.tex_coords) * material_uniforms.diffuse).rgb;
    let specular_color = (textureSample(t_specular, s_diffuse, in.tex_coords) * material_uniforms.specular);
    let ambient_reflection = calculate_ambient_reflection(diffuse_color);
    let directional_light_effect = calculate_directional_light_effect(pos, normalize(in.view_normal), diffuse_color, specular_color.rgb, specular_color.a);
    return vec4<f32>(ambient_reflection + directional_light_effect, 1.0);
}
