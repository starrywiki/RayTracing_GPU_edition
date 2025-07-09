//shaders.wgsl  GPUedition-version
// const OBJECT_COUNT: u32 = 4;
// const FLT_MAX: f32 = 3.40282346638528859812e+38;
// const MAX_PATH_LENGTH: u32 = 13u;
// const EPSILON: f32 = 1e-3;
// const TWO_PI: f32 = 6.2831853;


// alias Materials = array<Material, OBJECT_COUNT>;
// var<private> materials: Materials = Materials(
//   Material(/*color*/ vec3(0.7, 0.5, 0.5), /*specular_or_ior*/1.),
//   Material(/*color*/ vec3(0.5, 0.5, 0.9), /*specular_or_ior*/0.),
//   Material(/*color*/ vec3(0.7, 0.9, 0.2), /*specular_or_ior*/0.),
//   Material(/*color*/ vec3(1.), /*specular_or_ior*/-1.0/1.5),
// );


// alias Scene = array<Sphere, OBJECT_COUNT>;
// var<private> scene: Scene = Scene(
//   Sphere(/*center*/ vec3(-1.1, 0.5, 0.), /*radius*/ 0.5, /*material_index*/ 0),
//   Sphere(/*center*/ vec3(0., 0.5, 0.),   /*radius*/ 0.5, /*material_index*/ 1),
//   Sphere(/*center*/ vec3(1.1, 0.5, 0.),  /*radius*/ 0.5, /*material_index*/ 3),

//   // Ground
//   Sphere(/*center*/ vec3(0., -2e2 - EPSILON, 0.), /*radius*/ 2e2, /*material_index*/ 2),
// );

// alias TriangleVertices = array<vec2f, 6>;
// var<private> vertices: TriangleVertices = TriangleVertices(
//     vec2f(-1.0, 1.0),
//     vec2f(-1.0, -1.0),
//     vec2f(1.0, 1.0),
//     vec2f(1.0, 1.0),
//     vec2f(-1.0, -1.0),
//     vec2f(1.0, -1.0),
// );

// @vertex fn display_vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f {
//     return vec4f(vertices[vid], 0.0, 1.0);
// }

// struct Uniforms {
//     camera: CameraUniforms,
//     width: u32,
//     height: u32,
//     frame_count: u32,
// }

// @group(0) @binding(0) var<uniform> uniforms: Uniforms;
// @group(0) @binding(1) var radiance_samples_old: texture_2d<f32>;
// @group(0) @binding(2) var radiance_samples_new: texture_storage_2d<rgba32float, write>;

// struct CameraUniforms {
//   origin: vec3f,
//   u: vec3f,
//   v: vec3f,
//   w: vec3f,
// }

// struct Scatter {
//   attenuation: vec3f,
//   ray: Ray,
// }

// fn sample_lambertian(normal: vec3f) -> vec3f {
//   return normal + sample_sphere() * (1. - EPSILON);
// }

// fn scatter(input_ray: Ray, hit: Intersection, material: Material) -> Scatter {
//   let incident = normalize(input_ray.direction);
//   let incident_dot_normal = dot(incident, hit.normal);
//   let is_front_face = incident_dot_normal < 0.;
//   let N = select(-hit.normal, hit.normal, is_front_face);
//   let cos_theta = abs(incident_dot_normal);

//   // `ior`, `ref_ratio`, and `cannot_refract` only have meaning if the material is transmissive.
//   let is_transmissive = material.specular_or_ior < 0.;
//   let is_specular = material.specular_or_ior > 0.;
//   let ior = abs(material.specular_or_ior);
//   let ref_ratio = select(ior, 1. / ior, is_front_face);
//   let cannot_refract = ref_ratio * ref_ratio * (1.0 - cos_theta * cos_theta) > 1.;

//   var scattered: vec3f;
//   if is_specular || (is_transmissive && cannot_refract) {
//     scattered = reflect(incident, N);
//   } else if is_transmissive {
//     scattered = refract(incident, N, ref_ratio);
//   } else {
//     scattered = sample_lambertian(N);
//   }
//   let output_ray = Ray(point_on_ray(input_ray, hit.t), scattered);
//   let attenuation = material.color;
//   return Scatter(attenuation, output_ray);
// }

