//! What things look like (Laz's items step 2; drawing only).
//!
//! Every item is a few boxes, listed here by its form: a blade, a guard and
//! a grip; a shaft and a head; a coat's body and sleeves. The same boxes are
//! drawn wherever the thing is seen: in someone's hand, worn on the figure,
//! lying on the ground, and as its icon in the panels, so they always agree.
//! Boxes take their colour from what the thing is made of (`Tint::Main` is
//! its main material, so an Edgeglass knife and a Forgeiron one differ).
//! Placeholder shapes: the final models (Blender) can replace them at the
//! same grip and the same parts later.
//!
//! A shape is laid out in the item's own frame, in metres: *along* it (a
//! weapon's grip at 0, its point outwards; a worn thing from the bottom up),
//! *across* it, and its *thickness* (out of the front).

use bevy::math::{vec3, Vec3};
use bevy_egui::egui::{self, Color32};

use gahturiyu_sim::sim::{
    items::{info, item, ItemId, Kind, Slot},
    materials::Material,
    person::PersonId,
    World,
};

use super::hud::Canvas;
use super::mesh::Builder;
use super::palette::Rgb;

/// What a box is coloured by.
#[derive(Clone, Copy)]
pub enum Tint {
    /// The thing's main material (cloth takes a plain undyed colour).
    Main,
    /// Its main material, shaded darker (seams, scale rows, a collar).
    Dark,
    /// Its main material, a little brighter (a ridge, a boss).
    Light,
    /// Its backing material, else the main.
    Second,
    Wood,
    Leather,
    Iron,
    Paper,
    Gold,
    /// A bowstring, a cord.
    Cord,
    /// Pitch on a torch's head, a seal.
    Pitch,
    /// The thing's own colour as it lies about (`squadui::ground_color`,
    /// or its material's), for things with no material of their own.
    Own,
    Rgb(Rgb),
}

/// One box of a shape: its centre and half its size, (along, across,
/// thickness), and its colour.
#[derive(Clone, Copy)]
pub struct Bit {
    pub at: [f32; 3],
    pub half: [f32; 3],
    pub tint: Tint,
}

/// How a shape is held and shown.
#[derive(Clone, Copy, PartialEq)]
pub enum Pose {
    /// A weapon, a tool: held by its grip, shown slanting across its icon.
    Long,
    /// A bow: held upright by its middle, string towards the archer.
    Bow,
    /// Worn or set down: shown standing, along is up.
    Upright,
}

/// A box from `l0` to `l1` along, `a` across and `t` thick, on the line.
const fn bx(l0: f32, l1: f32, a: f32, t: f32, tint: Tint) -> Bit {
    Bit { at: [(l0 + l1) * 0.5, 0.0, 0.0], half: [(l1 - l0) * 0.5, a * 0.5, t * 0.5], tint }
}

/// The same, its middle `x` across from the line.
const fn bxo(l0: f32, l1: f32, x: f32, a: f32, t: f32, tint: Tint) -> Bit {
    Bit { at: [(l0 + l1) * 0.5, x, 0.0], half: [(l1 - l0) * 0.5, a * 0.5, t * 0.5], tint }
}

/// The same, its middle `x` across and `z` out of the front.
const fn bxz(l0: f32, l1: f32, x: f32, a: f32, z: f32, t: f32, tint: Tint) -> Bit {
    Bit { at: [(l0 + l1) * 0.5, x, z], half: [(l1 - l0) * 0.5, a * 0.5, t * 0.5], tint }
}

use Pose::*;
use Tint::*;

const SHIRT: &[Bit] = &[bx(0.40, 0.85, 0.38, 0.20, Main), bxo(0.55, 0.85, -0.25, 0.10, 0.15, Main), bxo(0.55, 0.85, 0.25, 0.10, 0.15, Main)];
const LEGS: &[Bit] = &[bx(0.45, 0.55, 0.34, 0.20, Main), bxo(0.0, 0.47, -0.09, 0.14, 0.16, Main), bxo(0.0, 0.47, 0.09, 0.14, 0.16, Main)];
const BOOTS: &[Bit] = &[
    bxo(0.0, 0.22, -0.08, 0.09, 0.10, Main),
    bxo(0.0, 0.22, 0.08, 0.09, 0.10, Main),
    bxz(0.0, 0.06, -0.08, 0.09, 0.08, 0.12, Dark),
    bxz(0.0, 0.06, 0.08, 0.09, 0.08, 0.12, Dark),
];
const GLOVES: &[Bit] = &[bxo(0.0, 0.15, -0.08, 0.07, 0.04, Main), bxo(0.15, 0.2, -0.08, 0.08, 0.05, Dark), bxo(0.0, 0.15, 0.08, 0.07, 0.04, Main), bxo(0.15, 0.2, 0.08, 0.08, 0.05, Dark)];
const RING: &[Bit] = &[
    bxo(0.0, 0.006, 0.0, 0.03, 0.006, Main),
    bxo(0.024, 0.03, 0.0, 0.03, 0.006, Main),
    bxo(0.0, 0.03, -0.012, 0.006, 0.006, Main),
    bxo(0.0, 0.03, 0.012, 0.006, 0.006, Main),
    bxz(0.026, 0.036, 0.0, 0.012, 0.0, 0.012, Light),
];
const NECKLACE: &[Bit] = &[
    bxo(0.10, 0.20, -0.07, 0.012, 0.01, Main),
    bxo(0.10, 0.20, 0.07, 0.012, 0.01, Main),
    bxo(0.05, 0.11, -0.04, 0.012, 0.01, Main),
    bxo(0.05, 0.11, 0.04, 0.012, 0.01, Main),
    bx(0.0, 0.05, 0.035, 0.02, Light),
];

