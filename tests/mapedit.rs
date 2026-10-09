//! Editing the land by hand: brushes, undo, map files, saves.

use gahturiyu_sim::sim::{
    geo::V2,
    mapedit::{Brush, Dab, MapEdits, TEXTURES},
    terrain::Ground,
    world::HOUR,
    worldgen, World,
};

fn dab(brush: Brush, at: V2, radius: f32, secs: f32) -> Dab {
    Dab { brush, at, radius, strength: 1.0, falloff: 0.5, dt: secs, target: 0.0 }
}

/// Somewhere inland, out of town.
fn spot(w: &World) -> V2 {
    let p = w.settlements[0].pos.add(V2::new(400.0, 300.0));
    assert!(gahturiyu_sim::sim::geo::is_land(p));
    p
}

#[test]
fn raise_lower_smooth_flatten_and_restore() {
    let mut w = worldgen::generate(1);
    let p = spot(&w);
    let h0 = w.terrain.height(p);
    w.terrain.edits.begin_stroke();
    w.terrain.dab(dab(Brush::Raise, p, 40.0, 1.0));
    w.terrain.edits.end_stroke();
    let h1 = w.terrain.height(p);
    assert!(h1 > h0 + 8.0, "raised: {h0} → {h1}");
    // The edge of the brush moves least.
    assert!(w.terrain.height(p.add(V2::new(35.0, 0.0))) - w.terrain.base_height(p.add(V2::new(35.0, 0.0))) < h1 - h0);
    // Smoothing a spike brings its top down.
    w.terrain.dab(dab(Brush::Raise, p, 6.0, 2.0));
    let spike = w.terrain.height(p);
    for _ in 0..30 {
        w.terrain.dab(dab(Brush::Smooth, p, 20.0, 0.2));
    }
    assert!(w.terrain.height(p) < spike - 2.0, "smoothed");
    // Flatten to a height.
    let mut d = dab(Brush::Flatten, p, 30.0, 0.1);
    d.target = 50.0;
    for _ in 0..60 {
        w.terrain.dab(d);
    }
    assert!((w.terrain.height(p) - 50.0).abs() < 1.0, "flat at 50: {}", w.terrain.height(p));
    // Restore takes it back to the seed's land.
    for _ in 0..80 {
        w.terrain.dab(dab(Brush::Restore, p, 60.0, 0.1));
    }
    assert!((w.terrain.height(p) - h0).abs() < 0.5);
}

#[test]
fn a_stroke_undoes_and_redoes_as_one() {
    let mut w = worldgen::generate(1);
    let p = spot(&w);
    let h0 = w.terrain.height(p);
    w.terrain.edits.begin_stroke();
    for k in 0..10 {
        w.terrain.dab(dab(Brush::Raise, p.add(V2::new(k as f32 * 5.0, 0.0)), 30.0, 0.1));
    }
    w.terrain.edits.end_stroke();
    let h1 = w.terrain.height(p);
    assert!(h1 > h0 + 1.0);
    assert!(w.terrain.edits.undo());
    assert_eq!(w.terrain.height(p), h0);
    assert!(w.terrain.edits.is_empty());
    assert!(w.terrain.edits.redo());
    assert_eq!(w.terrain.height(p), h1);
}

#[test]
fn painted_ground_counts_underfoot() {
    let mut w = worldgen::generate(1);
    let p = spot(&w);
    let snow = TEXTURES.iter().position(|t| t.name == "Snow").unwrap() as u8;
    for _ in 0..20 {
        w.terrain.dab(dab(Brush::Paint(snow), p, 20.0, 0.1));
    }
    assert_eq!(w.terrain.ground(p), Ground::Snow);
    assert!(w.terrain.ground(p).pace() < 0.8, "snow is slow going");
    for _ in 0..40 {
        w.terrain.dab(dab(Brush::Unpaint, p, 20.0, 0.1));
    }
    assert_ne!(w.terrain.ground(p), Ground::Snow);
}

#[test]
fn plants_and_rocks_brush() {
    let mut w = worldgen::generate(1);
    let p = spot(&w);
    for _ in 0..10 {
        w.terrain.dab(dab(Brush::Plants { kind: 2, less: false }, p, 30.0, 0.1));
        w.terrain.dab(dab(Brush::Plants { kind: 0, less: true }, p, 30.0, 0.1));
    }
    let f = w.terrain.edits.plants_at(p);
    assert!(f[2].1 > 0.5, "more trees: {f:?}");
    assert!(f[0].0 < 0.2, "less grass: {f:?}");
    for _ in 0..20 {
        w.terrain.dab(dab(Brush::Rocks(0), p, 25.0, 0.2));
    }
    let n = w.terrain.edits.rocks.len();
    assert!(n > 3, "rocks placed: {n}");
    assert!(w.terrain.edits.rocks.iter().all(|r| r.pos.dist(p) <= 25.0));
    for _ in 0..40 {
        w.terrain.dab(dab(Brush::ClearRocks, p, 30.0, 0.2));
    }
    assert!(w.terrain.edits.rocks.len() < n);
}

#[test]
fn a_map_file_starts_new_games_and_saves_carry_the_edits() {
    let mut w = worldgen::generate(1);
    let p = spot(&w);
    for _ in 0..20 {
        w.terrain.dab(dab(Brush::Raise, p, 50.0, 0.2));
    }
    w.terrain.dab(dab(Brush::Rocks(4), p, 40.0, 2.0));
    let lifted = w.terrain.height(p);
    // A map file.
    let bytes = w.terrain.edits.to_bytes(w.seed);
    let (seed, edits) = MapEdits::from_bytes(&bytes).unwrap();
    assert_eq!(seed, 1);
    let fresh = worldgen::generate_with(seed, edits);
    assert_eq!(fresh.terrain.height(p), lifted);
    assert_eq!(fresh.terrain.edits.rocks, w.terrain.edits.rocks);
    // The towns stand where they stood.
    for (a, b) in fresh.settlements.iter().zip(&worldgen::generate(1).settlements) {
        assert_eq!(a.pos, b.pos);
    }
    // A game save keeps them too.
    w.refit_land();
    let back = World::load_bytes(&w.save_bytes()).unwrap();
    assert_eq!(back.terrain.height(p), lifted);
    assert_eq!(back.routes.roads, w.routes.roads);
}

#[test]
fn no_edits_change_nothing() {
    // An unedited land is the seed's land exactly.
    let w = worldgen::generate(1);
    for k in 0..200 {
        let p = V2::new(300.0 + k as f32 * 97.3, 500.0 + k as f32 * 83.1);
        assert_eq!(w.terrain.height(p), w.terrain.base_height(p));
    }
    let mut a = worldgen::generate(1);
    let mut b = worldgen::generate_with(1, MapEdits::default());
    for _ in 0..12 {
        a.step(HOUR);
        b.step(HOUR);
    }
    assert_eq!(a.society.households, b.society.households);
}
