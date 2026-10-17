//! Grown stone's surface: the strata as a texture (Laz, 15 Oct: "just like
//! tree rings — some bigger, some smaller, some closer together, and waves
//! so they're not all perfectly horizontal"). One texture is made at start
//! and put on the lit material; a building's walls carry texture
//! coordinates (round the building, and height), everything else points
//! at a white corner of it and is unchanged. The strata's boundaries are
//! one table here, read by the texture and by the walls' ledges
//! (`interiors::strata`), so the two agree.
//!
//! The texture is an atlas: the first two columns white; then `STRATA_COLS`
//! columns of grown stone round a building; then the Qotiro's quarried
//! stone (`ASHLAR_COLS`: big regular blocks in courses, `ASHLAR_W` metres
//! along a wall before it repeats). `HEIGHT` rows for `TEX_H` metres of
//! height, repeating.

use bevy::image::{Image, ImageAddressMode};

pub const WIDTH: usize = 4096;
pub const HEIGHT: usize = 2048;
/// Metres of wall the texture's height covers before it repeats.
pub const TEX_H: f32 = 16.0;
/// The white columns at the left, for everything that isn't textured.
const WHITE_COLS: usize = 2;
/// Four regions side by side, a gap between each: the grown stone's
/// strata, the quarried stone's ashlar, the Horaro's basket weave, and
/// their plank decking. Each is `TEX_H` metres tall and repeats top to
/// bottom; across, each covers its own width in metres (below) and a face
/// is laid along it from its start.
const STRATA_COLS: usize = 1020;
const GAP_COLS: usize = 4;
const ASHLAR_COLS: usize = 1020;
const WEAVE_COLS: usize = 1020;
const PLANK_COLS: usize = WIDTH - WHITE_COLS - STRATA_COLS - ASHLAR_COLS - WEAVE_COLS - 3 * GAP_COLS;
const ASHLAR_AT: usize = WHITE_COLS + STRATA_COLS + GAP_COLS;
const WEAVE_AT: usize = ASHLAR_AT + ASHLAR_COLS + GAP_COLS;
const PLANK_AT: usize = WEAVE_AT + WEAVE_COLS + GAP_COLS;
/// Metres of a quarried wall its columns cover.
pub const ASHLAR_W: f32 = 16.0;
/// Metres round a woven hull its columns cover (a lobe's girth is laid
/// along it in laps).
pub const WEAVE_W: f32 = 8.0;
/// Metres across the boards the plank columns cover (the boards run up
/// the texture, so a deck's length is unlimited; its width is not).
pub const PLANK_W: f32 = 24.0;
/// Texture coordinates that land on the white columns.
pub const WHITE_UV: [f32; 2] = [1.0 / WIDTH as f32, 0.5];

/// The u coordinate for a spot `t` (0..1) of the way round a grown building.
pub fn u_of(t: f32) -> f32 {
    (WHITE_COLS as f32 + 0.5 + t.clamp(0.0, 1.0) * (STRATA_COLS - 1) as f32) / WIDTH as f32
}

/// The u coordinate `s` metres along a quarried wall (0..`ASHLAR_W`).
pub fn u_ashlar(s: f32) -> f32 {
    let t = (s / ASHLAR_W).clamp(0.0, 1.0);
    (ASHLAR_AT as f32 + 0.5 + t * (ASHLAR_COLS - 1) as f32) / WIDTH as f32
}

/// The u coordinate `s` metres round a woven hull (0..`WEAVE_W`).
pub fn u_weave(s: f32) -> f32 {
    let t = (s / WEAVE_W).clamp(0.0, 1.0);
    (WEAVE_AT as f32 + 0.5 + t * (WEAVE_COLS - 1) as f32) / WIDTH as f32
}

/// The u coordinate `s` metres across the boards of a deck (0..`PLANK_W`).
pub fn u_plank(s: f32) -> f32 {
    let t = (s / PLANK_W).clamp(0.0, 1.0);
    (PLANK_AT as f32 + 0.5 + t * (PLANK_COLS - 1) as f32) / WIDTH as f32
}

/// One stratum: how thick, and its tone against the stone's colour.
#[derive(Clone, Copy)]
pub struct Stratum {
    pub thick: f32,
    pub tone: f32,
    /// Its own wave on top of the shared one (amplitude, turns, phase).
    pub wave: (f32, f32, f32),
}

fn hash(seed: u64, k: u64) -> u64 {
    let mut h = seed ^ k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    h
}

fn unit(seed: u64, k: u64) -> f32 {
    (hash(seed, k) >> 40) as f32 / (1u64 << 24) as f32
}

