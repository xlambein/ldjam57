#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    sprite_view_bindings::view,
}
#import bevy_pbr::{
    utils::coords_to_viewport_uv,
}

struct BlurSettings {
    blur_intensity: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>,
#endif
}

@group(2) @binding(0) var<uniform> settings: BlurSettings;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let upper = 5;
    let kernel_size = upper * 2 + 1;
    let texture_size = vec2<f32>(textureDimensions(texture));
    let texel_size = 1.0 / view.viewport.zw * settings.blur_intensity / f32(upper);
    var color = vec4(0.0);
    for (var x = -upper; x <= upper; x++) {
        for (var y = -upper; y <= upper; y++) {
            let uv = in.uv + vec2<f32>(f32(x) * texel_size.x, f32(y) * texel_size.y);
            let texel = textureSample(texture, texture_sampler, uv);
            color += texel * texel.a;
        }
    }
    color /= (f32(kernel_size * kernel_size));

    // Darken pixels out of focus
    const DARKEST_DEPTH: f32 = 100.0;
    color = vec4(color.rgb * (1.0 - settings.blur_intensity / (DARKEST_DEPTH + settings.blur_intensity)), color.a);

    return color;
}
