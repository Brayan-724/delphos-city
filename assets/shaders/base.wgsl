
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
}

@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;
  out.color = model.color;
  out.uv = model.uv;
  out.position = camera.view * vec4<f32>(model.position, 1.0);
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
  return in.color * textureSample(texture, sam, in.uv);

  // let t = min(1, length(vex.uv) * 0.7);
  // return smoothstep(1., rand2d(floor(vex.uv * 40.)), t) * textureSample(texture, sam, vex.uv);

  // return vec4<f32>(1.0, 0, 0, 1.0);
}