// struct Ray {
//   origin: vec3f,
//   direction: vec3f,
// }

// fn point_on_ray(ray: Ray, t: f32) -> vec3<f32> {
//   return ray.origin + t * ray.direction;
// }

// struct Rng {
//   state: u32,
// };
// var<private> rng: Rng;

// fn init_rng(pixel: vec2u) {
//   // Seed the PRNG using the scalar index of the pixel and the current frame count.
//   let seed = (pixel.x + pixel.y * uniforms.width) ^ jenkins_hash(uniforms.frame_count);
//   rng.state = jenkins_hash(seed);
// }

// // A slightly modified version of the "One-at-a-Time Hash" function by Bob Jenkins.
// // See https://www.burtleburtle.net/bob/hash/doobs.html
// fn jenkins_hash(i: u32) -> u32 {
//   var x = i;
//   x += x << 10u;
//   x ^= x >> 6u;
//   x += x << 3u;
//   x ^= x >> 11u;
//   x += x << 15u;
//   return x;
// }

// // The 32-bit "xor" function from Marsaglia G., "Xorshift RNGs", Section 3.
// fn xorshift32() -> u32 {
//   var x = rng.state;
//   x ^= x << 13;
//   x ^= x >> 17;
//   x ^= x << 5;
//   rng.state = x;
//   return x;
// }

// // Returns a random float in the range [0...1]. This sets the floating point exponent to zero and
// // sets the most significant 23 bits of a random 32-bit unsigned integer as the mantissa. That
// // generates a number in the range [1, 1.9999999], which is then mapped to [0, 0.9999999] by
// // subtraction. See Ray Tracing Gems II, Section 14.3.4.
// fn rand_f32() -> f32 {
//   return bitcast<f32>(0x3f800000u | (xorshift32() >> 9u)) - 1.;
// }

// // Uniformly sample a unit sphere centered at the origin
// fn sample_sphere() -> vec3f {
//   let r0 = rand_f32();
//   let r1 = rand_f32();

//   // Map r0 to [-1, 1]
//   let y = 1. - 2. * r0;

//   // Compute the projected radius on the xz-plane using Pythagorean theorem
//   let xz_r = sqrt(1. - y * y);

//   let phi = TWO_PI * r1;
//   return vec3(xz_r * cos(phi), y, xz_r * sin(phi));
// }

// struct Material{
//   color: vec3f,
//   specular_or_ior: f32,
// }

// struct Intersection {
//   normal: vec3f,
//   t: f32,
//   material_index: u32,
// }

// fn no_intersection() -> Intersection {
//   return Intersection(vec3(0.), -1., 0);
// }

// fn is_intersection_valid(hit: Intersection) -> bool {
//   return hit.t > 0.;
// }

// struct Sphere {
//   center: vec3f,
//   radius: f32,
//   material_index: u32,
// }

// fn intersect_sphere(ray: Ray, sphere: Sphere) -> Intersection {
//   let v = ray.origin - sphere.center;
//   let a = dot(ray.direction, ray.direction);
//   let b = dot(v, ray.direction);
//   let c = dot(v, v) - sphere.radius * sphere.radius;

//   let d = b * b - a * c;
//   if d < 0.0 {
//     return no_intersection();
//   }

//   let sqrt_d = sqrt(d);
//   let recip_a = 1.0 / a;
//   let mb = -b;
//   let t1 = (mb - sqrt_d) * recip_a;
//   let t2 = (mb + sqrt_d) * recip_a;
//   let t = select(t2, t1, t1 > EPSILON);
//   if t <= EPSILON {
//     return no_intersection();
//   }

//   let p = point_on_ray(ray, t);
//   let N = (p - sphere.center) / sphere.radius;
//   return Intersection(N, t, sphere.material_index);
// }

// fn intersect_scene(ray: Ray) -> Intersection {
//   var closest_hit = no_intersection();
//   closest_hit.t = FLT_MAX;  
//   for (var i = 0u; i < OBJECT_COUNT; i += 1u) {
//     let sphere = scene[i];
//     let hit = intersect_sphere(ray, sphere);
//     if hit.t > 0. && hit.t < closest_hit.t {
//       closest_hit = hit;
//     }
//   }
//   if closest_hit.t < FLT_MAX {
//     return closest_hit;
//   }
//   return no_intersection();
// }

