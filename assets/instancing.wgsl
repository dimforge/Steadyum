#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var<uniform> mesh: Mesh;

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec3<f32>,
    @location(4) i_rot0: vec3<f32>,
    @location(5) i_rot1: vec3<f32>,
    @location(6) i_rot2: vec3<f32>,
    @location(7) i_scale: vec3<f32>,
    @location(8) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let rotmat = mat3x3(vertex.i_rot0, vertex.i_rot1, vertex.i_rot2);
    let position = rotmat * (vertex.position * vertex.i_scale) + vertex.i_pos;
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(position, 1.0));
    out.color = vertex.i_color;
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let total = floor(in.uv.x * 3.0) + floor(in.uv.y * 3.0);
    let is_even = (total % 2.0) == 0.0;
    let checkerboard = select(in.color - vec4(0.15, 0.15, 0.15, 0.0), in.color, is_even);
    let brd_lo = 0.05;
    let brd_hi = 1.0 - brd_lo;
    let is_border = in.uv.x < brd_lo || in.uv.x > brd_hi || in.uv.y < brd_lo || in.uv.y > brd_hi;
    let black = vec4(0.0, 0.0, 0.0, 1.0);

    return select(checkerboard, black, is_border);
}
