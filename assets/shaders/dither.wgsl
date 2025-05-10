#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::view::View

@group(0) @binding(0) var<uniform> view: View;

const BAYER_DITHER: mat4x4<f32> = mat4x4(
    0.0, 8.0, 2.0, 10.0,
    12.0, 4.0, 14.0, 6.0,
    3.0, 11.0, 1.0, 9.0,
    15.0, 7.0, 13.0, 5.0
) / 16.0;

fn random(p: vec2f) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn gradient_noise(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let a = random(i);
    let b = random(i + vec2f(1.0, 0.0));
    let c = random(i + vec2f(0.0, 1.0));
    let d = random(i + vec2f(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2f) -> f32 {
    var total = 0.0;
    var frequency = 0.4;
    var amplitude = 0.5;
    for (var i = 0; i < 5; i = i + 1) {
        total += gradient_noise(p * frequency) * amplitude;
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return total;
}

struct Settings {
    offset: vec2f,
    fill: f32,
    flags: u32,
    scale: f32,
    padding: vec3f,
}

@group(2) @binding(0) var<uniform> settings: Settings;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var uv = vec2f(0.0);
    if((settings.flags & 0x1) != 0) {
        uv = mesh.position.xy; // Use the mesh position directly
    } else {
        uv = mesh.uv.xy;
    }
    uv = uv * settings.scale + settings.offset;

    var dither = 0.0;
    if((settings.flags & 0x2) != 0) {
        dither = BAYER_DITHER[u32(uv.y) % 4][u32(uv.x) % 4];
    } else if((settings.flags & 0x4) != 0) {
        dither = fbm(floor(uv + settings.offset));
    } else {
        dither = random(floor(uv + settings.offset));
    }

    if(dither <= settings.fill) {
        return vec4(1.0);
    }
    var alpha = 1.0;
    if((settings.flags & 0x8) != 0) {
        alpha = 0.0;
    }
    return vec4(vec3(0.0), alpha);
}