/// Every form's shape, by its key. Things not listed take one by their
/// kind (`fallback`).
static SHAPES: &[(&str, Pose, &[Bit])] = &[
    // --- Blades ------------------------------------------------------------
    ("knife", Long, &[bx(-0.10, 0.0, 0.03, 0.025, Leather), bx(0.0, 0.012, 0.06, 0.03, Iron), bx(0.012, 0.20, 0.035, 0.008, Main)]),
    ("hatchet", Long, &[bx(-0.05, 0.42, 0.03, 0.03, Wood), bxo(0.32, 0.42, 0.06, 0.10, 0.015, Main), bxo(0.34, 0.40, -0.025, 0.03, 0.02, Main)]),
    ("short_sword", Long, &[bx(-0.16, -0.13, 0.045, 0.045, Iron), bx(-0.13, 0.0, 0.032, 0.03, Leather), bx(0.0, 0.02, 0.14, 0.035, Iron), bx(0.02, 0.55, 0.05, 0.01, Main)]),
    ("longsword", Long, &[bx(-0.24, -0.20, 0.05, 0.05, Iron), bx(-0.20, 0.0, 0.032, 0.03, Leather), bx(0.0, 0.025, 0.22, 0.035, Iron), bx(0.025, 0.92, 0.055, 0.01, Main)]),
    ("dirk", Long, &[bx(-0.11, 0.0, 0.03, 0.028, Leather), bx(0.0, 0.015, 0.11, 0.03, Iron), bx(0.015, 0.30, 0.04, 0.009, Main)]),
    ("cutlass", Long, &[bx(-0.13, 0.0, 0.032, 0.03, Leather), bxo(-0.13, 0.02, -0.04, 0.012, 0.03, Iron), bx(0.0, 0.02, 0.12, 0.035, Iron), bxo(0.02, 0.45, 0.01, 0.06, 0.01, Main), bxo(0.45, 0.62, 0.02, 0.045, 0.01, Main)]),
    ("greatsword", Long, &[bx(-0.40, -0.35, 0.06, 0.06, Iron), bx(-0.35, 0.0, 0.035, 0.035, Leather), bx(0.0, 0.03, 0.32, 0.04, Iron), bx(0.03, 1.25, 0.07, 0.012, Main)]),
    ("battle_axe", Long, &[bx(-0.10, 1.05, 0.045, 0.045, Wood), bxo(0.82, 1.08, 0.12, 0.22, 0.02, Main), bxo(0.86, 1.04, -0.05, 0.08, 0.03, Main)]),
    // --- Blunt -------------------------------------------------------------
    ("sap", Long, &[bx(-0.05, 0.12, 0.03, 0.03, Leather), bx(0.12, 0.24, 0.06, 0.06, Main)]),
    ("mace", Long, &[bx(-0.05, 0.50, 0.035, 0.035, Wood), bx(0.48, 0.66, 0.11, 0.11, Main), bx(0.52, 0.62, 0.16, 0.05, Main), bx(0.52, 0.62, 0.05, 0.16, Main)]),
    ("knuckle_guards", Upright, &[bx(0.0, 0.03, 0.12, 0.04, Main), bxz(0.03, 0.06, 0.0, 0.11, 0.0, 0.025, Main)]),
    ("club", Long, &[bx(-0.05, 0.35, 0.04, 0.04, Main), bx(0.35, 0.70, 0.075, 0.075, Main)]),
    ("war_pick", Long, &[bx(-0.05, 0.72, 0.035, 0.035, Wood), bxo(0.64, 0.70, 0.06, 0.28, 0.04, Main)]),
    ("stone_maul", Long, &[bx(-0.05, 0.90, 0.045, 0.045, Wood), bx(0.82, 1.08, 0.30, 0.20, Main)]),
    ("staff", Long, &[bx(-0.10, 1.70, 0.04, 0.04, Main), bx(1.70, 1.80, 0.06, 0.06, Dark)]),
    // --- Spears ------------------------------------------------------------
    ("spear", Long, &[bx(-0.10, 1.95, 0.035, 0.035, Wood), bx(1.95, 2.20, 0.06, 0.012, Main)]),
    (
        "trident",
        Long,
        &[
            bx(-0.10, 1.75, 0.035, 0.035, Wood),
            bx(1.75, 1.79, 0.22, 0.03, Main),
            bxo(1.79, 2.05, -0.09, 0.02, 0.02, Main),
            bx(1.79, 2.10, 0.02, 0.02, Main),
            bxo(1.79, 2.05, 0.09, 0.02, 0.02, Main),
        ],
    ),
    ("glaive", Long, &[bx(-0.10, 1.80, 0.035, 0.035, Wood), bxo(1.75, 2.35, 0.03, 0.09, 0.01, Main)]),
    ("short_spear", Long, &[bx(-0.10, 1.35, 0.03, 0.03, Wood), bx(1.35, 1.55, 0.055, 0.012, Main)]),
    ("javelin", Long, &[bx(-0.05, 1.20, 0.025, 0.025, Wood), bx(1.20, 1.38, 0.035, 0.015, Main)]),
    ("throwing_knives", Long, &[bx(0.0, 0.08, 0.02, 0.01, Leather), bx(0.08, 0.22, 0.03, 0.006, Main), bxo(0.0, 0.08, 0.035, 0.02, 0.01, Leather), bxo(0.08, 0.22, 0.035, 0.03, 0.006, Main)]),
    ("net", Long, &[bx(0.0, 0.35, 0.30, 0.12, Main), bx(0.35, 0.45, 0.06, 0.04, Cord)]),
    // --- Ranged ------------------------------------------------------------
    (
        "short_bow",
        Bow,
        &[bxo(-0.55, -0.10, 0.03, 0.025, 0.025, Main), bx(-0.10, 0.10, 0.035, 0.035, Leather), bxo(0.10, 0.55, 0.03, 0.025, 0.025, Main), bxo(-0.55, 0.55, -0.06, 0.005, 0.005, Cord)],
    ),
    ("crossbow", Long, &[bx(-0.30, 0.40, 0.05, 0.05, Wood), bx(0.32, 0.36, 0.60, 0.03, Main), bxo(0.28, 0.30, 0.0, 0.58, 0.006, Cord)]),
    (
        "longbow",
        Bow,
        &[bxo(-0.85, -0.10, 0.03, 0.028, 0.028, Main), bx(-0.10, 0.10, 0.035, 0.035, Leather), bxo(0.10, 0.85, 0.03, 0.028, 0.028, Main), bxo(-0.85, 0.85, -0.05, 0.005, 0.005, Cord)],
    ),
    ("hand_crossbow", Long, &[bx(-0.12, 0.20, 0.04, 0.04, Wood), bx(0.16, 0.19, 0.30, 0.025, Main), bxo(0.14, 0.15, 0.0, 0.28, 0.005, Cord)]),
    ("sling", Long, &[bx(0.0, 0.50, 0.008, 0.008, Cord), bx(0.50, 0.58, 0.06, 0.04, Main)]),
    ("arrows", Long, &[bx(0.0, 0.10, 0.03, 0.004, Paper), bx(0.0, 0.70, 0.01, 0.01, Wood), bx(0.70, 0.76, 0.025, 0.006, Iron)]),
    ("bolts", Long, &[bx(0.0, 0.06, 0.03, 0.004, Paper), bx(0.0, 0.40, 0.014, 0.014, Wood), bx(0.40, 0.45, 0.025, 0.012, Iron)]),
    ("sling_stones", Upright, &[bxo(0.0, 0.04, -0.03, 0.045, 0.04, Rgb([0.52, 0.50, 0.46])), bxo(0.0, 0.035, 0.03, 0.04, 0.04, Rgb([0.46, 0.44, 0.40])), bx(0.03, 0.07, 0.04, 0.04, Rgb([0.56, 0.54, 0.50]))]),
    // --- Body, under and over ----------------------------------------------
    ("cloth_shirt", Upright, SHIRT),
    ("wraps", Upright, &[bx(0.40, 0.85, 0.38, 0.20, Main), bx(0.50, 0.53, 0.39, 0.21, Dark), bx(0.62, 0.65, 0.39, 0.21, Dark), bx(0.74, 0.77, 0.39, 0.21, Dark)]),
    (
        "padded_jacket",
        Upright,
        &[
            bx(0.30, 0.88, 0.44, 0.26, Main),
            bxo(0.50, 0.88, -0.28, 0.12, 0.18, Main),
            bxo(0.50, 0.88, 0.28, 0.12, 0.18, Main),
            bx(0.45, 0.47, 0.45, 0.27, Dark),
            bx(0.60, 0.62, 0.45, 0.27, Dark),
            bx(0.75, 0.77, 0.45, 0.27, Dark),
        ],
    ),
    ("hide_coat", Upright, &[bx(0.20, 0.88, 0.44, 0.26, Main), bxo(0.50, 0.88, -0.28, 0.12, 0.18, Main), bxo(0.50, 0.88, 0.28, 0.12, 0.18, Main), bx(0.84, 0.92, 0.30, 0.28, Dark)]),
    (
        "scale_hauberk",
        Upright,
        &[
            bx(0.25, 0.88, 0.44, 0.26, Main),
            bxo(0.55, 0.88, -0.28, 0.12, 0.18, Main),
            bxo(0.55, 0.88, 0.28, 0.12, 0.18, Main),
            bx(0.32, 0.34, 0.45, 0.27, Dark),
            bx(0.44, 0.46, 0.45, 0.27, Dark),
            bx(0.56, 0.58, 0.45, 0.27, Dark),
            bx(0.68, 0.70, 0.45, 0.27, Dark),
            bx(0.80, 0.82, 0.45, 0.27, Dark),
        ],
    ),
    ("plate_cuirass", Upright, &[bx(0.45, 0.88, 0.42, 0.28, Main), bx(0.47, 0.86, 0.04, 0.30, Light), bx(0.45, 0.48, 0.43, 0.29, Dark)]),
    ("cloak", Upright, &[bx(0.05, 0.90, 0.50, 0.04, Main), bxz(0.85, 0.93, 0.0, 0.36, 0.04, 0.10, Dark)]),
    // --- Legs, head, hands, feet -------------------------------------------
    ("trousers", Upright, LEGS),
    ("hide_leggings", Upright, LEGS),
    (
        "scale_greaves",
        Upright,
        &[
            bx(0.45, 0.55, 0.34, 0.20, Second),
            bxo(0.0, 0.47, -0.09, 0.14, 0.16, Main),
            bxo(0.0, 0.47, 0.09, 0.14, 0.16, Main),
            bx(0.15, 0.17, 0.34, 0.17, Dark),
            bx(0.30, 0.32, 0.34, 0.17, Dark),
        ],
    ),
    ("leather_cap", Upright, &[bx(0.0, 0.10, 0.22, 0.22, Main), bx(0.10, 0.14, 0.16, 0.16, Main)]),
    ("iron_helm", Upright, &[bx(0.0, 0.18, 0.26, 0.26, Main), bx(0.18, 0.22, 0.18, 0.18, Light), bxz(0.0, 0.12, 0.0, 0.03, 0.135, 0.02, Dark)]),
    ("seers_hood", Upright, &[bx(0.0, 0.20, 0.26, 0.26, Rgb([0.36, 0.30, 0.46])), bxz(0.0, 0.2, 0.0, 0.18, 0.12, 0.04, Rgb([0.16, 0.13, 0.20]))]),
    ("leather_gloves", Upright, GLOVES),
    ("duelists_gloves", Upright, GLOVES),
    ("boots", Upright, BOOTS),
    ("striders_boots", Upright, BOOTS),
    // --- Shields -------------------------------------------------------------
    ("buckler", Upright, &[bx(0.0, 0.40, 0.40, 0.03, Main), bxz(0.15, 0.25, 0.0, 0.10, 0.03, 0.04, Iron)]),
    (
        "round_shield",
        Upright,
        &[bx(0.0, 0.70, 0.56, 0.04, Main), bx(0.07, 0.63, 0.70, 0.04, Main), bxz(0.29, 0.41, 0.0, 0.12, 0.035, 0.05, Iron), bx(0.34, 0.36, 0.70, 0.045, Dark)],
    ),
    // --- Packs and gear ------------------------------------------------------
    ("small_pack", Upright, &[bx(0.0, 0.40, 0.32, 0.18, Main), bxz(0.28, 0.42, 0.0, 0.33, 0.01, 0.19, Dark), bxz(0.0, 0.30, 0.0, 0.05, 0.095, 0.01, Leather)]),
    (
        "large_pack",
        Upright,
        &[bx(0.0, 0.55, 0.42, 0.24, Main), bxz(0.40, 0.57, 0.0, 0.43, 0.01, 0.25, Dark), bx(0.57, 0.68, 0.46, 0.12, Rgb([0.55, 0.52, 0.62]))],
    ),
    (
        "porters_belt_pack",
        Upright,
        &[bx(0.0, 0.70, 0.40, 0.22, Main), bxo(0.0, 0.85, -0.22, 0.03, 0.03, Wood), bxo(0.0, 0.85, 0.22, 0.03, 0.03, Wood), bx(0.80, 0.83, 0.47, 0.03, Wood)],
    ),
    ("plain_belt", Upright, &[bx(0.0, 0.05, 0.42, 0.02, Leather), bxz(-0.005, 0.055, 0.0, 0.07, 0.01, 0.02, Iron)]),
    ("bronze_brooch", Upright, &[bx(0.0, 0.05, 0.05, 0.015, Rgb([0.62, 0.45, 0.26])), bxz(0.015, 0.035, 0.0, 0.02, 0.008, 0.01, Gold)]),
    ("gold_ring", Upright, RING),
    ("ring_swiftness", Upright, RING),
    ("ring_might", Upright, RING),
    ("ring_hearth", Upright, RING),
    ("pearl_necklace", Upright, NECKLACE),
    ("amulet_wellspring", Upright, NECKLACE),
    ("amulet_clear_mind", Upright, NECKLACE),
    ("torch", Long, &[bx(0.0, 0.50, 0.04, 0.04, Wood), bx(0.50, 0.62, 0.07, 0.07, Pitch)]),
    ("standing_torch", Long, &[bx(0.0, 1.40, 0.05, 0.05, Wood), bx(1.40, 1.55, 0.09, 0.09, Pitch), bx(0.0, 0.05, 0.03, 0.03, Iron)]),
    ("lockpick", Long, &[bx(0.0, 0.03, 0.012, 0.006, Wood), bx(0.03, 0.12, 0.006, 0.003, Iron), bxo(0.115, 0.125, 0.006, 0.012, 0.003, Iron)]),
    ("tent", Upright, &[bx(0.0, 0.25, 0.55, 0.25, Rgb([0.62, 0.58, 0.48])), bxo(0.0, 0.26, -0.15, 0.03, 0.26, Leather), bxo(0.0, 0.26, 0.15, 0.03, 0.26, Leather), bxo(0.05, 0.12, 0.30, 0.04, 0.04, Wood)]),
    ("bedroll", Upright, &[bx(0.0, 0.20, 0.60, 0.20, Rgb([0.45, 0.42, 0.62])), bxo(0.0, 0.21, -0.18, 0.03, 0.21, Leather), bxo(0.0, 0.21, 0.18, 0.03, 0.21, Leather)]),
    ("mortar_and_pestle", Upright, &[bx(0.0, 0.10, 0.16, 0.16, Rgb([0.50, 0.48, 0.44])), bxo(0.05, 0.24, 0.03, 0.03, 0.03, Wood)]),
    ("hearthclay_pot", Upright, &[bx(0.0, 0.18, 0.20, 0.20, Rgb([0.58, 0.36, 0.24])), bx(0.18, 0.22, 0.12, 0.12, Rgb([0.52, 0.32, 0.21]))]),
    ("sandglass_flask", Upright, &[bx(0.0, 0.14, 0.10, 0.10, Rgb([0.78, 0.70, 0.50])), bx(0.14, 0.20, 0.04, 0.04, Rgb([0.78, 0.70, 0.50])), bx(0.20, 0.22, 0.035, 0.035, Wood)]),
    ("coin", Upright, &[bx(0.0, 0.03, 0.03, 0.005, Gold)]),
    ("note", Upright, &[bx(0.0, 0.10, 0.18, 0.003, Paper), bxz(0.03, 0.07, 0.0, 0.04, 0.003, 0.002, Rgb([0.35, 0.45, 0.30]))]),
    ("sealed_letter", Upright, &[bx(0.0, 0.12, 0.20, 0.006, Paper), bxz(0.04, 0.08, 0.0, 0.04, 0.005, 0.006, Rgb([0.70, 0.15, 0.12]))]),
    ("ink", Upright, &[bx(0.0, 0.07, 0.06, 0.06, Rgb([0.16, 0.15, 0.18])), bx(0.07, 0.09, 0.03, 0.03, Wood)]),
    ("reed_paper", Upright, &[bx(0.0, 0.22, 0.16, 0.012, Paper)]),
];