// fn sky_color(ray: Ray) -> vec3f {
//   let t = 0.5 * (normalize(ray.direction).y + 1.0);
//   return (1.0 - t) * vec3(1.0) + t * vec3(0.3, 0.5, 1.0);
// }

// @fragment fn display_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
//   init_rng(vec2u(pos.xy));

//   let origin = uniforms.camera.origin;
//   let focus_distance = 1.0;
//   let aspect_ratio = f32(uniforms.width) / f32(uniforms.height);
  
//   // Normalize the viewport coordinates.
//   // Offset and normalize the viewport coordinates of the ray.
//   let offset = vec2(rand_f32() - 0.5, rand_f32() - 0.5);
  
//   var uv = (pos.xy + offset) / vec2f(f32(uniforms.width - 1u), f32(uniforms.height - 1u));

//   // Map `uv` from y-down (normalized) viewport coordinates to camera coordinates.
//   uv = (2.0 * uv - vec2(1.0)) * vec2(aspect_ratio, -1.0);    
  
//   // Compute the scene-space ray direction by rotating the camera-space vector into a new
//   // basis.
//   let camera_rotation = mat3x3(uniforms.camera.u, uniforms.camera.v, uniforms.camera.w);
//   let direction = camera_rotation * vec3(uv, focus_distance);
//   var ray = Ray(origin, direction);
//   var throughput = vec3f(1.);
//   var radiance_sample = vec3(0.);

//   var path_length = 0u;
//   while path_length < MAX_PATH_LENGTH {
//     let hit = intersect_scene(ray);
//     if !is_intersection_valid(hit) {
//       // If no intersection was found, return the color of the sky and terminate the path.
//       radiance_sample += throughput * sky_color(ray);
//       break;
//     }

//     let material = materials[hit.material_index];
//     let scattered = scatter(ray, hit, material);
//     throughput *= scattered.attenuation;
//     ray = scattered.ray;
//     path_length += 1u;
//   }
//   // Fetch the old sum of samples.
//   var old_sum: vec3f;
//   if uniforms.frame_count > 1 {
//     old_sum = textureLoad(radiance_samples_old, vec2u(pos.xy), 0).xyz;
//   } else {
//     old_sum = vec3(0.);
//   }

//   // Compute and store the new sum.
//   let new_sum = radiance_sample + old_sum;
//   textureStore(radiance_samples_new, vec2u(pos.xy), vec4(new_sum, 0.));

//   // Display the average after gamma correction (gamma = 2.2)
//   let color = new_sum / f32(uniforms.frame_count);
//   return vec4(pow(color, vec3(1. / 2.2)), 1.);
// }

//shaders.wgsl week1 version
const FLT_MAX: f32 = 3.40282346638528859812e+38;
const MAX_PATH_LENGTH: u32 = 50u;
const EPSILON: f32 = 1e-3;
const TWO_PI: f32 = 6.2831853;

// 场景参数
const GRID_SIZE: i32 = 10;  // -5 to 5 (减少球数量)
const TOTAL_SMALL_SPHERES: u32 = 100u;  // 10 * 10
const MAIN_SPHERES: u32 = 4u;  // 3 large spheres + 1 ground
const TOTAL_SPHERES: u32 = TOTAL_SMALL_SPHERES + MAIN_SPHERES;

alias TriangleVertices = array<vec2f, 6>;
var<private> vertices: TriangleVertices = TriangleVertices(
    vec2f(-1.0, 1.0),
    vec2f(-1.0, -1.0),
    vec2f(1.0, 1.0),
    vec2f(1.0, 1.0),
    vec2f(-1.0, -1.0),
    vec2f(1.0, -1.0),
);

