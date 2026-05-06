// Sprite-occlusion mask overlay. Mirrors `quad.wgsl`'s vertex layout
// 1:1 so the same per-quad VBO feeds both pipelines, but the fragment
// stage samples *two* textures: the static binary mask alpha (set
// once at level load from `RuntimeMask::bitmap`) and the live
// background colour. Where the mask alpha is non-zero, the fragment
// outputs the bg pixel — drawing the building over a sprite that's
// supposed to be occluded by it. CPU-side mask+bg compositing
// (`compose_mask_with_background`) is gone; the per-blit recompose
// churn that triggered amdgpu GTT exhaustion went with it.
//
// `tint` is repurposed: tint.xy = bbox top-left / bg_size, tint.zw =
// mask extent / bg_size. The fragment computes the bg sample uv as
// `tint.xy + in.uv * tint.zw`, since `in.uv` already linearly
// interpolates 0..1 across the quad.

struct ScreenUniform {
    screen_size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> screen: ScreenUniform;
@group(1) @binding(0) var mask_alpha: texture_2d<f32>;
@group(1) @binding(1) var bg_color:   texture_2d<f32>;
@group(1) @binding(2) var samp:       sampler;

struct VsIn {
    @location(0) pos:  vec2<f32>,
    @location(1) uv:   vec2<f32>,
    @location(2) tint: vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv:   vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(vin: VsIn) -> VsOut {
    let ndc_x =  (vin.pos.x / screen.screen_size.x) * 2.0 - 1.0;
    let ndc_y = -(vin.pos.y / screen.screen_size.y) * 2.0 + 1.0;
    var out: VsOut;
    out.clip_pos = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = vin.uv;
    out.tint = vin.tint;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let m = textureSample(mask_alpha, samp, in.uv).r;
    let bg_uv = in.tint.xy + in.uv * in.tint.zw;
    let c = textureSample(bg_color, samp, bg_uv);
    // Premultiplied output paired with `BlendMode::Blend`. Matches the
    // original CPU compose's edge falloff: bitmap values were 0x00 /
    // 0xFF, the bg pixel was stored only at 0xFF, so a linear-filtered
    // edge effectively pre-multiplied rgb by alpha.
    return vec4<f32>(c.rgb * m, m);
}