const POTION: &[Bit] = &[bx(0.0, 0.12, 0.09, 0.09, Own), bx(0.12, 0.17, 0.035, 0.035, Own), bx(0.17, 0.19, 0.03, 0.03, Wood)];
const SCROLL: &[Bit] = &[bx(0.0, 0.02, 0.06, 0.06, Wood), bx(0.02, 0.22, 0.05, 0.05, Paper), bx(0.22, 0.24, 0.06, 0.06, Wood)];
const NOTES: &[Bit] = &[bx(0.0, 0.22, 0.16, 0.015, Paper), bxz(0.05, 0.17, 0.0, 0.11, 0.008, 0.002, Rgb([0.25, 0.22, 0.28]))];
const BOOK: &[Bit] = &[bx(0.0, 0.24, 0.18, 0.05, Rgb([0.36, 0.20, 0.14])), bxz(0.01, 0.23, 0.005, 0.17, 0.0, 0.04, Paper)];
const FOOD: &[Bit] = &[bx(0.0, 0.07, 0.14, 0.10, Own), bxo(0.02, 0.09, 0.03, 0.06, 0.06, Own)];
const COIN: &[Bit] = &[bx(0.0, 0.03, 0.03, 0.005, Gold)];
const LETTER: &[Bit] = &[bx(0.0, 0.12, 0.20, 0.006, Paper)];
const LUMP: &[Bit] = &[bx(0.0, 0.05, 0.05, 0.05, Own)];
const POLEARM: &[Bit] = &[bx(-0.1, 1.5, 0.04, 0.04, Wood), bx(1.5, 1.8, 0.08, 0.02, Main)];
const BLADE: &[Bit] = &[bx(-0.1, 0.0, 0.03, 0.03, Leather), bx(0.0, 0.6, 0.05, 0.01, Main)];
const SHIELD: &[Bit] = &[bx(0.0, 0.5, 0.5, 0.04, Main)];
const PACK: &[Bit] = &[bx(0.0, 0.45, 0.35, 0.2, Main)];
const HAT: &[Bit] = &[bx(0.0, 0.14, 0.24, 0.24, Main)];
const CLOAK: &[Bit] = &[bx(0.05, 0.90, 0.50, 0.04, Main)];
const SMALL: &[Bit] = &[bx(0.0, 0.1, 0.1, 0.05, Main)];
const TRINKET: &[Bit] = &[bx(0.0, 0.08, 0.08, 0.06, Own)];
const TORCH: &[Bit] = &[bx(0.0, 0.5, 0.04, 0.04, Wood), bx(0.5, 0.62, 0.07, 0.07, Pitch)];
const TOOL: &[Bit] = &[bx(0.0, 0.12, 0.16, 0.10, Own)];
const STUFF: &[Bit] = &[bx(0.0, 0.10, 0.16, 0.12, Own), bxo(0.0, 0.07, 0.10, 0.08, 0.10, Own)];

