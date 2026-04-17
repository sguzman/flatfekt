// Built-in fallback postprocess shader for Flatfekt.
//
// This is intentionally minimal. Scene-controlled shaders are validated/loaded
// as assets, but this baseline fullscreen pass uses a stable built-in shader
// path to keep the runtime simple while still providing “shader hooks”.

@group(1) @binding(0) var src_tex: texture_2d<f32>;
@group(1) @binding(1) var src_samp: sampler;

struct Params {
  intensity: f32,
};

@group(1) @binding(2) var<uniform> params: Params;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec2<f32>, @location(1) uv: vec2<f32>) -> VsOut {
  var out: VsOut;
  out.pos = vec4<f32>(pos, 0.0, 1.0);
  out.uv = uv;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(src_tex, src_samp, in.uv);
  // Simple identity with optional fade-to-black.
  let t = clamp(params.intensity, 0.0, 1.0);
  return vec4<f32>(c.rgb * t, c.a);
}

