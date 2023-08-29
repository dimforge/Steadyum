// NOTE: this is from https://raw.githubusercontent.com/setzer22/blackjack/9b42c3a41625dc7e5c493eea3ee829382078a1f3/src/rendergraph/grid_shader.wgsl
//       Original file was MIT licensed (https://github.com/setzer22/blackjack).


// Vertex shader
#import bevy_pbr::mesh_view_bind_group

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] near_point: vec3<f32>;
    [[location(1)]] far_point: vec3<f32>;
};

struct FragmentOutput {
    [[builtin(frag_depth)]] depth: f32;
    [[location(0)]] color: vec4<f32>;
};

struct GridRoutineUniform {
    view: mat4x4<f32>;
    proj: mat4x4<f32>;
    inv_view: mat4x4<f32>;
    inv_proj: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> matrices: GridRoutineUniform;

var<private> vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>( 
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
);


fn inverse(m: mat4x4<f32>) -> mat4x4<f32>
{
   let SubFactor00 = m[2][2] * m[3][3] - m[3][2] * m[2][3];
   let SubFactor01 = m[2][1] * m[3][3] - m[3][1] * m[2][3];
   let SubFactor02 = m[2][1] * m[3][2] - m[3][1] * m[2][2];
   let SubFactor03 = m[2][0] * m[3][3] - m[3][0] * m[2][3];
   let SubFactor04 = m[2][0] * m[3][2] - m[3][0] * m[2][2];
   let SubFactor05 = m[2][0] * m[3][1] - m[3][0] * m[2][1];
   let SubFactor06 = m[1][2] * m[3][3] - m[3][2] * m[1][3];
   let SubFactor07 = m[1][1] * m[3][3] - m[3][1] * m[1][3];
   let SubFactor08 = m[1][1] * m[3][2] - m[3][1] * m[1][2];
   let SubFactor09 = m[1][0] * m[3][3] - m[3][0] * m[1][3];
   let SubFactor10 = m[1][0] * m[3][2] - m[3][0] * m[1][2];
   let SubFactor11 = m[1][1] * m[3][3] - m[3][1] * m[1][3];
   let SubFactor12 = m[1][0] * m[3][1] - m[3][0] * m[1][1];
   let SubFactor13 = m[1][2] * m[2][3] - m[2][2] * m[1][3];
   let SubFactor14 = m[1][1] * m[2][3] - m[2][1] * m[1][3];
   let SubFactor15 = m[1][1] * m[2][2] - m[2][1] * m[1][2];
   let SubFactor16 = m[1][0] * m[2][3] - m[2][0] * m[1][3];
   let SubFactor17 = m[1][0] * m[2][2] - m[2][0] * m[1][2];
   let SubFactor18 = m[1][0] * m[2][1] - m[2][0] * m[1][1];
   let adj00 = (m[1][1] * SubFactor00 - m[1][2] * SubFactor01 + m[1][3] * SubFactor02);
   let adj10 = - (m[1][0] * SubFactor00 - m[1][2] * SubFactor03 + m[1][3] * SubFactor04);
   let adj20 = (m[1][0] * SubFactor01 - m[1][1] * SubFactor03 + m[1][3] * SubFactor05);
   let adj30 = - (m[1][0] * SubFactor02 - m[1][1] * SubFactor04 + m[1][2] * SubFactor05);
   let adj01 = - (m[0][1] * SubFactor00 - m[0][2] * SubFactor01 + m[0][3] * SubFactor02);
   let adj11 = (m[0][0] * SubFactor00 - m[0][2] * SubFactor03 + m[0][3] * SubFactor04);
   let adj21 = - (m[0][0] * SubFactor01 - m[0][1] * SubFactor03 + m[0][3] * SubFactor05);
   let adj31 = (m[0][0] * SubFactor02 - m[0][1] * SubFactor04 + m[0][2] * SubFactor05);
   let adj02 = (m[0][1] * SubFactor06 - m[0][2] * SubFactor07 + m[0][3] * SubFactor08);
   let adj12 = - (m[0][0] * SubFactor06 - m[0][2] * SubFactor09 + m[0][3] * SubFactor10);
   let adj22 = (m[0][0] * SubFactor11 - m[0][1] * SubFactor09 + m[0][3] * SubFactor12);
   let adj32 = - (m[0][0] * SubFactor08 - m[0][1] * SubFactor10 + m[0][2] * SubFactor12);
   let adj03 = - (m[0][1] * SubFactor13 - m[0][2] * SubFactor14 + m[0][3] * SubFactor15);
   let adj13 = (m[0][0] * SubFactor13 - m[0][2] * SubFactor16 + m[0][3] * SubFactor17);
   let adj23 = - (m[0][0] * SubFactor14 - m[0][1] * SubFactor16 + m[0][3] * SubFactor18);
   let adj33 = (m[0][0] * SubFactor15 - m[0][1] * SubFactor17 + m[0][2] * SubFactor18);
   
   let adj = mat4x4<f32>(
       adj00, adj01, adj02, adj03,
       adj10, adj11, adj12, adj13,
       adj20, adj21, adj22, adj23,
       adj30, adj31, adj32, adj33,
   );

   let det: f32 = (m[0][0] * adj[0][0]
		+ m[0][1] * adj[1][0]
		+ m[0][2] * adj[2][0]
		+ m[0][3] * adj[3][0]);
   return adj * (1.0 / det);
}


fn unproject_point(point: vec3<f32>, inv_view_proj: mat4x4<f32>) -> vec3<f32> {
    let unprojected_point = inv_view_proj * vec4<f32>(point, 1.0);
    return unprojected_point.xyz / unprojected_point.w;
}

[[stage(vertex)]]
fn vertex(
    [[builtin(vertex_index)]] in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let pos_xy = vertices[in_vertex_index];
    let pos = vec4<f32>(pos_xy.x, pos_xy.y, 0.0, 1.0);

    out.clip_position = pos;
    // TODO: Compute near_point / far_point
    let inv_view_proj = inverse(view.view_proj);
    out.near_point = unproject_point(vec3<f32>(pos.x, pos.y, 0.1), inv_view_proj).xyz; 
    out.far_point = unproject_point(vec3<f32>(pos.x, pos.y, 1.0), inv_view_proj).xyz; 

    return out;
}

// Fragment shader

fn grid(frag_pos_3d: vec3<f32>, scale: f32) -> vec4<f32> {
    let coord = frag_pos_3d.xz * scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let line = min(grid.x, grid.y);
    let minimumz = min(derivative.y, 1.0);
    let minimumx = min(derivative.x, 1.0);
    var color = vec4<f32>(0.2, 0.2, 0.2, 1.0 - min(line, 1.0));

    let threshold = 1.0 / scale;

    // z axis
    if (frag_pos_3d.x > -threshold * minimumx && frag_pos_3d.x < threshold * minimumx) {
        color.z = 1.0;
    }
    // x axis
    if (frag_pos_3d.z > -threshold * minimumz && frag_pos_3d.z < threshold * minimumz) {
        color.x = 1.0;
    }
    return color;
}

fn compute_depth(frag_pos_3d: vec3<f32>) -> f32 {
    let clip_space_pos = view.view_proj * vec4<f32>(frag_pos_3d, 1.0);
    return (clip_space_pos.z / clip_space_pos.w);
}

fn fading(frag_pos_3d: vec3<f32>, depth: f32) -> f32 {
    let znear = 0.01;
    // If you're using far plane at infinity as described here, then linearized depth is simply znear / depth.
    // From: https://www.reddit.com/r/GraphicsProgramming/comments/f9zwin/linearising_reverse_depth_buffer/
    let linear_depth = znear / depth;
    // NOTE: we take the sqrt to make the transition smoother at the horizon.
    return max(0.0, 5.5 - sqrt(linear_depth));
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> FragmentOutput {
    let t = -in.near_point.y / (in.far_point.y - in.near_point.y);
    let frag_pos_3d = in.near_point + t * (in.far_point - in.near_point);

    let depth = compute_depth(frag_pos_3d);

    var out: FragmentOutput;
    out.color = grid(frag_pos_3d, 2.0) * f32(t < 0.0);
    out.depth = depth;
    out.color.a = 0.01 * out.color.a * fading(frag_pos_3d, depth);

    return out;

}