/// A shape by kind, for things without one of their own.
fn fallback(id: ItemId) -> (Pose, &'static [Bit]) {
    let d = item(id);
    match d.kind {
        Kind::Potion => (Upright, POTION),
        Kind::Scroll(_) => (Long, SCROLL),
        Kind::Notes(_) => (Upright, NOTES),
        Kind::Text(_) | Kind::Manual(_) => (Upright, BOOK),
        Kind::Food(_) => (Upright, FOOD),
        Kind::Coin => (Upright, COIN),
        Kind::Errand => (Upright, LETTER),
        Kind::Ammo => (Upright, LUMP),
        Kind::Weapon(w) => (Long, if w.two_handed { POLEARM } else { BLADE }),
        Kind::Shield(_) => (Upright, SHIELD),
        Kind::Pack(_) => (Upright, PACK),
        Kind::Armor(_) => match d.slot {
            Slot::BodyUnder | Slot::BodyOver => (Upright, SHIRT),
            Slot::LegsUnder | Slot::LegsOver => (Upright, LEGS),
            Slot::Head => (Upright, HAT),
            Slot::Hands => (Upright, GLOVES),
            Slot::Feet => (Upright, BOOTS),
            Slot::Cloak => (Upright, CLOAK),
            _ => (Upright, SMALL),
        },
        Kind::Trinket => match d.slot {
            Slot::Ring | Slot::Ring2 => (Upright, RING),
            Slot::Neck => (Upright, NECKLACE),
            _ => (Upright, TRINKET),
        },
        Kind::Torch(_) | Kind::StandingTorch(_) => (Long, TORCH),
        Kind::Tool => (Upright, TOOL),
        Kind::Material => (Upright, STUFF),
    }
}

