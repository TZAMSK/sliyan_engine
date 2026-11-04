struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct OutlineUniform {
    center: vec3<f32>,
    scale: f32,
}

@group(1) @binding(0)
var<uniform> outline: OutlineUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) id: u32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Scale the (already world-space) vertex away from the shape's center.
    let scaled = outline.center + (input.position - outline.center) * outline.scale;
    out.clip_position = camera.view_proj * vec4<f32>(scaled, 1.0);
    return out;
}

@fragment
fn fs_main() -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(1.0, 0.45, 0.05, 1.0);
    out.id = 0u;
    return out;
}
