//! Places built in the place builder (`placefile.rs`, `docs/place-builder.md`)
//! become towns: the buildings, workplaces and pens as placed, the ground,
//! height, rocks and trees laid under the town, and the capacity and roles
//! the builder shows are what the game makes of it.
//!
//! The place here is a THROWAWAY test fixture, not anyone's town.

use gahturiyu_sim::sim::{
    atlas,
    geo::V2,
    jobs::PlaceKind,
    placefile::{height_of, PlaceFile},
    plan::Plans,
    roster::Rosters,
    terrain::Ground,
    worldgen, World,
};

const KEY: &str = "imperial_village";

/// A throwaway place: 40 × 32 m of dirt, two metres up, a cottage, a
/// quarters block, a smithy, a turiyu pen, a boulder, a tree, a bush and a
/// big rock, a road, and two things the game can't stand up (a cart, a
/// squad hut).
fn fixture() -> PlaceFile {
    let (w, h) = (20u32, 16u32);
    let n = (w * h) as usize;
    let text = format!(
        r#"{{
        "format": 1, "key": "{KEY}", "title": "THROWAWAY test place", "notes": "fixture for tests/places.rs only",
        "cell": 2.0, "size": [{w}, {h}], "origin": [10, 8],
        "ground": "{ground}", "height": "{height}",
        "foliage": [
            {{ "k": "tree_broad", "x": 14.0, "y": -12.0, "s": 1.0 }},
            {{ "k": "bush", "x": -15.0, "y": 10.0 }},
            {{ "k": "rock_large", "x": 16.0, "y": 12.0, "s": 1.2 }}
        ],
        "roads": [ {{ "k": "road", "w": 3.0, "pts": [[0, 0], [18, 0]] }} ],
        "things": [
            {{ "id": "t1", "k": "roduro_cottage", "x": -8.0, "y": -6.0, "rot": 90, "s": 1.0, "label": "A cottage" }},
            {{ "id": "t2", "k": "qotiro_quarters", "x": 8.0, "y": -6.0, "rot": 180, "room": 10 }},
            {{ "id": "t3", "k": "smithy", "x": 0.0, "y": 10.0, "rot": 270 }},
            {{ "id": "t4", "k": "pen_turiyu", "x": -14.0, "y": 8.0 }},
            {{ "id": "t5", "k": "rock_boulder", "x": -16.0, "y": -12.0 }},
            {{ "id": "t6", "k": "cart", "x": 5.0, "y": 5.0, "label": "The cart" }},
            {{ "id": "t7", "k": "hut", "x": 12.0, "y": 12.0 }}
        ],
        "saved": "2026-10-14T12:00:00Z"
    }}"#,
        ground = "d".repeat(n),
        height = "c".repeat(n)
    );
    PlaceFile::from_json(&text).expect("the fixture reads")
}

fn town_of(key: &str) -> usize {
    atlas::town_marks().iter().position(|m| m.key() == key).expect("the mark")
}

fn built() -> (World, World) {
    let mut plans = Plans::default();
    plans.towns.insert(KEY.into(), fixture().to_plan());
    (worldgen::generate_with_plans(1, &Rosters::default(), &plans), worldgen::generate_with_plans(1, &Rosters::default(), &Plans::default()))
}

#[test]
fn a_place_file_reads_back_as_written() {
    let f = fixture();
    let back = PlaceFile::from_json(&f.to_json()).unwrap();
    assert_eq!(back, f);
    assert_eq!(height_of('A'), Some(-26.0));
    assert_eq!(height_of('Z'), Some(-1.0));
    assert_eq!(height_of('a'), Some(0.0));
    assert_eq!(height_of('z'), Some(25.0));
    assert_eq!(height_of('.'), None);
}

#[test]
fn the_plan_has_what_was_placed_and_says_what_it_cannot_stand_up() {
    let f = fixture();
    let plan = f.to_plan();
    assert_eq!(plan.buildings.len(), 2);
    assert_eq!(plan.buildings[0].variant.as_deref(), Some("roduro_cottage"));
    assert_eq!(plan.buildings[0].facing, 90.0);
    assert_eq!(plan.buildings[1].room_for, 10, "the room the builder gave");
    assert_eq!(plan.places.len(), 1);
    assert_eq!(plan.places[0].kind, PlaceKind::Forge);
    assert_eq!(plan.pens.len(), 1);
    assert_eq!(plan.pens[0].species, "turiyu");
    let land = plan.land.as_ref().unwrap();
    assert_eq!(land.plants.len(), 2, "a tree and a bush");
    assert_eq!(land.rocks.len(), 2, "the stamped rock and the placed boulder");
    assert!(plan.later.iter().any(|l| l.contains("The cart")), "{:?}", plan.later);
    assert!(plan.later.iter().any(|l| l.contains("hut")), "{:?}", plan.later);
    // Capacity and roles, as the builder shows them.
    assert_eq!(f.capacity(), 1 + 10);
    assert_eq!(f.capacity() as usize, plan.room());
    assert!(f.roles().contains(&"Smith".to_string()), "{:?}", f.roles());
}

#[test]
fn a_built_place_becomes_the_town() {
    let (w, plain) = built();
    let t = town_of(KEY);
    let s = &w.settlements[t];
    let c = s.pos;
    // The hearth, then the two buildings as placed, their variants the ones picked.
    assert_eq!(s.buildings.len(), 3);
    assert!(s.buildings[1].pos.dist(c.add(V2::new(-8.0, -6.0))) < 0.01);
    assert!((s.buildings[1].rot - 90f32.to_radians()).abs() < 1e-4);
    assert_eq!(s.styles.get(1).copied().flatten(), Some("roduro_cottage"));
    assert_eq!(s.styles.get(2).copied().flatten(), Some("qotiro_quarters"));
    // The smithy where it was put.
    assert!(w.society.towns[t].places.iter().any(|p| p.kind == PlaceKind::Forge && p.pos.dist(c.add(V2::new(0.0, 10.0))) < 0.5), "the smithy");
    // The pen.
    assert!(w.animals.pens.iter().any(|p| p.town == Some(t as u16) && p.home.dist(c.add(V2::new(-14.0, 8.0))) < 0.5), "the pen");
    // The ground painted, and lifted two metres over the land at the centre.
    assert_eq!(w.terrain.ground(c.add(V2::new(-12.0, -14.0))), Ground::Dirt);
    let lift = w.terrain.height(c) - plain.terrain.height(c);
    assert!((lift - 2.0).abs() < 0.6, "lifted {lift} m");
    // Rocks of the land, two more than the plain world.
    assert_eq!(w.terrain.authored.rocks.len(), plain.terrain.authored.rocks.len() + 2);
    // Only the stamped trees grow in it.
    assert_eq!(w.plan_woods_at(c.add(V2::new(5.0, -5.0))), Some(0.0));
    let stamps = w.place_stamps_in(c.x - 30.0, c.y - 30.0, c.x + 30.0, c.y + 30.0);
    assert_eq!(stamps.len(), 2);
}

#[test]
fn a_built_place_saves_and_loads() {
    let (w, _) = built();
    let back = World::load_bytes(&w.save_bytes()).expect("loads");
    let t = town_of(KEY);
    assert_eq!(back.plan_of(t as u16), w.plan_of(t as u16));
    assert_eq!(back.terrain.height(w.settlements[t].pos), w.terrain.height(w.settlements[t].pos));
}
