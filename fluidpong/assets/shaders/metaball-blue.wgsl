#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> color: vec4<f32>;
@group(2) @binding(1)
var<uniform> radius: f32;
@group(2) @binding(2)
var<uniform> metaballs: array<vec4<f32>, 1024>;
// array strides must be multiple of 16. 
// metaball.x : x position
// metaball.y : y position
// metaball.z : density
// metaball.w : velocity magnitude


@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var threshold: f32 = 0.4;

    var pos: vec2<f32> = mesh.world_position.xy;
    var sum: f32 = 0.0;

    for (var i = 0; i < 1024; i++) {
        var ball: vec4<f32> = metaballs[i];
        // skip if inactive ball
        if (ball.x != 0.0 && ball.y != 0.0 && ball.w != 0.0) {
            let dist = distance(pos, vec2(ball.x, ball.y));
            let influence = radius / (dist * dist + 1.0);
            sum += influence;
        }
    }

    if (sum > threshold) {
        return vec4<f32>(0.1, 0.2, 1.0, sum);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    
}