/// The strata, bottom to top, filling exactly `TEX_H` metres. Like tree
/// rings: most a hand or two thick, a few broad, runs of thin ones close
/// together; the tone wanders a little from one to the next.
pub fn strata() -> Vec<Stratum> {
    let seed = 0x5EED_57A7_A000;
    let mut out = Vec::new();
    let mut total = 0.0;
    let mut tone = 0.86f32;
    let mut k = 0u64;
    let mut thin_run = 0;
    while total < TEX_H * 1.2 {
        k += 1;
        let r = unit(seed, k);
        let thick = if thin_run > 0 {
            thin_run -= 1;
            0.06 + unit(seed, k + 1000) * 0.08
        } else if r < 0.12 {
            thin_run = 2 + (unit(seed, k + 2000) * 4.0) as i32;
            0.07 + unit(seed, k + 1000) * 0.08
        } else if r < 0.28 {
            0.55 + unit(seed, k + 1000) * 0.45
        } else {
            0.16 + unit(seed, k + 1000) * 0.3
        };
        tone = (tone + (unit(seed, k + 3000) - 0.5) * 0.16).clamp(0.7, 1.0);
        let wave = (0.01 + unit(seed, k + 4000) * 0.03, (2.0 + (unit(seed, k + 5000) * 4.0).floor()), unit(seed, k + 6000) * std::f32::consts::TAU);
        out.push(Stratum { thick, tone, wave });
        total += thick;
    }
    // Scale to the period exactly, so the texture repeats without a step.
    let s = TEX_H / total;
    for st in &mut out {
        st.thick *= s;
    }
    out
}

/// The shared wave every stratum follows, metres up at `t` of the way round.
pub fn wave(t: f32) -> f32 {
    let a = t * std::f32::consts::TAU;
    0.07 * (a * 2.0 + 0.3).sin() + 0.045 * (a * 5.0 + 1.9).sin() + 0.025 * (a * 9.0 + 4.0).sin()
}

/// Where the top of stratum `i` (0-based, from the bottom of the period)
/// runs at `t` of the way round, metres above the period's base. `i` may
/// be any integer: the strata repeat every `TEX_H`.
pub fn top_of(strata: &[Stratum], i: i64, t: f32) -> f32 {
    let n = strata.len() as i64;
    let period = i.div_euclid(n);
    let j = i.rem_euclid(n) as usize;
    let mut y = period as f32 * TEX_H;
    for st in &strata[..=j] {
        y += st.thick;
    }
    let (amp, turns, phase) = strata[j].wave;
    y + wave(t) + amp * (t * std::f32::consts::TAU * turns + phase).sin()
}

/// The strata whose tops lie between heights `lo` and `hi` (metres above
/// the period's base, any range): their indices for `top_of`.
pub fn between(strata: &[Stratum], lo: f32, hi: f32) -> Vec<i64> {
    let n = strata.len() as i64;
    let first = (lo / TEX_H).floor() as i64 * n - 1;
    let last = (hi / TEX_H).ceil() as i64 * n + n;
    (first..=last).filter(|&i| {
        let y = top_of(strata, i, 0.0);
        y > lo - 0.5 && y < hi + 0.5
    }).collect()
}

/// Smooth value noise, repeating every `kt` cells across and `kv` up.
fn noise(t: f32, v: f32, kt: usize, kv: usize, seed: u64) -> f32 {
    let (x, y) = (t * kt as f32, v * kv as f32);
    let (i, j) = (x.floor(), y.floor());
    let (fx, fy) = (x - i, y - j);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let at = |di: i64, dj: i64| {
        let ci = (i as i64 + di).rem_euclid(kt as i64) as u64;
        let cj = (j as i64 + dj).rem_euclid(kv as i64) as u64;
        unit(seed, ci * 7919 + cj * 104729)
    };
    let a = at(0, 0) + (at(1, 0) - at(0, 0)) * sx;
    let b = at(0, 1) + (at(1, 1) - at(0, 1)) * sx;
    a + (b - a) * sy
}

