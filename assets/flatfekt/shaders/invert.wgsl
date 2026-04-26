struct PostProcessParams {
    intensity: f32,
};

@group(2) @binding(0) var<uniform> params: PostProcessParams;
@group(2) @binding(1) var source: texture_2d<f32>;
@group(2) @binding(2) var source_sampler: sampler;

@fragment
fn fragment(
    @location(0) v_Uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let color = textureSample(source, source_sampler, v_Uv);
    let inverted = vec4<f32>(1.0 - color.rgb, color.a);
    return mix(color, inverted, params.intensity);
}
