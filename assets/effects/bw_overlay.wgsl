// Flatfekt postprocess effect: black/white transparent overlay.
//
// Uses the standard Flatfekt postprocess material bind group (group 2).

struct Params {
  intensity: f32,
};

@group(2) @binding(0) var<uniform> params: Params;
@group(2) @binding(1) var src_tex: texture_2d<f32>;
@group(2) @binding(2) var src_samp: sampler;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vertex(@location(0) pos: vec3<f32>, @location(1) uv: vec2<f32>) -> VsOut {
  var out: VsOut;
  out.pos = vec4<f32>(pos.x, pos.y, 0.0, 1.0);
  out.uv = uv;
  return out;
}

fn hash21(p: vec2<f32>) -> f32 {
  let h = dot(p, vec2<f32>(127.1, 311.7));
  return fract(sin(h) * 43758.5453);
}

@fragment
fn fragment(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(src_tex, src_samp, in.uv);
  let t = clamp(params.intensity, 0.0, 1.0);

  // Static film-grain-ish overlay, B/W only.
  let cell = floor(in.uv * 240.0);
  let n = hash21(cell);
  let bw = select(0.0, 1.0, n > 0.5);
  let overlay = vec3<f32>(bw, bw, bw);

  // Blend overlay on top with alpha controlled by intensity.
  let out_rgb = mix(c.rgb, overlay, 0.25 * t);
  return vec4<f32>(out_rgb, c.a);
}

