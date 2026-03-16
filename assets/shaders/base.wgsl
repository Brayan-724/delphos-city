
struct CameraUniform {
    view: mat4x4<f32>
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) color: vec4<f32>,
  @location(2) uv: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) world: vec3<f32>,
}

@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;
  out.color = model.color;
  out.uv = model.uv;
  out.position = camera.view * vec4<f32>(model.position, 1.0);
  out.world = model.position;
  return out;
}

@group(0)
@binding(0)
var texture: texture_2d<f32>;

@group(0)
@binding(1)
var sam: sampler;

fn rand2d(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let tex_color = in.color * textureSample(texture, sam, in.uv);
  
  if tex_color.a == 0 {
    return tex_color;
  }

  let light_source = vec2<f32>(1, -1) - in.world.xy;

  let normal = (in.uv  - 0.5) * 0.2;
  let light_dot = min(1, dot(light_source, normal) + 0.05);
  let light_distance2 = 1 / (pow(length(light_source), 2));
  let light_distance_flare = 1 / (pow(length(light_source), 4));

  let light = smoothstep(0., rand2d(floor(in.uv * 60.) + in.position.xy), light_dot) * light_distance_flare;
  // return vec4<f32>(vec3(light), 1.);
  var light_color = mix(vec4<f32>(1.), tex_color, 0.2);
  light_color.a = tex_color.a;

  var dark_color = mix(vec4<f32>(0.), tex_color, light_distance2);
  dark_color.a = tex_color.a;

  return mix(dark_color, light_color, light);
}