/// A thing's shape: its form's, or one by its kind.
pub fn shape(id: ItemId) -> (Pose, &'static [Bit]) {
    let key = item(info(id).form).key;
    SHAPES.iter().find(|s| s.0 == key).map(|s| (s.1, s.2)).unwrap_or_else(|| fallback(id))
}

/// How long a shape is along itself, metres (for holding it at the right
/// distance).
pub fn length(id: ItemId) -> f32 {
    shape(id).1.iter().map(|b| b.at[0] + b.half[0]).fold(0.0, f32::max)
}

/// Raw materials and food by their own colours (the rest by kind).
fn own_colour(id: ItemId) -> Rgb {
    match item(id).key {
        "iron_ore" | "rock" => [0.42, 0.38, 0.35],
        "iron_ingot" | "salvage" => [0.55, 0.57, 0.60],
        "bronze_ingot" => [0.62, 0.45, 0.26],
        "gold_nugget" => [0.86, 0.68, 0.30],
        "timber" => [0.48, 0.35, 0.22],
        "charcoal" => [0.15, 0.14, 0.13],
        "hide" | "light_hide" | "thick_hide" | "tough_skin" => [0.55, 0.42, 0.30],
        "leather" => [0.42, 0.28, 0.17],
        "cloth" | "fibre" | "tentsilk" => [0.72, 0.68, 0.58],
        "clay" | "hearthclay" => [0.58, 0.40, 0.28],
        "sand" => [0.80, 0.72, 0.55],
        "ash" => [0.62, 0.62, 0.60],
        "seareed" | "kelp_frond" => [0.36, 0.48, 0.30],
        "pearl" | "nacre" | "shell" => [0.85, 0.83, 0.78],
        "ghostcap" => [0.85, 0.85, 0.80],
        "emberroot" => [0.85, 0.42, 0.18],
        "bloodleaf" => [0.42, 0.20, 0.18],
        "salt_crystal" => [0.92, 0.92, 0.90],
        "storm_glass" | "edgeglass" | "edge_seed" => [0.62, 0.72, 0.78],
        "ringstone" | "slatewing" => [0.40, 0.40, 0.42],
        "wild_berries" => [0.30, 0.25, 0.50],
        "flatbread" | "grain" => [0.80, 0.66, 0.42],
        "dried_fish" | "mussels" => [0.62, 0.55, 0.42],
        "salted_meat" | "raw_meat" => [0.62, 0.30, 0.26],
        "egg" => [0.90, 0.86, 0.76],
        "pitch" => [0.15, 0.12, 0.10],
        _ => super::squadui::ground_color(id),
    }
}

