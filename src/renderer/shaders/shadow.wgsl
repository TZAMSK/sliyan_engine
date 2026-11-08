struct LightUniform {
    view_proj: mat4x4<f32>,
    direction: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    return light.view_proj * vec4<f32>(input.position, 1.0);
}
