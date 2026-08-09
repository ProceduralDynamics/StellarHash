#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

struct StarMaterial {
    base_color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> material: StarMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let star_center = round(in.world_position.xy / 80.0) * 80.0;
    let seed = star_center.x * 0.1337 + star_center.y * 0.7331;
    let time = globals.time;

    let twinkle = (sin(time * 3.0 + seed) + 1.0) * 0.5;

    let intensity = 0.9 + 0.8 * twinkle;

    return material.base_color * vec4<f32>(intensity, intensity, intensity, 1.0);
}