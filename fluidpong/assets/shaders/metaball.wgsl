#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> color: vec4<f32>;
@group(2) @binding(1)
var<uniform> metaballs: array<vec4<f32>, 1728>;
// array strides must be multiple of 16.
// metaball.x : x position
// metaball.y : y position
// metaball.z : density
// metaball.w : velocity magnitude
const HUE_MIN: f32 = 0.67;
const HUE_MAX: f32 = 0.50;
const MAX_DENSITY: f32 = 10.0;
const MAX_SPEED: f32 = 3000.0;
const RADIUS: f32 = 3.0;
const THRESHOLD: f32 = 0.2;
const MIN_OPACITY: f32 = 0.2;
const MAX_OPACITY: f32 = 0.8;
const DIST_THRESHOLD: f32 = 30.0;

fn hsv2rgb(c: vec3<f32>) -> vec3<f32> {
    // assumes components are 0...1
    var K: vec4<f32> = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    var p: vec3<f32> = abs(fract(vec3<f32>(c.x) + vec3<f32>(K.x, K.y, K.z)) * 6.0 - vec3<f32>(K.w));
    return c.z * mix(vec3<f32>(K.x), vec3<f32>(
        clamp(p.x - (K.x), 0.0, 1.0),
        clamp(p.y - (K.x), 0.0, 1.0),
        clamp(p.z - (K.x), 0.0, 1.0),
    ), c.y);
}



@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pos: vec2<f32> = mesh.world_position.xy;
    var sum: f32 = 0.0;
    var density_sum: f32 = 0.0;
    var speed_sum: f32 = 0.0;

    for (var i = 0; i < 1728; i++) {
        var ball: vec4<f32> = metaballs[i];
        let dist = distance(pos, vec2(ball.x, ball.y));
        if dist < DIST_THRESHOLD {
            let influence = RADIUS / (dist*dist + 1.0);
            sum += influence;
            density_sum += ball.z / (dist + 1.0) / 6.0;
            speed_sum += ball.w / (dist + 1.0);
        }
    }

    let opacity = clamp(density_sum / MAX_DENSITY, MIN_OPACITY, MAX_OPACITY);
    let hue = clamp(HUE_MIN - (speed_sum / MAX_SPEED) * HUE_MAX, 0.0, 1.0);

    let colorhsv: vec3<f32> = vec3(hue, 1.0, 0.5);

    if (sum > THRESHOLD) {
        return vec4<f32>(hsv2rgb(colorhsv), opacity);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
