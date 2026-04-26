#import bevy_sprite_render::mesh2d_types
#import bevy_sprite_render::mesh2d_view_bindings

struct PostProcessParams {
    intensity: f32,
};

@group(2) @binding(0) var source: texture_2d<f32>;
@group(2) @binding(1) var source_sampler: sampler;
@group(2) @binding(2) var<uniform> params: PostProcessParams;

@fragment
fn fragment(
    @location(0) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let color = textureSample(source, source_sampler, uv);
    let inverted = vec4<f32>(1.0 - color.rgb, color.a);
    return mix(color, inverted, params.intensity);
}