/// The texture itself, with its smaller copies for distance.
pub fn texture() -> Image {
    let strata = strata();
    let n = strata.len();
    // Each stratum's top across the width, once.
    let cols = STRATA_COLS;
    let tops: Vec<Vec<f32>> = (0..n as i64).map(|i| (0..cols).map(|c| top_of(&strata, i, (c as f32) / (cols - 1) as f32)).collect()).collect();
    let mut px = vec![0u8; WIDTH * HEIGHT * 4];
    for c in 0..WIDTH {
        for r in 0..HEIGHT {
            let at = ((HEIGHT - 1 - r) * WIDTH + c) * 4;
            if c < WHITE_COLS {
                px[at..at + 4].copy_from_slice(&[255, 255, 255, 255]);
                continue;
            }
            if c >= WHITE_COLS + STRATA_COLS {
                // The other regions, each behind a gap in its own mean tone.
                let v = (r as f32 + 0.5) / HEIGHT as f32 * TEX_H;
                let (rr, gg, bb) = if c < ASHLAR_AT {
                    (0.80, 0.80, 0.80)
                } else if c < ASHLAR_AT + ASHLAR_COLS {
                    ashlar((c - ASHLAR_AT) as f32 / (ASHLAR_COLS - 1) as f32 * ASHLAR_W, v)
                } else if c < WEAVE_AT {
                    (0.42, 0.38, 0.30)
                } else if c < WEAVE_AT + WEAVE_COLS {
                    weave((c - WEAVE_AT) as f32 / (WEAVE_COLS - 1) as f32 * WEAVE_W, v)
                } else if c < PLANK_AT {
                    (0.46, 0.41, 0.34)
                } else {
                    plank((c - PLANK_AT) as f32 / (PLANK_COLS - 1) as f32 * PLANK_W, v)
                };
                px[at] = (rr.clamp(0.0, 1.0) * 255.0) as u8;
                px[at + 1] = (gg.clamp(0.0, 1.0) * 255.0) as u8;
                px[at + 2] = (bb.clamp(0.0, 1.0) * 255.0) as u8;
                px[at + 3] = 255;
                continue;
            }
            let t = (c - WHITE_COLS) as f32 / (cols - 1) as f32;
            let v = (r as f32 + 0.5) / HEIGHT as f32 * TEX_H;
            // Which stratum we're in, and how far from its edges.
            let cc = c - WHITE_COLS;
            let mut band = n - 1;
            let mut below = tops[n - 1][cc] - TEX_H;
            for i in 0..n {
                if tops[i][cc] > v {
                    band = i;
                    break;
                }
                below = tops[i][cc];
            }
            let above = tops[band][cc];
            let st = strata[band];
            let thick = (above - below).max(0.02);
            let f = ((v - below) / thick).clamp(0.0, 1.0);
            // The tone, darker toward the foot of each stratum (the ledge
            // above shades it), a dark crack along every seam, grain.
            let mut tone = st.tone * (0.9 + 0.1 * f);
            let d_seam = (v - below).min(above - v);
            tone *= 0.55 + 0.45 * (d_seam / 0.035).clamp(0.0, 1.0);
            let g = noise(t, v / TEX_H, 96, 192, 11) * 0.5 + noise(t, v / TEX_H, 24, 48, 12) * 0.35 + noise(t, v / TEX_H, 6, 12, 13) * 0.15;
            tone *= 0.9 + 0.2 * g;
            // Faint streaks down the stone where water runs.
            let streak = noise(t, v / TEX_H * 0.08, 160, 4, 14);
            tone *= 0.96 + 0.08 * streak;
            let tone = tone.clamp(0.0, 1.0);
            // A touch cooler in the dark bands, warmer in the light.
            let warm = (st.tone - 0.85) * 0.08;
            let (rr, gg, bb) = (tone * (1.0 + warm), tone, tone * (1.0 - warm * 0.5));
            px[at] = (rr.clamp(0.0, 1.0) * 255.0) as u8;
            px[at + 1] = (gg.clamp(0.0, 1.0) * 255.0) as u8;
            px[at + 2] = (bb.clamp(0.0, 1.0) * 255.0) as u8;
            px[at + 3] = 255;
        }
    }
    with_mips(px, WIDTH, HEIGHT, ImageAddressMode::Repeat, ImageAddressMode::Repeat)
}

