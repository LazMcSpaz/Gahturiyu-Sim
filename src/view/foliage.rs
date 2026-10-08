//! Grass, bushes and trees (decoration only). Filled in later.

use bevy::prelude::*;
use bevy_egui::egui;

use gahturiyu_sim::sim::{geo::V2, World};

use super::cam::OrbitCam;
use super::hud::Canvas;
use super::models::Models;
use super::scene::Scene3d;

#[derive(Resource, Default)]
pub struct Foliage {}

pub fn setup() {}
pub fn update() {}

pub fn biggest_wood_near(_w: &World, _p: V2) -> Option<V2> {
    None
}

pub fn draw_readout(c: &Canvas, oc: &OrbitCam, scene: &Scene3d, _f: &Foliage, models: &Models) -> egui::Rect {
    let lines = vec![
        (format!("Camera {:.0} m out", oc.dist), super::palette::TEXT),
        (format!("Scene triangles {}", scene.triangles), super::palette::DIM),
        (models.status.clone(), super::palette::DIM),
    ];
    c.panel(&lines, c.w - 420.0, c.h - 300.0, 14.0)
}
