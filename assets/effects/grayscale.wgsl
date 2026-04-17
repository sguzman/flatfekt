// Flatfekt demo postprocess effect: grayscale.
//
// This file is intentionally tiny and stable. It is referenced from scene TOML.

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
  let g = dot(c.rgb, vec3<f32>(0.299, 0.587, 0.114));
  let out_rgb = mix(c.rgb, vec3<f32>(g, g, g), clamp(params.intensity, 0.0, 1.0));
  return vec4<f32>(out_rgb, c.a);
}