/// The Qotiro's quarried stone at `s` metres along the wall and `v` up:
/// big regular blocks laid in courses, each course's blocks offset half a
/// block from the last, dark mortar joints, each block its own tone with
/// a little grain and a chipped edge or two. Repeats every `ASHLAR_W`
/// along and `TEX_H` up.
fn ashlar(s: f32, v: f32) -> (f32, f32, f32) {
    const COURSE: f32 = 0.6;
    const BLOCK: f32 = 1.6;
    const JOINT: f32 = 0.018;
    let course = (v / COURSE).floor();
    let cj = course as i64;
    // Blocks offset by half a block on alternate courses; a course's own
    // small variation in its blocks' lengths, keyed so it repeats.
    let offset = if cj.rem_euclid(2) == 0 { 0.0 } else { BLOCK * 0.5 };
    let along = (s + offset).rem_euclid(ASHLAR_W);
    let bi = (along / BLOCK).floor();
    let row = cj.rem_euclid((TEX_H / COURSE) as i64) as u64;
    // Each joint sits a little off its grid line (the same for the block
    // either side of it), so blocks run long and short; the row's last
    // joint stays on its line, so the wall repeats.
    let nb = (ASHLAR_W / BLOCK) as u64;
    let joint_at = |k: u64| -> f32 {
        let k = k % nb;
        if k == 0 { 0.0 } else { k as f32 * BLOCK + (unit(0xA5_A5, row * 977 + k) - 0.5) * 0.7 }
    };
    let end = |k: u64| if k + 1 >= nb { ASHLAR_W } else { joint_at(k + 1) };
    // The jittered joints may put this spot in the block before or after.
    let mut k = bi as u64;
    if k > 0 && along < joint_at(k) {
        k -= 1;
    } else if along >= end(k) && k + 1 < nb {
        k += 1;
    }
    let key = row * 131 + k;
    let x0 = joint_at(k);
    let x1 = end(k);
    let y0 = course * COURSE;
    let y1 = y0 + COURSE;
    let dx = (along - x0).min(x1 - along);
    let dy = (v - y0).min(y1 - v);
    let d = dx.min(dy);
    // Tone of the block, grain, the joint's shadow.
    let tone = 0.78 + (unit(0x5A5A, key) - 0.5) * 0.14;
    let g = noise(s / ASHLAR_W, v / TEX_H, 128, 128, 21) * 0.6 + noise(s / ASHLAR_W, v / TEX_H, 32, 32, 22) * 0.4;
    let mut l = tone * (0.92 + 0.16 * g);
    // The joint and a bevelled, chipped edge.
    let edge = (d / 0.045).clamp(0.0, 1.0);
    let chip = noise(s / ASHLAR_W, v / TEX_H, 256, 256, 23);
    let bevel = 0.72 + 0.28 * edge * (0.85 + 0.3 * chip).min(1.0);
    l *= bevel;
    if d < JOINT {
        l *= 0.45;
    }
    // Faint damp streaks down the wall.
    l *= 0.95 + 0.1 * noise(s / ASHLAR_W, v / TEX_H * 0.1, 96, 4, 24);
    // A touch cool.
    (l * 0.98, l, l * 1.03)
}

/// The Horaro's basket weave at `s` metres round and `v` up: strands a
/// hand wide over and under each other in a plain weave, each strand
/// rounded (light along its crown, dark at its edges), a little wobble
/// and its own tone, and the dark inside showing through the gaps
/// between them. Every third strand up is a thicker rib (the lashed
/// hoops the paintings show). Repeats every `WEAVE_W` round and `TEX_H`
/// up.
fn weave(s: f32, v: f32) -> (f32, f32, f32) {
    const CELL: f32 = 0.24;
    const GAP: f32 = 0.045;
    let (ts, tv) = (s / WEAVE_W, v / TEX_H);
    // The strands wander a little, so the weave isn't a grid.
    let ws = (noise(ts, tv, 12, 24, 31) - 0.5) * 0.06;
    let wv = (noise(ts, tv, 24, 12, 32) - 0.5) * 0.06;
    let (x, y) = (s + ws, v + wv);
    let (ci, cj) = ((x / CELL).floor(), (y / CELL).floor());
    let (fx, fy) = (x - ci * CELL, y - cj * CELL);
    // Which strand is on top in this cell: the checkerboard of a plain
    // weave. The upright strand runs along v in column ci, the lying one
    // along s in row cj.
    let upright_on_top = ((ci as i64) + (cj as i64)).rem_euclid(2) == 0;
    let rib = (cj as i64).rem_euclid(3) == 0;
    // How far across each strand (0 at its edge, 1 at its crown).
    let across = |f: f32, w: f32| {
        let m = (f - CELL * 0.5).abs();
        let half = (CELL - GAP) * 0.5 * w;
        if m > half { -1.0 } else { 1.0 - (m / half).powi(2) }
    };
    let up = across(fx, 1.0);
    let ly = across(fy, if rib { 1.15 } else { 0.95 });
    let tone_of = |key: u64, k: u64| 0.70 + (unit(key, k) - 0.5) * 0.34;
    let col_tone = tone_of(0x7E_A1, ci.rem_euclid(WEAVE_W / CELL) as u64 * 7 + 1);
    let row_tone = tone_of(0x7E_A2, cj.rem_euclid(TEX_H / CELL) as u64 * 11 + 3);
    let (top, bottom, top_tone, bot_tone) = if upright_on_top { (up, ly, col_tone, row_tone) } else { (ly, up, row_tone, col_tone) };
    let grain = 0.9 + 0.2 * noise(ts, tv, 160, 320, 33);
    let l = if top >= 0.0 {
        top_tone * (0.55 + 0.45 * top) * grain
    } else if bottom >= 0.0 {
        // The strand underneath, shaded by the one over it.
        bot_tone * (0.5 + 0.38 * bottom) * 0.8 * grain
    } else {
        // A gap: the dark inside.
        0.12 + 0.05 * noise(ts, tv, 64, 128, 34)
    };
    let l = if rib && ly >= 0.0 && (!upright_on_top || up < 0.0) { l * 0.92 } else { l };
    // Weathered wicker: grey-brown, greyer toward the top of a hull.
    let warm = 0.5 + 0.5 * (1.0 - (v / TEX_H).min(1.0));
    (l * (0.90 + 0.10 * warm), l * (0.84 + 0.05 * warm), l * (0.70 + 0.03 * warm))
}

