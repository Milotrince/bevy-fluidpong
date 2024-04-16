#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> color: vec4<f32>;
@group(2) @binding(1)
var<uniform> radius: f32;
@group(2) @binding(2)
var<uniform> metaball_positions: array<vec4<f32>, 1024>;


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pos: vec2<f32> = mesh.world_position.xy;
    // var pos: vec2<f32> = vec2<f32>(mesh.world_position.x + 200.0 / 2.0, mesh.world_position.y + 200.0 / 2.0);
    var sum: f32 = 0.0;
    for (var i = 0; i < 1024; i++) {
        let dist = distance(pos, vec2(metaball_positions[i].x, metaball_positions[i].y));
        sum += radius / (dist * dist + 1.0);
    }

    var threshold: f32 = 0.5;
    if (sum > threshold) {
        return color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return color * vec4<f32>(sum, sum, sum, 1.0);

    // return vec4<f32>(vec3<f32>((mesh.world_position.y + 200.0 / 2.) / 200.0), 1.);
}
