//! Grass as a texture (Laz, 15 Oct), for the ground and the grown roofs:
//! one tile, `TILE` metres across, repeating, made at start. Like the
//! grass textures games use: clumps of blades, lighter at the tips,
//! darker soil and shadow between them, and broad soft patches of
//! yellower and bluer green. It only *modulates*: the colour underneath
//! (the land's by place, a roof's) stays what it was, so the same tile
//! serves lowland grass, dry grass and heath. Its mean brightness is
//! `MEAN`; the turf material lifts by that, so nothing gets darker.

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const SIZE: usize = 1024;
/// Metres one tile covers.
pub const TILE: f32 = 4.0;
/// The tile's mean brightness (what the material lifts by).
pub const MEAN: f32 = 0.80;

/// Texture coordinates for a spot on the ground (or a roof) at `x`, `z`.
pub fn uv_at(x: f32, z: f32) -> [f32; 2] {
    [x / TILE, z / TILE]
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

/// Smooth value noise over the tile, repeating every `k` cells.
fn noise(x: f32, y: f32, k: usize, seed: u64) -> f32 {
    let (fx, fy) = (x * k as f32, y * k as f32);
    let (i, j) = (fx.floor(), fy.floor());
    let (ux, uy) = (fx - i, fy - j);
    let (sx, sy) = (ux * ux * (3.0 - 2.0 * ux), uy * uy * (3.0 - 2.0 * uy));
    let at = |di: i64, dj: i64| {
        let ci = (i as i64 + di).rem_euclid(k as i64) as u64;
        let cj = (j as i64 + dj).rem_euclid(k as i64) as u64;
        unit(seed, ci * 7919 + cj * 104_729)
    };
    let a = at(0, 0) + (at(1, 0) - at(0, 0)) * sx;
    let b = at(0, 1) + (at(1, 1) - at(0, 1)) * sx;
    a + (b - a) * sy
}

/// The tile: brightness and a warm/cool lean per pixel, then colour.
pub fn texture() -> Image {
    let n = SIZE;
    let seed = 0x6A55_7EED;
    // Soil and shadow between the clumps, with broad soft patches.
    let mut lum = vec![0.0f32; n * n];
    let mut warm = vec![0.0f32; n * n];
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 / n as f32, y as f32 / n as f32);
            let broad = noise(fx, fy, 3, 1) * 0.6 + noise(fx, fy, 7, 2) * 0.4;
            let fine = noise(fx, fy, 64, 3) * 0.6 + noise(fx, fy, 160, 4) * 0.4;
            lum[y * n + x] = 0.72 + 0.12 * broad + 0.08 * fine;
            warm[y * n + x] = (noise(fx, fy, 4, 5) - 0.5) * 2.0;
        }
    }
    // Clumps of blades: a jittered grid of them, each a fan of strokes out
    // from its root, dark at the root, light at the tip.
    let grid = 48;
    let px_per_m = n as f32 / TILE;
    let mut plot = |x: f32, y: f32, w: f32, v: f32, lean: f32| {
        let r = w.ceil() as i64;
        let (cx, cy) = (x.round() as i64, y.round() as i64);
        for dy in -r..=r {
            for dx in -r..=r {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d > w {
                    continue;
                }
                let k = (1.0 - d / (w + 0.5)).clamp(0.0, 1.0);
                let at = ((cy + dy).rem_euclid(n as i64) as usize) * n + (cx + dx).rem_euclid(n as i64) as usize;
                lum[at] = lum[at] * (1.0 - k) + v * k;
                warm[at] = warm[at] * (1.0 - k * 0.5) + lean * k * 0.5;
            }
        }
    };
    for gy in 0..grid {
        for gx in 0..grid {
            let id = (gy * grid + gx) as u64;
            let cx = (gx as f32 + 0.15 + unit(seed, id * 3) * 0.7) / grid as f32 * n as f32;
            let cy = (gy as f32 + 0.15 + unit(seed, id * 3 + 1) * 0.7) / grid as f32 * n as f32;
            let blades = 8 + (unit(seed, id * 3 + 2) * 8.0) as usize;
            let size = 0.7 + unit(seed, id * 5 + 7) * 0.9;
            let lean = unit(seed, id * 5 + 9) * 2.0 - 1.0;
            // A dark patch of shadow and soil under the clump.
            let shade = 0.11 * px_per_m * size;
            let steps = 14;
            for s in 0..steps {
                let a = s as f32 / steps as f32 * std::f32::consts::TAU;
                plot(cx + a.cos() * shade * 0.5, cy + a.sin() * shade * 0.5, shade * 0.5, 0.5, -0.3);
            }
            for bl in 0..blades {
                let k = id * 97 + bl as u64 * 13;
                let dir = unit(seed, k) * std::f32::consts::TAU;
                let len = (0.09 + unit(seed, k + 1) * 0.13) * px_per_m * size;
                let bend = (unit(seed, k + 2) - 0.5) * 1.4;
                let bright = 0.9 + unit(seed, k + 3) * 0.25;
                let segs = (len / 0.7).ceil().max(4.0) as usize;
                for s in 0..=segs {
                    let t = s as f32 / segs as f32;
                    let a = dir + bend * t * t;
                    let x = cx + a.cos() * len * t;
                    let y = cy + a.sin() * len * t;
                    let w = (3.2 - 2.0 * t) * (n as f32 / 1024.0);
                    let v = 0.55 + (bright - 0.55) * t.powf(0.8);
                    plot(x, y, w, v, lean + 0.4 * t);
                }
            }
        }
    }
    // Bring the mean to `MEAN` (the material lifts by it), then colour:
    // brightness, leaning yellow where warm and blue-green where cool.
    let mean = lum.iter().map(|l| l.clamp(0.0, 1.0)).sum::<f32>() / lum.len() as f32;
    let fix = MEAN / mean.max(0.05);
    let mut px = vec![0u8; n * n * 4];
    for i in 0..n * n {
        let l = (lum[i] * fix).clamp(0.0, 1.0);
        let w = warm[i].clamp(-1.0, 1.0) * 0.07;
        let (r, g, b) = (l * (1.0 + w), l * (1.0 + w * 0.3), l * (1.0 - w * 1.2));
        px[i * 4] = (r.clamp(0.0, 1.0) * 255.0) as u8;
        px[i * 4 + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
        px[i * 4 + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
        px[i * 4 + 3] = 255;
    }
    super::stone::with_mips(px, n, n, ImageAddressMode::Repeat, ImageAddressMode::Repeat)
}

/// Sampler and sizes shared with the stone (kept here so the two match).
pub fn sampler(u: ImageAddressMode, v: ImageAddressMode) -> ImageSampler {
    ImageSampler::Descriptor(ImageSamplerDescriptor { address_mode_u: u, address_mode_v: v, mag_filter: ImageFilterMode::Linear, min_filter: ImageFilterMode::Linear, mipmap_filter: ImageFilterMode::Linear, anisotropy_clamp: 4, ..Default::default() })
}

/// An image of `w` × `h` pixels with its smaller copies for distance.
pub fn image(data: Vec<u8>, w: usize, h: usize, levels: u32, u: ImageAddressMode, v: ImageAddressMode) -> Image {
    let mut img = Image::new_uninit(Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 }, TextureDimension::D2, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::RENDER_WORLD);
    img.texture_descriptor.mip_level_count = levels;
    img.data = Some(data);
    img.sampler = sampler(u, v);
    img
}
