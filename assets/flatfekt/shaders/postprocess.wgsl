// Built-in fallback postprocess shader for Flatfekt.
//
// This is intentionally minimal. Scene-controlled shaders are validated/loaded
// as assets, but this baseline fullscreen pass uses a stable built-in shader
// path to keep the runtime simple while still providing “shader hooks”.

struct Params {
  intensity: f32,
};

@group(2) @binding(0) var<uniform> params: Params;
@group(2) @binding(1) var src_tex: texture_2d<f32>;
@group(2) @binding(2) var src_samp: sampler;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) @interpolate(perspective) v_Uv: vec2<f32>,
};

@vertex
fn vertex(@location(0) pos: vec3<f32>, @location(1) uv: vec2<f32>) -> VsOut {
  var out: VsOut;
  out.pos = vec4<f32>(pos.x, pos.y, 0.0, 1.0);
  out.v_Uv = uv;
  return out;
}

@fragment
fn fragment(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(src_tex, src_samp, in.v_Uv);
  // Simple identity with optional fade-to-black.
  let t = clamp(params.intensity, 0.0, 1.0);
  return vec4<f32>(c.rgb * t, c.a);
}

