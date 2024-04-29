#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> width: f32;
@group(2) @binding(1)
var<uniform> height: f32;
@group(2) @binding(2)
var<uniform> cells: array<vec4<f32>, 4096>;
// array strides must be multiple of 16. 
// cell.x : vx
// cell.y : vy
// cell.z : density
// cell.w : --
const GRID_LENGTH: u32 = 64;
const HUE_MIN: f32 = 0.67;
const HUE_MAX: f32 = 0.50;
const MAX_VEL_MAGNITUDE: f32 = 10000.0;

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

fn bilinear(v0: f32, v1: f32, v2: f32, v3: f32, fracX: f32, fracY: f32) -> f32 {
    var ix0: f32 = mix(v0, v1, fracX);
    var ix1: f32 = mix(v2, v3, fracX);
    return mix(ix0, ix1, fracY);
}

fn mag(cell: vec4<f32>) -> f32 {
    return length(vec2<f32>(cell.x, cell.y));
}

// grid
// @fragment
// fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
//     var pos: vec2<f32> = mesh.world_position.xy;

//     var cellX: u32 = u32((pos.x + width/2.0) / width * f32(GRID_LENGTH));
//     var cellY: u32 = u32((pos.y + height/2.0) / height * f32(GRID_LENGTH));

//     var i: u32 = cellY * GRID_LENGTH + cellX;
//     var cell: vec4<f32> = cells[i];
//     var d: f32 = cell[2];

//     var dir: vec2<f32> = normalize(vec2<f32>(cell.x, cell.y));
//     var mag: f32 = length(vec2<f32>(cell.x, cell.y));
//     var hue: f32 = HUE_MIN - (mag / MAX_VEL_MAGNITUDE) * HUE_MAX;
//     var color: vec3<f32> = hsv2rgb(vec3<f32>(hue, 1.0, 1.0));

//     return vec4<f32>(color, d);
// }

// with bilinear interpolation
@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pos: vec2<f32> = mesh.world_position.xy;
    
    var gridX: f32 = (pos.x + width / 2.0) / width * f32(GRID_LENGTH);
    var gridY: f32 = (pos.y + height / 2.0) / height * f32(GRID_LENGTH);
    
    var baseX: u32 = u32(gridX);
    var baseY: u32 = u32(gridY);
    var fracX: f32 = fract(gridX);
    var fracY: f32 = fract(gridY);
    
    var i0: u32 = baseY * GRID_LENGTH + baseX;
    var i1: u32 = baseY * GRID_LENGTH + min(baseX + 1, GRID_LENGTH - 1);
    var i2: u32 = min(baseY + 1, GRID_LENGTH - 1) * GRID_LENGTH + baseX;
    var i3: u32 = min(baseY + 1, GRID_LENGTH - 1) * GRID_LENGTH + min(baseX + 1, GRID_LENGTH - 1);
    
    var c0: vec4<f32> = cells[i0];
    var c1: vec4<f32> = cells[i1];
    var c2: vec4<f32> = cells[i2];
    var c3: vec4<f32> = cells[i3];
    
    var d = bilinear(c0[2], c1[2], c2[2], c3[2], fracX, fracY);
    var m = bilinear(mag(c0), mag(c1), mag(c2), mag(c3), fracX, fracY);

    var hue: f32 = HUE_MIN - (m / MAX_VEL_MAGNITUDE) * HUE_MAX;
    var color: vec3<f32> = hsv2rgb(vec3<f32>(hue, 1.0, 1.0));
    
    return vec4<f32>(color, d);
}
