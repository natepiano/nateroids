import bevy_pbr::render::{
    mesh_functions::{get_tag, get_world_from_local},
    mesh_view_bindings::view,
    forward_io::Vertex,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
};

@if(PREPASS_PIPELINE)
import bevy_pbr::{
    prepass::io::{VertexOutput, FragmentOutput},
    deferred::functions::deferred_output,
};
@else
import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
};

struct StarData {
    emissive: vec4<f32>,
    // Initial phase, amplitude fraction, speed fraction, alignment padding.
    twinkle: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<storage, read> stars: array<StarData>;

// Integrated master phase and amplitude; z/w are alignment padding.
@group(constants::MATERIAL_BIND_GROUP) @binding(101)
var<storage, read> clock: vec4<f32>;

// Minimum full width at half maximum in physical render pixels.
const MIN_DIAMETER_PIXELS: f32 = 1.0;
const FWHM_TO_SIGMA: f32 = 0.4246609;

// Project the original sphere radius without changing its world-space transform.
fn footprint(instance_index: u32) -> vec4<f32> {
    let model = get_world_from_local(instance_index);
    let center = view.clip_from_world * model[3];
    let radius = length(model[0].xyz) * abs(view.clip_from_view[1][1])
        * view.viewport.w * 0.5 / max(abs(center.w), 0.0001);
    let pixel_center = view.viewport.xy
        + (center.xy / center.w * vec2(0.5, -0.5) + 0.5) * view.viewport.zw;
    // This width preserves the original disk's peak and integrated light for
    // larger stars. Enlarging tiny stars instead lowers their peak brightness.
    let sigma = max(radius * 0.7071068, MIN_DIAMETER_PIXELS * FWHM_TO_SIGMA);
    return vec4(pixel_center, radius, sigma);
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let model = get_world_from_local(in.instance_index);
    let center = view.clip_from_world * model[3];
    let spot = footprint(in.instance_index);
    // Include the Gaussian tails and the half-pixel integration margin.
    let offset = in.position.xy * (4.0 * spot.w + 0.5);
    out.position = center;
    out.position.x += offset.x * 2.0 / view.viewport.z * center.w;
    out.position.y += offset.y * 2.0 / view.viewport.w * center.w;
    out.world_position = model[3];
    out.world_normal = normalize(view.world_position - model[3].xyz);
    out.uv = in.uv;
    out.instance_index = in.instance_index;
    return out;
}

// Approximate the normal CDF, allowing integration over each pixel instead of
// point sampling a bright spot as it moves between pixel centers.
fn normal_cdf(x: vec2<f32>) -> vec2<f32> {
    let a = abs(x);
    let t = 1.0 / (1.0 + 0.2316419 * a);
    let tail = 0.39894228 * exp(-0.5 * a * a) * t
        * (0.31938153 + t * (-0.35656378 + t
        * (1.78147794 + t * (-1.82125598 + t * 1.33027443))));
    return select(tail, 1.0 - tail, x >= vec2(0.0));
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    let star = stars[get_tag(in.instance_index)];
    let phase = star.twinkle.x + star.twinkle.z * clock.x;
    let factor = max(1.0 + clock.y * star.twinkle.y * sin(phase), 0.0);

    let spot = footprint(in.instance_index);
    let offset = in.position.xy - spot.xy;
    let integrated = normal_cdf((offset + 0.5) / spot.w)
        - normal_cdf((offset - 0.5) / spot.w);
    // The old emissive disk carried pi * radius^2 units of light.
    let coverage = 3.14159265 * spot.z * spot.z * integrated.x * integrated.y;

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    // StandardMaterial stores exposure weight in emissive.a, not color alpha.
    pbr_input.material.emissive = vec4<f32>(
        star.emissive.rgb * factor * coverage,
        pbr_input.material.emissive.a,
    );
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color,
    );

    @if(PREPASS_PIPELINE) {
        return deferred_output(in, pbr_input);
    } @else {
        var out: FragmentOutput;
        out.color = apply_pbr_lighting(pbr_input);
        out.color = main_pass_post_lighting_processing(pbr_input, out.color);
        return out;
    }
}
