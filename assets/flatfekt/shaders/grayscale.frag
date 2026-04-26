#version 450
layout(location = 0) in vec2 v_Uv;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform PostProcessParams {
    float intensity;
} params;

layout(set = 2, binding = 1) uniform texture2D source;
layout(set = 2, binding = 2) uniform sampler source_sampler;

void main() {
    vec4 color = texture(sampler2D(source, source_sampler), v_Uv);
    float gray = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    vec3 gray_color = vec3(gray);
    o_Target = vec4(mix(color.rgb, gray_color, params.intensity), color.a);
}
