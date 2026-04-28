// Flatfekt postprocess effect: sepia tone.
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

@fragment
fn fragment(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(src_tex, src_samp, in.uv);
  let t = clamp(params.intensity, 0.0, 1.0);

  let sepia = vec3<f32>(
    dot(c.rgb, vec3<f32>(0.393, 0.769, 0.189)),
    dot(c.rgb, vec3<f32>(0.349, 0.686, 0.168)),
    dot(c.rgb, vec3<f32>(0.272, 0.534, 0.131)),
  );

  let out_rgb = mix(c.rgb, sepia, t);
  return vec4<f32>(out_rgb, c.a);
}

