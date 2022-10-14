let PI = 3.14159265;
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
    albedo: vec4<f32>,
    roughness: f32,
    metalic: f32,
    padding: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> material_uniforms: MaterialUniforms;
@group(0) @binding(1)
var t_albedo: texture_2d<f32>;
@group(0) @binding(2)
var s_albedo: sampler;


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

// pbr functions
fn geometry_schlick_GGX(NV: f32, roughness: f32) -> f32{
  let r = (roughness + 1.0);
  let k = (r * r) / 8.0;

  let denom = NV * (1.0 - k) + k;

  return NV / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32{
  let NV = max(dot(N, V), 0.0);
  let NL = max(dot(N, L), 0.0);

  let ggx2 = geometry_schlick_GGX(NV, roughness);
  let ggx1 = geometry_schlick_GGX(NL, roughness);

  return ggx1 * ggx2;
}
fn distribution_GGX(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32{
  let a = roughness * roughness;
  let a2 = a * a;
  let NH = max(dot(N, H), 0.0);
  let NH2 = NH * NH;

  var denom = (NH2 * (a2 - 1.0) + 1.0);
  denom = PI * denom * denom;

  return a2 / denom;
}
/// conpute fresnel
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32>{
  return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

/// Calculate reflectance based on pbr
fn reflectance(
  N: vec3<f32>,
  V: vec3<f32>,
  L: vec3<f32>,
  albedo: vec3<f32>,
  roughness: f32,
  metalic: f32,
  light_radiance: vec3<f32>
) -> vec3<f32>{
  let H = normalize(V + L);
  let NDF = distribution_GGX(N, H, roughness);
  let G = geometry_smith(N, V, L, roughness);
  let F0 = mix(vec3<f32>(0.04, 0.04, 0.04), albedo, metalic);
  let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

  let kS = F;
  var kD = vec3<f32>(1.0, 1.0, 1.0) - kS;

  kD = kD * (1.0 - metalic);

  let numerator = NDF * G * F;
  let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
  let specular = numerator / denominator;

  let NL = max(dot(N, L), 0.0);
  
  return (kD * albedo / PI + specular) * light_radiance * NL;
}
fn calculate_ambient_reflection(albedo: vec3<f32>) -> vec3<f32>{
    return ambient_light.rgb * albedo;
}

fn calculate_directional_light_effect(pos: vec3<f32>, normal: vec3<f32>, albedo: vec3<f32>, roughness: f32, metalic: f32) -> vec3<f32>{
    let v = normalize(-pos);
    let l = -normalize(camera.view * vec4<f32>(directional_light.direction, 0.0)).xyz;
    let h = normalize(v + l);
    return reflectance(normal, v, l, albedo, roughness, metalic, directional_light.color);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = in.view_pos;
    let albedo = (textureSample(t_albedo, s_albedo, in.tex_coords) * material_uniforms.albedo).rgb;
    let ambient_reflection = calculate_ambient_reflection(albedo);
    let directional_light_effect = calculate_directional_light_effect(pos, normalize(in.view_normal), albedo, material_uniforms.roughness, material_uniforms.metalic);
    var color = (ambient_reflection + directional_light_effect);
    color = color / (color + vec3<f32>(1.0, 1.0, 1.0));

    return vec4<f32>(color, 1.0);
}
