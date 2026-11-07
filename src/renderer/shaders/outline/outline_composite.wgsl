@group(0) @binding(0)
var selection_mask: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;

    let position = array<vec2<f32>, 3>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>(3.0, -1.0),
            vec2<f32>(-1.0, 3.0),
        );

    out.position = vec4<f32>(position[index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let dimensions = textureDimensions(selection_mask);
    let pixel = vec2<i32>(position.xy);
    let max_pixel = vec2<i32>(dimensions) - vec2<i32>(1);

    let center = textureLoad(selection_mask, clamp(pixel, vec2<i32>(0), max_pixel), 0).r;

    if center > 0.5 {
        return vec4<f32>(0.0);
    }

    var edge = false;
    let thickness = 3;
    for(var x = -thickness; x <= thickness; x++) {
        for(var y = -thickness; y <= thickness; y++) {
            let offset = vec2<i32>(x, y);

            if dot(offset, offset) > thickness * thickness {
                continue;
            }

            let sample_position = pixel + offset;

            if any(sample_position < vec2<i32>(0))
                || any(sample_position >= vec2<i32>(dimensions)) {
                continue;
            }

            let value = textureLoad(selection_mask, sample_position, 0).r;

            if value > 0.5 {
                edge = true;
            }
        }
    }

    if edge {
        return vec4<f32>(1.0, 0.45, 0.05, 1.0);
    }

    return vec4<f32>(0.0);
}