@vertex fn display_vs(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f {
    return vec4f(vertices[vid], 0.0, 1.0);
}

struct Uniforms {
    camera: CameraUniforms,
    width: u32,
    height: u32,
    frame_count: u32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var radiance_samples_old: texture_2d<f32>;
@group(0) @binding(2) var radiance_samples_new: texture_storage_2d<rgba32float, write>;

struct CameraUniforms {
  origin: vec3f,
  u: vec3f,
  v: vec3f,
  w: vec3f,
}

struct Scatter {
  attenuation: vec3f,
  ray: Ray,
}

fn sample_lambertian(normal: vec3f) -> vec3f {
  return normal + sample_sphere() * (1. - EPSILON);
}

// Schlick反射率近似 (对应Rust中的Dielectric::reflectance)
fn schlick_reflectance(cosine: f32, ref_idx: f32) -> f32 {
  let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
  let r0_squared = r0 * r0;
  let one_minus_cosine = 1.0 - cosine;
  return r0_squared + (1.0 - r0_squared) * pow(one_minus_cosine, 5.0);
}

fn scatter(input_ray: Ray, hit: Intersection, material: Material) -> Scatter {
  let incident = normalize(input_ray.direction);
  let incident_dot_normal = dot(incident, hit.normal);
  let is_front_face = incident_dot_normal < 0.;
  let N = select(-hit.normal, hit.normal, is_front_face);
  let cos_theta = abs(incident_dot_normal);

  let is_transmissive = material.specular_or_ior < 0.;
  let is_specular = material.specular_or_ior > 0.;
  let is_lambertian = material.specular_or_ior == 0.;
  
  var scattered: vec3f;
  var attenuation: vec3f;
  
  if is_transmissive {
    // Dielectric材质处理
    let ior = abs(material.specular_or_ior);  // 现在直接是折射率，比如1.5
    let ref_ratio = select(ior, 1. / ior, is_front_face);  // 从外部射入时用1/ior，从内部射出时用ior
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    let cannot_refract = ref_ratio * sin_theta > 1.0;
    
    // 使用Schlick反射率近似决定反射还是折射
    let reflectance = schlick_reflectance(cos_theta, ref_ratio);
    let should_reflect = cannot_refract || reflectance > rand_f32();
    
    if should_reflect {
      scattered = reflect(incident, N);
    } else {
      scattered = refract(incident, N, ref_ratio);
    }
    attenuation = vec3(1.0, 1.0, 1.0); // Dielectric无颜色衰减
  } else if is_specular {
    // 金属材质处理
    let reflected = reflect(incident, N);
    let fuzz = max(0.0, material.specular_or_ior - 1.0); // 确保fuzz非负
    scattered = normalize(reflected + fuzz * sample_sphere());
    attenuation = material.color;
    
    // 确保散射方向在表面上方
    if dot(scattered, N) <= 0.0 {
      // 如果散射方向在表面下方，则使用纯反射
      scattered = reflect(incident, N);
    }
  } else {
    // 漫反射材质处理
    scattered = sample_lambertian(N);
    attenuation = material.color;
  }
  
  let output_ray = Ray(point_on_ray(input_ray, hit.t), scattered);
  return Scatter(attenuation, output_ray);
}

struct Ray {
  origin: vec3f,
  direction: vec3f,
}

fn point_on_ray(ray: Ray, t: f32) -> vec3<f32> {
  return ray.origin + t * ray.direction;
}

struct Rng {
  state: u32,
};
var<private> rng: Rng;

fn init_rng(pixel: vec2u) {
  let seed = (pixel.x + pixel.y * uniforms.width) ^ jenkins_hash(uniforms.frame_count);
  rng.state = jenkins_hash(seed);
}

fn jenkins_hash(i: u32) -> u32 {
  var x = i;
  x += x << 10u;
  x ^= x >> 6u;
  x += x << 3u;
  x ^= x >> 11u;
  x += x << 15u;
  return x;
}

fn xorshift32() -> u32 {
  var x = rng.state;
  x ^= x << 13;
  x ^= x >> 17;
  x ^= x << 5;
  rng.state = x;
  return x;
}

fn rand_f32() -> f32 {
  return bitcast<f32>(0x3f800000u | (xorshift32() >> 9u)) - 1.;
}

// 确定性随机数生成器（用于场景生成）
fn deterministic_rand(seed: u32) -> f32 {
  let x = jenkins_hash(seed);
  return bitcast<f32>(0x3f800000u | (x >> 9u)) - 1.;
}

fn sample_sphere() -> vec3f {
  let r0 = rand_f32();
  let r1 = rand_f32();
  let y = 1. - 2. * r0;
  let xz_r = sqrt(1. - y * y);
  let phi = TWO_PI * r1;
  return vec3(xz_r * cos(phi), y, xz_r * sin(phi));
}

struct Material{
  color: vec3f,
  specular_or_ior: f32,
  /*
  材质编码系统：
  - specular_or_ior = 0.0: 漫反射材质(Lambertian)
  - specular_or_ior > 0.0: 金属材质(Metal)
    - 1.0: 无fuzz的金属
    - 1.0 + fuzz: 有fuzz的金属，fuzz范围[0, 0.5]
  - specular_or_ior < 0.0: 透明材质(Dielectric)
    - -ior: 直接存储负的折射率
    - 例如：玻璃(ior=1.5) -> -1.5
    
  这个编码系统允许我们在单个f32值中存储所有材质类型和参数，
  与您的Rust代码中的材质系统完全兼容。
  */
}

struct Intersection {
  normal: vec3f,
  t: f32,
  material: Material,
}

fn no_intersection() -> Intersection {
  return Intersection(vec3(0.), -1., Material(vec3(0.), 0.));
}

fn is_intersection_valid(hit: Intersection) -> bool {
  return hit.t > 0.;
}

struct Sphere {
  center: vec3f,
  radius: f32,
  material: Material,
}

// 根据索引生成球的属性
fn generate_sphere(index: u32) -> Sphere {
  if index == 0u {
    // 地面球 (大的漫反射球)
    return Sphere(
      vec3(0., -1000., 0.), 
      1000., 
      Material(vec3(0.5, 0.5, 0.5), 0.)  // 漫反射
    );
  } else if index == 1u {
    // 中央玻璃球 (Dielectric, ir=1.5)
    return Sphere(
      vec3(0., 1., 0.), 
      1.0, 
      Material(vec3(1., 1., 1.), -1.5)  // 直接存储负的折射率
    );
  } else if index == 2u {
    // 左侧棕色漫反射球
    return Sphere(
      vec3(-4., 1., 0.), 
      1.0, 
      Material(vec3(0.4, 0.2, 0.1), 0.)  // 漫反射
    );
  } else if index == 3u {
    // 右侧金属球 (fuzz = 0.0)
    return Sphere(
      vec3(4., 1., 0.), 
      1.0, 
      Material(vec3(0.7, 0.6, 0.5), 1.0)  // 无fuzz的金属
    );
  } else {
    // 小球
    let small_index = index - MAIN_SPHERES;
    let grid_x = i32(small_index % u32(GRID_SIZE)) - 5;  // 改为-5到5
    let grid_z = i32(small_index / u32(GRID_SIZE)) - 5;  // 改为-5到5
    
    // 使用确定性随机数
    let seed1 = jenkins_hash(small_index * 3u);
    let seed2 = jenkins_hash(small_index * 3u + 1u);
    let seed3 = jenkins_hash(small_index * 3u + 2u);
    
    let random1 = deterministic_rand(seed1);
    let random2 = deterministic_rand(seed2);
    let choose_mat = deterministic_rand(seed3);
    
    let center = vec3(
      f32(grid_x) + 0.9 * random1,
      0.2,
      f32(grid_z) + 0.9 * random2
    );
    
    // 检查是否与主球重叠
    if length(center - vec3(4., 0.2, 0.)) <= 0.9 {
      // 返回一个无效的球（半径为0）
      return Sphere(center, 0., Material(vec3(0.), 0.));
    }
    
    var material: Material;
    if choose_mat < 0.8 {
      // 漫反射材质
      let color_seed1 = jenkins_hash(small_index * 6u);
      let color_seed2 = jenkins_hash(small_index * 6u + 1u);
      let color_seed3 = jenkins_hash(small_index * 6u + 2u);
      let color_seed4 = jenkins_hash(small_index * 6u + 3u);
      let color_seed5 = jenkins_hash(small_index * 6u + 4u);
      let color_seed6 = jenkins_hash(small_index * 6u + 5u);
      
      let albedo = vec3(
        deterministic_rand(color_seed1) * deterministic_rand(color_seed2),
        deterministic_rand(color_seed3) * deterministic_rand(color_seed4),
        deterministic_rand(color_seed5) * deterministic_rand(color_seed6)
      );
      material = Material(albedo, 0.);
    } else if choose_mat < 0.95 {
      // 金属材质
      let metal_seed1 = jenkins_hash(small_index * 4u);
      let metal_seed2 = jenkins_hash(small_index * 4u + 1u);
      let metal_seed3 = jenkins_hash(small_index * 4u + 2u);
      let fuzz_seed = jenkins_hash(small_index * 4u + 3u);
      
      let albedo = vec3(
        0.5 + 0.5 * deterministic_rand(metal_seed1),
        0.5 + 0.5 * deterministic_rand(metal_seed2),
        0.5 + 0.5 * deterministic_rand(metal_seed3)
      );
      let fuzz = 0.5 * deterministic_rand(fuzz_seed);
      // 编码：1.0 + fuzz，其中fuzz范围是[0, 0.5]
      material = Material(albedo, 1.0 + fuzz);
    } else {
      // Dielectric材质（玻璃）
      // 直接存储负的折射率：-1.5
      material = Material(vec3(1., 1., 1.), -1.5);
    }
    
    return Sphere(center, 0.2, material);
  }
}

fn intersect_sphere(ray: Ray, sphere: Sphere) -> Intersection {
  if sphere.radius <= 0. {
    return no_intersection();
  }
  
  let v = ray.origin - sphere.center;
  let a = dot(ray.direction, ray.direction);
  let b = dot(v, ray.direction);
  let c = dot(v, v) - sphere.radius * sphere.radius;

  let d = b * b - a * c;
  if d < 0.0 {
    return no_intersection();
  }

  let sqrt_d = sqrt(d);
  let recip_a = 1.0 / a;
  let mb = -b;
  let t1 = (mb - sqrt_d) * recip_a;
  let t2 = (mb + sqrt_d) * recip_a;
  let t = select(t2, t1, t1 > EPSILON);
  if t <= EPSILON {
    return no_intersection();
  }

  let p = point_on_ray(ray, t);
  let N = (p - sphere.center) / sphere.radius;
  return Intersection(N, t, sphere.material);
}

fn intersect_scene(ray: Ray) -> Intersection {
  var closest_hit = no_intersection();
  closest_hit.t = FLT_MAX;
  
  for (var i = 0u; i < TOTAL_SPHERES; i += 1u) {
    let sphere = generate_sphere(i);
    let hit = intersect_sphere(ray, sphere);
    if hit.t > 0. && hit.t < closest_hit.t {
      closest_hit = hit;
    }
  }
  
  if closest_hit.t < FLT_MAX {
    return closest_hit;
  }
  return no_intersection();
}

fn sky_color(ray: Ray) -> vec3f {
  let t = 0.5 * (normalize(ray.direction).y + 1.0);
  return (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0);
}

@fragment fn display_fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
  init_rng(vec2u(pos.xy));

  let origin = uniforms.camera.origin;
  let focus_distance = 1.0;
  let aspect_ratio = f32(uniforms.width) / f32(uniforms.height);
  
  let offset = vec2(rand_f32() - 0.5, rand_f32() - 0.5);
  var uv = (pos.xy + offset) / vec2f(f32(uniforms.width - 1u), f32(uniforms.height - 1u));
  uv = (2.0 * uv - vec2(1.0)) * vec2(aspect_ratio, -1.0);    
  
  let camera_rotation = mat3x3(uniforms.camera.u, uniforms.camera.v, uniforms.camera.w);
  let direction = camera_rotation * vec3(uv, focus_distance);
  var ray = Ray(origin, direction);
  var throughput = vec3f(1.);
  var radiance_sample = vec3(0.);

  var path_length = 0u;
  while path_length < MAX_PATH_LENGTH {
    let hit = intersect_scene(ray);
    if !is_intersection_valid(hit) {
      radiance_sample += throughput * sky_color(ray);
      break;
    }

    let scattered = scatter(ray, hit, hit.material);
    throughput *= scattered.attenuation;
    ray = scattered.ray;
    path_length += 1u;
  }
  
  var old_sum: vec3f;
  if uniforms.frame_count > 1 {
    old_sum = textureLoad(radiance_samples_old, vec2u(pos.xy), 0).xyz;
  } else {
    old_sum = vec3(0.);
  }

  let new_sum = radiance_sample + old_sum;
  textureStore(radiance_samples_new, vec2u(pos.xy), vec4(new_sum, 0.));

  let color = new_sum / f32(uniforms.frame_count);
  return vec4(pow(color, vec3(1. / 2.2)), 1.);
}