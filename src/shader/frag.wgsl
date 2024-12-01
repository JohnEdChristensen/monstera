struct ScreenDimensions {
    width: f32,
    height: f32,
};


fn gradientNoise(uv: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(dot(uv, vec2<f32>(0.06711056, 0.00583715))));
}

const left_color = vec4<f32>(0.02, 0.04, 0.04, 1.0);
const right_color = vec4<f32>(0.015, 0.025, 0.025, 1.0);
const noise_strength = 2.0;


@group(0) @binding(0)
var<uniform> screenDimensions: ScreenDimensions;
@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {

    let normalized_x = pos.x / screenDimensions.width;
    let normalized_y = pos.y / screenDimensions.height;

    var color = mix(left_color, right_color, mix(0.0, 1.0, (normalized_x + normalized_y) / 2.0));

    color += (noise_strength / 255.0) * gradientNoise(pos.xy) - ((noise_strength * 0.5) / 255.0);
    return color;
}
