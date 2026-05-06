//! CHROMA console cheat implementation: hue-rotate palette entries in
//! the selected PC's sprite dictionaries.
//!
//! Loops over every dictionary entry of the PC's current sprite
//! dictionary, converts RGB565 → HSV, rotates hue by the configured
//! rotation, scales saturation/value by the configured percentages, and
//! writes the result back as RGB565.  Shadow pixels and the transparent
//! key (`0x07C0`) are skipped.
//!
//! Dictionary entries live on the shared [`FrameHolder`], and decompressed
//! sprite frames are cached as GPU textures in the renderer.  After
//! mutating the palette we clear the sprite cache so the next render
//! redecodes the affected frames with the new colours.

use super::{Engine, Host, LevelAssets, PendingChromaShift};
use robin_assets::frame_holder::TRANSPARENT_COLOR_16;

/// Apply all queued [`PendingChromaShift`]s. Called once per frame from
/// the game loop before rendering. Drains the queue on the way out.
pub fn drain_pending_chroma_shifts(
    engine: &mut Engine,
    host: &mut Host,
    assets: &LevelAssets,
    renderer: &mut crate::renderer::Renderer,
) {
    let shifts: Vec<PendingChromaShift> = std::mem::take(&mut host.pending_chroma_shifts);
    if shifts.is_empty() {
        return;
    }
    let mut any_applied = false;
    for shift in shifts {
        match apply_chroma_shift(engine, host, assets, &shift) {
            ChromaApplyOutcome::Applied(count) => {
                any_applied = true;
                host.pending_console_output
                    .push(format!("Patching finished : {count} pixels patched"));
            }
            ChromaApplyOutcome::NoDictionary => {
                host.pending_console_output
                    .push("Current sprite is not dico based".to_string());
            }
            ChromaApplyOutcome::MissingPc => {}
        }
    }
    if any_applied {
        // Palette mutated → every cached sprite texture is stale.
        // Easiest correct answer: drop the cache and let the render
        // pass redecode on demand.
        renderer.clear_sprite_cache();
    }
}

enum ChromaApplyOutcome {
    /// Shift applied — `usize` is the number of dictionary entries modified.
    Applied(usize),
    /// PC's current sprite has no dictionary (RLE-only).
    NoDictionary,
    /// PC entity or sprite script was missing — diagnostic only.
    MissingPc,
}

/// Apply a single chroma shift, reporting the number of pixels
/// modified so the caller can emit `"Patching finished : N pixels
/// patched"`.
fn apply_chroma_shift(
    engine: &Engine,
    host: &mut Host,
    _assets: &LevelAssets,
    shift: &PendingChromaShift,
) -> ChromaApplyOutcome {
    // Resolve the PC's current sprite frame → bank id → dictionary index.
    let entity = match engine.get_entity(shift.pc_entity_id) {
        Some(e) => e,
        None => {
            tracing::warn!("CHROMA: PC entity {:?} missing", shift.pc_entity_id);
            return ChromaApplyOutcome::MissingPc;
        }
    };
    let sprite = entity.sprite();
    let scripts = match sprite.current_scripts_opt() {
        Some(s) => s,
        None => return ChromaApplyOutcome::MissingPc,
    };
    let row = sprite.current_row as usize;
    let frame = sprite.current_frame as usize;
    if row >= scripts.len() || frame >= scripts[row].frame_ids.len() {
        return ChromaApplyOutcome::MissingPc;
    }
    let bank_id = scripts[row].frame_ids[frame];
    let dict_idx = host.frame_holder.dictionary_index(bank_id);
    if dict_idx == robin_assets::frame_holder::UNMAPPED_DICT {
        return ChromaApplyOutcome::NoDictionary;
    }

    // Apply to day + variant dictionaries: the variant dictionaries
    // derive from day via `FrameDictionary::with_variant`, but because
    // they are pre-baked we have to shift each variant independently.
    let mut count: usize = 0;
    let fh = host.frame_holder_mut();
    let shadow = fh.global_shadow();
    // Day
    if let Some(d) = fh.dictionaries_mut().get_mut(dict_idx as usize) {
        count += shift_dictionary(d.raw_data_mut(), shadow, shift);
    }
    ChromaApplyOutcome::Applied(count)
}

/// RGB565 → HSV → rotate → RGB565 palette shift.  Skips the transparent
/// key (`0x07C0`) and the shadow colour, and clamps S/V after scaling.
fn shift_dictionary(values: &mut [u16], shadow_color: u16, shift: &PendingChromaShift) -> usize {
    let mut count: usize = 0;
    let saturation_scale = shift.saturation_pct / 100.0;
    let value_scale = shift.value_pct / 100.0;
    for v in values.iter_mut() {
        let px = *v;
        if px == shadow_color || px == TRANSPARENT_COLOR_16 {
            continue;
        }
        let r = ((px >> 11) & 0x1F) as f32 / 32.0;
        let g = ((px >> 5) & 0x3F) as f32 / 64.0;
        let b = (px & 0x1F) as f32 / 32.0;
        let (h, s, val) = rgb_to_hsv(r, g, b);
        if h < shift.start_hue || h > shift.end_hue {
            continue;
        }
        let mut new_h = h + shift.rotation;
        let mut new_s = s * saturation_scale;
        let mut new_v = val * value_scale;
        if new_h >= 360.0 {
            new_h -= 360.0;
        }
        if new_h < 0.0 {
            new_h += 360.0;
        }
        if new_s > 1.0 {
            new_s = 1.0;
        }
        if new_v > 1.0 {
            new_v = 1.0;
        }
        let (nr, ng, nb) = hsv_to_rgb(new_h, new_s, new_v);
        let packed = (((nr * 32.0) as u16) << 11) & 0xF800
            | (((ng * 64.0) as u16) << 5) & 0x07E0
            | ((nb * 32.0) as u16) & 0x001F;
        if packed != *v {
            *v = packed;
        }
        // Count every pixel that matched the hue window, even if the
        // RGB round-trip happened to land on the same packed value.
        count += 1;
    }
    count
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max;
    let s = if max > 0.0 { delta / max } else { 0.0 };
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h6 = h / 60.0;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = if (0.0..1.0).contains(&h6) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h6) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h6) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h6) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h6) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}