/// The Horaro's deck boards at `s` metres across the boards and `v` along
/// them: boards a hand and a half wide with a thin dark gap between, each
/// run of boards 1.8–3.6 m long with its joints staggered against the
/// next board's, each board its own grey-brown tone and grain, silvered
/// by the weather. Repeats every `PLANK_W` across and `TEX_H` along.
fn plank(s: f32, v: f32) -> (f32, f32, f32) {
    const BOARD: f32 = 0.24;
    const GAP: f32 = 0.014;
    let bi = (s / BOARD).floor();
    let fx = s - bi * BOARD;
    let col = bi.rem_euclid(PLANK_W / BOARD) as u64;
    // The joints along this board's run: a seeded start, then boards of
    // 1.8–3.6 m, the pattern fitted to repeat every TEX_H.
    let mut joint = -1.0f32;
    let mut y = (unit(0x9B_01, col * 3 + 1)) * 2.0 - 2.0;
    let mut k = 0u64;
    let mut key = col * 97;
    while y < TEX_H {
        let len = 1.8 + unit(0x9B_02, col * 131 + k) * 1.8;
        let next = y + len;
        if v >= y && v < next {
            joint = (v - y).min(next - v);
            key = col * 97 + k;
            break;
        }
        y = next;
        k += 1;
    }
    if joint < 0.0 {
        joint = 1.0;
    }
    let tone = 0.72 + (unit(0x9B_03, key) - 0.5) * 0.26;
    let (ts, tv) = (s / PLANK_W, v / TEX_H);
    // Grain runs along the board: long streaks, a little mottle.
    let grain = noise(ts, tv, 384, 24, 41) * 0.5 + noise(ts, tv, 96, 6, 42) * 0.3 + noise(ts, tv, 48, 96, 43) * 0.2;
    let mut l = tone * (0.86 + 0.28 * grain);
    // The gap between boards and the shadow at a joint.
    let edge = fx.min(BOARD - fx);
    if edge < GAP {
        l *= 0.3;
    } else if edge < 0.03 {
        l *= 0.75 + 0.25 * ((edge - GAP) / 0.016);
    }
    if joint < 0.012 {
        l *= 0.35;
    } else if joint < 0.05 {
        l *= 0.8 + 0.2 * (joint / 0.05);
    }
    // Silvered grey-brown, warmer where the grain is fresh.
    let silver = unit(0x9B_04, key * 5 + 2);
    (l * (1.02 - 0.08 * silver), l * (0.92 - 0.02 * silver), l * (0.76 + 0.08 * silver))
}

/// An image from its full-size pixels, with its smaller copies for
/// distance (each a box filter of the last).
pub fn with_mips(px: Vec<u8>, width: usize, height: usize, u: ImageAddressMode, v: ImageAddressMode) -> Image {
    let mut data = px.clone();
    let (mut w, mut h) = (width, height);
    let mut last = px;
    let mut levels = 1;
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                for ch in 0..4 {
                    let mut s = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(w - 1);
                        let sy = (y * 2 + dy).min(h - 1);
                        s += last[(sy * w + sx) * 4 + ch] as u32;
                    }
                    next[(y * nw + x) * 4 + ch] = (s / 4) as u8;
                }
            }
        }
        data.extend_from_slice(&next);
        last = next;
        w = nw;
        h = nh;
        levels += 1;
    }
    super::turf::image(data, width, height, levels, u, v)
}
