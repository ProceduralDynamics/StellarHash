#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

struct GiantStarMaterial {
    base_color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> material: GiantStarMaterial;

// 1. Générateur pseudo-aléatoire
fn hash(p: vec2<f32>) -> f32 {
    var p2 = fract(p * vec2<f32>(5.3983, 5.4427));
    p2 += dot(p2.yx, p2.xy + vec2<f32>(21.5351, 14.3137));
    return fract(p2.x * p2.y * 95.4337);
}

// 2. Bruit 2D interpolé (Simule des nuages de gaz)
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = globals.time * 0.5;
    let uv = (in.uv - 0.5) * 2.0;
    
    let n1 = noise(uv * 4.0 + time);
    let n2 = noise(uv * 8.0 - time * 1.5);
    let plasma = (n1 + n2 * 0.5) / 1.5;
    
    let contrast_plasma = pow(plasma, 4.0);
    let pulse = (sin(time * 2.0) + 1.0) * 0.5;

    let core_intensity = 0.05 + (contrast_plasma * 3.0) + (pulse * 0.1);

    let dist = length(uv);
    let deformation = (plasma - 0.5) * 0.1;
    let dist_ondule = dist + deformation;

    let corona = smoothstep(0.8, 0.8, dist_ondule) * 15.0;

    let masque = 1.0 - smoothstep(0.8, 0.95, dist_ondule);

    // Fusion des lumières
    let final_intensity = (core_intensity + corona) * masque;

    let color = material.base_color * vec4<f32>(final_intensity, final_intensity, final_intensity, 1.0);
    return vec4<f32>(color.rgb, masque);
}