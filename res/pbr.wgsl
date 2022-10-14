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

  let denom = (NH2 * (a2 - 1.0) + 1.0);
  let denom2 = PI * denom * denom;

  return a2 / denom;
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
  let kD = vec3<f32>(1.0, 1.0, 1.0) - kS;

  let kD = kD * (1.0 - metalic);

  let numerator = NDF * G * F;
  let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
  let specular = numerator / denominator;

  let NL = max(dot(N, L), 0.0);
  
  return (kD * albedo / PI + specular) * light_radiance * NL;
}
/// conpute fresnel
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32>{
  return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}
