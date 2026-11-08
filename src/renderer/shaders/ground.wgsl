struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct LightUniform {
    view_proj: mat4x4<f32>,
    direction: vec4<f32>,
}
@group(1) @binding(0)
var<uniform> light: LightUniform;
@group(1) @binding(1)
var shadow_map: texture_depth_2d;
@group(1) @binding(2)
var shadow_sampler: sampler_comparison;

const SHADOW_BIAS = 0.0005;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) shadow_position: vec4<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) id: u32,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 6>(
            vec2<f32>(-60.0, -60.0),
            vec2<f32>(60.0, -60.0),
            vec2<f32>(60.0, 60.0),
            vec2<f32>(-60.0, -60.0),
            vec2<f32>(60.0, 60.0),
            vec2<f32>(-60.0, 60.0),
        );

    let world = vec4<f32>(positions[index], -0.01, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world;
    out.shadow_position = light.view_proj * world;

    return out;
}

fn calculate_shadow(position: vec4<f32>) -> f32 {
    let projected = position.xyz / position.w;

    let uv = vec2<f32>(projected.x * 0.5 + 0.5, 0.5 - projected.y * 0.5);

    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || projected.z < 0.0
        || projected.z > 1.0 {
        return 1.0;
    }

    let texel = 1.0 / vec2<f32>(textureDimensions(shadow_map));
    let depth = projected.z - SHADOW_BIAS;

    var sum = 0.0;
    for(var x = -1; x <= 1; x++) {
        for(var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            sum += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, depth);
        }
    }

    return sum / 9.0;
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    let visibility = calculate_shadow(input.shadow_position);

    var out: FragmentOutput;

    let lighting = 0.35 + visibility * 0.65;
    out.color = vec4<f32>(vec3<f32>(0.16) * lighting, 1.0);
    out.id = 0u;

    return out;
}