fn scale(c: Rgb, k: f32) -> Rgb {
    [(c[0] * k).min(1.0), (c[1] * k).min(1.0), (c[2] * k).min(1.0)]
}

/// The colour of a box of this thing.
pub fn colour(id: ItemId, t: Tint) -> Rgb {
    const UNDYED: Rgb = [0.64, 0.58, 0.47];
    let main = || super::dress::material_colour(info(id).main).unwrap_or(if info(id).main == Material::None { own_colour(id) } else { UNDYED });
    match t {
        Main => main(),
        Dark => scale(main(), 0.7),
        Light => scale(main(), 1.25),
        Second => super::dress::material_colour(info(id).second).unwrap_or_else(main),
        Wood => [0.45, 0.33, 0.21],
        Leather => [0.40, 0.27, 0.16],
        Iron => [0.55, 0.57, 0.60],
        Paper => [0.88, 0.84, 0.70],
        Gold => [0.86, 0.68, 0.30],
        Cord => [0.82, 0.78, 0.66],
        Pitch => [0.18, 0.13, 0.10],
        Own => own_colour(id),
        Rgb(c) => c,
    }
}

/// Draw a thing in the 3D view: `grip` where its frame starts, `along` and
/// `across` its directions (unit, at right angles), `k` how much bigger
/// than life (things far from the camera are drawn up, like people).
pub fn draw(b: &mut Builder, id: ItemId, grip: Vec3, along: Vec3, across: Vec3, k: f32, lift: f32) {
    let out = along.cross(across);
    for bit in shape(id).1 {
        let c = grip + (along * bit.at[0] + across * bit.at[1] + out * bit.at[2]) * k;
        let h = [bit.half[0].max(0.004) * k, bit.half[1].max(0.004) * k, bit.half[2].max(0.004) * k];
        b.cuboid(c, along * h[0], across * h[1], out * h[2], scale(colour(id, bit.tint), lift));
    }
}

