#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> blur_intensity: f32;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // let kernel_size = (blur_intensity);
    // let upper = i32(blur_intensity * 1.0);
    let upper = 3;
    let kernel_size = upper * 2 + 1;
    let texture_size = vec2<f32>(textureDimensions(texture));
    // TODO we don't want to average over texels, but instead over viewport pixels
    let texel_size = 1.0 / texture_size * blur_intensity;
    var color = vec4(0.0);
    for (var x = -upper; x <= upper; x++) {
        for (var y = -upper; y <= upper; y++) {
            let uv = in.uv + vec2<f32>(f32(x) * texel_size.x, f32(y) * texel_size.y);
            color += textureSample(texture, texture_sampler, uv);
        }
    }
    color /= (f32(kernel_size * kernel_size));

    return color;
    // return textureSample(texture, texture_sampler, in.uv);
}
