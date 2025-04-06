#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    sprite_view_bindings::view,
}
#import bevy_pbr::{
    utils::coords_to_viewport_uv,
}

struct LevelSettings {
    focus_depth: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>,
#endif
}

@group(2) @binding(0) var<uniform> settings: LevelSettings;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var depths: texture_2d<f32>;
@group(2) @binding(4) var depths_sampler: sampler;

const BLUR_SCALING: f32 = 1.5;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_size = vec2<f32>(textureDimensions(texture));
    let depths_size = vec2<f32>(textureDimensions(depths));
    let tile_per_texel = depths_size / texture_size;
    let depth = sqrt(textureSample(depths, depths_sampler, in.uv).r) * 10.0;
    let blur_intensity = abs(settings.focus_depth - depth) * BLUR_SCALING;

    if (textureSample(depths, depths_sampler, in.uv).a < 0.5) {
        return textureSample(texture, texture_sampler, in.uv);
    }

    let upper = 5;
    let kernel_size = upper * 2 + 1;
    let step_size = 1.0 / view.viewport.zw * blur_intensity / f32(upper);
    var color = vec4(0.0);
    for (var x = -upper; x <= upper; x++) {
        for (var y = -upper; y <= upper; y++) {
            let uv = in.uv + vec2<f32>(f32(x) * step_size.x, f32(y) * step_size.y);
            let texel = textureSample(texture, texture_sampler, uv);
            color += texel * texel.a;
        }
    }
    color /= (f32(kernel_size * kernel_size));

    // Darken pixels out of focus
    const DARKEST_DEPTH: f32 = 10.0;
    color = vec4(color.rgb * (1.0 - blur_intensity / (DARKEST_DEPTH + blur_intensity)), color.a);

    return color;
}