/// Draw a thing lying on the ground at `at`, turned `rot` about the
/// vertical: flat on its back, whatever it is.
pub fn lying(b: &mut Builder, id: ItemId, at: Vec3, rot: f32, k: f32) {
    let along = vec3(rot.cos(), 0.0, rot.sin());
    let across = vec3(-rot.sin(), 0.0, rot.cos());
    // Thickness points up (along × across), so it lies on its back; lift it
    // by its own thickness so it rests on the ground rather than in it.
    let up = shape(id).1.iter().map(|b| b.half[2] - b.at[2]).fold(0.0, f32::max);
    let centre = shape(id).1.iter().map(|b| b.at[0]).sum::<f32>() / shape(id).1.len().max(1) as f32;
    let grip = at - along * (centre * k) + Vec3::Y * (up * k);
    let flip = if along.cross(across).y < 0.0 { -across } else { across };
    draw(b, id, grip, along, flip, k, 1.0);
}

/// The icon of a thing, drawn into a box on a panel: its shape seen from a
/// little above and to one side, shaded by a light over the left shoulder,
/// fitted to the box.
pub fn icon(c: &Canvas, id: ItemId, x: f32, y: f32, w: f32, h: f32) {
    let (pose, bits) = shape(id);
    let s2 = std::f32::consts::FRAC_1_SQRT_2;
    let (along, across) = match pose {
        Long => (vec3(s2, s2, 0.0), vec3(-s2, s2, 0.0)),
        Bow => (vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)),
        Upright => (vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)),
    };
    let out = along.cross(across);
    // Turned a little to show a side, and tipped to show the top.
    let (yaw, pitch) = if pose == Long { (-0.35f32, 0.3f32) } else { (-0.55f32, 0.4f32) };
    let view = |v: Vec3| {
        let (sy, cy) = yaw.sin_cos();
        let v = vec3(v.x * cy + v.z * sy, v.y, -v.x * sy + v.z * cy);
        let (sp, cp) = pitch.sin_cos();
        vec3(v.x, v.y * cp - v.z * sp, v.y * sp + v.z * cp)
    };
    let light = vec3(-0.35, 0.75, 0.55).normalize();
    let mut faces: Vec<(f32, [Vec3; 4], Rgb)> = Vec::new();
    for bit in bits {
        let ctr = along * bit.at[0] + across * bit.at[1] + out * bit.at[2];
        let ax = [along * bit.half[0].max(0.003), across * bit.half[1].max(0.003), out * bit.half[2].max(0.003)];
        let col = colour(id, bit.tint);
        for (a, u, v) in [(ax[0], ax[1], ax[2]), (ax[1], ax[2], ax[0]), (ax[2], ax[0], ax[1])] {
            for s in [-1.0f32, 1.0] {
                let n = view((a * s).normalize_or_zero());
                if n.z <= 0.0 {
                    continue;
                }
                let f = ctr + a * s;
                let pts = [view(f - u - v), view(f + u - v), view(f + u + v), view(f - u + v)];
                let depth = pts.iter().map(|p| p.z).sum::<f32>() * 0.25;
                faces.push((depth, pts, scale(col, 0.55 + 0.5 * n.dot(light).max(0.0))));
            }
        }
    }
    if faces.is_empty() {
        return;
    }
    let (mut lo, mut hi) = (bevy::math::Vec2::splat(f32::MAX), bevy::math::Vec2::splat(f32::MIN));
    for f in &faces {
        for p in f.1 {
            lo = lo.min(bevy::math::Vec2::new(p.x, p.y));
            hi = hi.max(bevy::math::Vec2::new(p.x, p.y));
        }
    }
    let span = (hi - lo).max(bevy::math::Vec2::splat(1e-4));
    let k = (w / span.x).min(h / span.y) * 0.86;
    let mid = (lo + hi) * 0.5;
    let at = |p: Vec3| egui::pos2(x + w * 0.5 + (p.x - mid.x) * k, y + h * 0.5 - (p.y - mid.y) * k);
    faces.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (_, pts, col) in faces {
        let to = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
        c.p.add(egui::Shape::convex_polygon(pts.iter().map(|&p| at(p)).collect(), Color32::from_rgb(to(col[0]), to(col[1]), to(col[2])), egui::Stroke::NONE));
    }
}

