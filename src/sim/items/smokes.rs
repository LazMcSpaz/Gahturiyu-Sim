//! Smokes (Part 5, "Smokes"; Laz, 2026-10-11): bought only, like drinks.
//! Each has a haze (how much higher one smoke makes you, `Kind::Smoke`) and
//! one small effect for about an hour (`World::smoke`).

use super::*;
use crate::sim::effects::lasting;

const H: f32 = 3600.0;
const ME: Reach = Reach::Caster;

const fn smoke(key: &'static str, name: &'static str, weight: f32, value: f32, haze: f32, effects: &'static [Effect]) -> ItemDef {
    ItemDef { key, name, slot: Slot::MainHand, kind: Kind::Smoke(haze), weight, value, effects }
}

/// How much higher one smoke makes you (points on the 0..100 scale).
const MILD: f32 = 14.0;
const MEDIUM: f32 = 22.0;
const STRONG: f32 = 34.0;

pub(super) static ITEMS: &[ItemDef] = &[
    // Incense (cones and bricks), burned in Qotiro smoke halls.
    smoke("hall_incense", "Hall incense", 0.1, 3.0, MILD, &[lasting(Does::Steady, 0.1, H, ME)]),
    smoke("ghost_incense", "Ghost incense", 0.1, 9.0, STRONG, &[lasting(Does::EnergyRegen, 0.25, H, ME)]),
    smoke("sun_incense", "Sun incense", 0.1, 20.0, MILD, &[now(Does::Rouse, 1.0, ME)]),
    // Pipe blends (pouches).
    smoke("drift_smoke", "Drift smoke", 0.1, 2.0, MILD, &[lasting(Does::Warmth, 0.1, H, ME)]),
    smoke("elders_blend", "Elder's blend", 0.1, 5.0, MEDIUM, &[lasting(Does::Steady, 0.15, H, ME)]),
    smoke("sweet_blend", "Sweet water-pipe blend", 0.1, 6.0, MILD, &[lasting(Does::EnergyRegen, 0.15, H, ME)]),
];

pub(super) static FORMS: &[Form] = &[];

pub(super) static SIZES: &[(&str, u8, u8)] = &[];

/// Smoked in a pipe (the rest is incense, burned).
pub static PIPE_BLENDS: &[&str] = &["drift_smoke", "elders_blend", "sweet_blend"];
