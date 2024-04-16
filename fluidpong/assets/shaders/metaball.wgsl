
@group(2) @binding(0)
var<uniform> color: vec4<f32>;
@group(2) @binding(1)
var<uniform> radius: f32;
@group(2) @binding(2)
var<uniform> metaball_positions: array<vec4<f32>, 128>;


@fragment
fn fragment(@builtin(position) FragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    var sum: f32 = 0.0;
    for (var i = 0; i < 128; i++) {
        let dist = distance(FragCoord.xy, vec2(metaball_positions[i].x, metaball_positions[i].y));
        sum += 1.0 / (dist * dist + 1.0);
    }
    return color * vec4<f32>(sum, sum, sum, 1.0);
}