/// The main-hand thing someone holds, if any.
fn in_hand(w: &World, pid: PersonId) -> Option<ItemId> {
    w.people[pid as usize].detail.as_ref().and_then(|d| d.gear.in_slot(Slot::MainHand))
}

/// Draw the weapon in someone's hand in a fight, pointing along `dir` from
/// `hand` (a bow upright before them, `dir` towards the mark). False if
/// they hold nothing (the caller draws its plain stick).
pub fn wield(b: &mut Builder, w: &World, pid: PersonId, hand: Vec3, dir: Vec3, k: f32) -> bool {
    let Some(id) = in_hand(w, pid) else { return false };
    let dir = dir.normalize_or(Vec3::X);
    let flat = if dir.y.abs() > 0.95 { Vec3::X } else { Vec3::Y };
    match shape(id).0 {
        Bow => {
            let up = (Vec3::Y - dir * dir.y).normalize_or(Vec3::Y);
            draw(b, id, hand + dir * (0.2 * k), up, dir, k, 1.0);
        }
        _ => draw(b, id, hand, dir, dir.cross(flat).normalize_or(Vec3::Z), k, 1.0),
    }
    true
}

/// What's worn over the figure's parts (`dress.rs` colours the parts
/// themselves): a cloak down the back, a belt, a brooch, the shield on the
/// arm, and, out of a fight, the weapon carried: a short one at the hip, a
/// spear or staff upright in the hand, anything else long or a bow across
/// the back. `base` at their feet, `fwd` and
/// `side` (their right) the way they face, `waist` and `body` heights.
#[allow(clippy::too_many_arguments)]
pub fn on_figure(b: &mut Builder, w: &World, pid: PersonId, base: Vec3, fwd: Vec3, side: Vec3, waist: f32, body: f32, r: f32, k: f32, lift: f32, fighting: bool) {
    let Some(gear) = w.people[pid as usize].detail.as_ref().map(|d| &d.gear) else { return };
    let up = Vec3::Y;
    let pack = gear.in_slot(Slot::Back).is_some();
    if let Some(id) = gear.in_slot(Slot::Cloak) {
        // From the shoulders nearly to the ground, over the pack if one's worn.
        let (top, bottom) = (body - r * 0.15, waist * 0.2);
        let back = r * (if pack { 1.75 } else { 1.0 });
        let c = base + up * ((top + bottom) * 0.5) - fwd * back;
        b.cuboid(c, up * ((top - bottom) * 0.5), side * (r * 1.15), fwd * (r * 0.08), scale(colour(id, Main), lift));
    }
    if let Some(id) = gear.in_slot(Slot::Belt) {
        b.column(base + up * (waist - body * 0.03), r * 1.02, r * 1.0, body * 0.06, 6, scale(colour(id, Leather), lift));
    }
    if let Some(id) = gear.in_slot(Slot::Brooch) {
        let at = base + up * (body - r * 0.55) + fwd * (r * 0.95) - side * (r * 0.4);
        b.cuboid(at, up * (r * 0.12), side * (r * 0.12), fwd * (r * 0.06), scale(colour(id, Light), lift));
    }
    if let Some(id) = gear.in_slot(Slot::OffHand).filter(|&i| matches!(item(i).kind, Kind::Shield(_))) {
        // On the left arm, its face out.
        let at = base + up * (waist * 0.35) - side * (r * 1.25);
        draw(b, id, at, up, fwd, k, lift);
    }
    if fighting {
        return;
    }
    let Some(id) = gear.in_slot(Slot::MainHand) else { return };
    let (pose, _) = shape(id);
    if pose == Long && length(id) > 1.4 {
        // A spear or a staff: held upright in the right hand like a walking
        // pole, its butt on the ground.
        let butt = shape(id).1.iter().map(|b| b.at[0] - b.half[0]).fold(0.0, f32::min);
        let grip = base + up * (0.04 - butt * k) + side * (r * 1.3);
        draw(b, id, grip, up, fwd, k, lift);
    } else if pose == Bow || length(id) > 1.0 {
        // Across the back, from the right hip to over the left shoulder.
        let along = (up * 1.0 - side * 0.7).normalize();
        let grip = base + up * (waist * 0.55) - fwd * (r * (if pack { 1.6 } else { 1.05 })) + side * (r * 0.7);
        draw(b, id, grip, along, fwd, k, lift);
    } else {
        // At the left hip, hanging point down.
        let along = (-up * 0.95 - fwd * 0.3).normalize();
        let grip = base + up * (waist + body * 0.02) - side * (r * 1.05);
        draw(b, id, grip, along, -side, k, lift);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_thing_has_a_shape_of_its_own_size() {
        for id in 0..gahturiyu_sim::sim::items::catalogue().defs.len() as ItemId {
            let (_, bits) = shape(id);
            assert!(!bits.is_empty(), "{} has no shape", item(id).name);
            for b in bits {
                assert!(b.half.iter().all(|&h| h >= 0.0) && b.half.iter().any(|&h| h > 0.0), "{}: an empty box", item(id).name);
            }
            assert!(length(id) < 3.0, "{} is {} m long", item(id).name, length(id));
        }
    }
}
