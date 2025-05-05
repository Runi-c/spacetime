#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

const BAYER_DITHER: mat4x4<f32> = mat4x4(
    0.0, 8.0, 2.0, 10.0,
    12.0, 4.0, 14.0, 6.0,
    3.0, 11.0, 1.0, 9.0,
    15.0, 7.0, 13.0, 5.0
) / 16.0;

fn random(c: vec2f) -> f32 {
  return fract(sin(dot(c.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

@group(2) @binding(0) var<uniform> fill: f32;
@group(2) @binding(1) var<uniform> dither_type: u32;
@group(2) @binding(2) var<uniform> dither_scale: f32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let resolution = view.viewport.zw;
    var uv = mesh.uv.xy * resolution * dither_scale; // Convert from [0, 1] to [-1, 1]
    uv.x *= resolution.y / resolution.x; // Adjust for aspect ratio
    //uv /= resolution.xy;

    //if(true) {return vec4<f32>(uv.x, uv.y, 0.0, 1.0);}
    //let dither = BAYER_DITHER[u32(uv.y) % 4][u32(uv.x) % 4];
    let dither = random(round(uv));
    if(dither <= fill) {
        return vec4(1.0);
    }
    return vec4(vec3(0.0), 1.0);
}