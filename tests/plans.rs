//! Town plans (`sim/plan.rs`, `docs/town-plans.md`): a town with a plan is
//! laid out as drawn (its buildings where the plan puts them, facing as it
//! says, the plan's variants), its drawn workplaces stand where they're
//! drawn and what else it needs is added, its pens are the plan's, and all
//! of it comes back from a save. The plan here is a throwaway for the
//! tests; Laz's real one is checked separately.

use gahturiyu_sim::sim::{
    atlas,
    jobs::PlaceKind,
    plan::{Plan, PlanBuilding, PlanPen, PlanPlace, Plans},
    roster::Rosters,
    settlement::BuildingKind,
    worldgen, World,
};

fn town_index(key: &str) -> usize {
    atlas::town_marks().iter().position(|m| m.key() == key).expect("a marked town")
}

const KEY: &str = "imperial_village";

fn test_plan() -> Plan {
    Plan {
        town: KEY.into(),
        title: "Test plan".into(),
        buildings: vec![
            PlanBuilding { label: "North house".into(), kind: BuildingKind::QotiroBlock, variant: Some("qotiro_court".into()), at: (0.0, -40.0), facing: 90.0, size: 15.0, room_for: 5, wants: String::new() },
            PlanBuilding { label: "East hall".into(), kind: BuildingKind::QotiroHall, variant: None, at: (45.0, 0.0), facing: 180.0, size: 14.0, room_for: 0, wants: String::new() },
            PlanBuilding { label: "South house".into(), kind: BuildingKind::QotiroBlock, variant: None, at: (0.0, 40.0), facing: 270.0, size: 16.0, room_for: 12, wants: String::new() },
        ],
        places: vec![
            PlanPlace { label: "The hall".into(), kind: PlaceKind::Hall, at: (45.0, 0.0), facing: None, wants: String::new() },
            PlanPlace { label: "The smithy".into(), kind: PlaceKind::Forge, at: (-30.0, 20.0), facing: Some(0.0), wants: String::new() },
        ],
        pens: vec![PlanPen { label: "The pen".into(), species: "turiyu".into(), at: (70.0, 30.0), limit: 6.0 }],
        ..Default::default()
    }
}

fn with_plan() -> World {
    let mut plans = Plans::default();
    plans.towns.insert(KEY.into(), test_plan());
    worldgen::generate_with_plans(1, &Rosters::default(), &plans)
}

#[test]
fn a_planned_town_is_laid_out_as_drawn() {
    let w = with_plan();
    let t = town_index(KEY);
    let s = &w.settlements[t];
    assert_eq!(s.buildings.len(), 4, "the hearth and the three drawn");
    assert_eq!(s.buildings[0].kind, BuildingKind::Hearth);
    let b = &s.buildings[1];
    assert_eq!(b.kind, BuildingKind::QotiroBlock);
    assert!((b.pos.x - s.pos.x).abs() < 0.01 && (b.pos.y - (s.pos.y - 40.0)).abs() < 0.01, "where the plan put it");
    assert!((b.rot - std::f32::consts::FRAC_PI_2).abs() < 1e-4, "facing south");
    assert_eq!(b.size, 15.0);
    assert_eq!(s.buildings[2].kind, BuildingKind::QotiroHall);
    assert!((s.buildings[3].rot - 270f32.to_radians()).abs() < 1e-4);
    // The plan's variant where it names one; the game's choice where it doesn't.
    assert_eq!(s.styles[1], Some("qotiro_court"));
    assert!(s.styles[2].is_some());
    assert!(s.styles[3].is_some());
    assert!(s.reach >= 40.0 + 16.0);
    assert_eq!(w.plan_of(t as u16).map(|p| p.title.as_str()), Some("Test plan"));
}

#[test]
fn residents_fill_the_plans_homes_in_order() {
    let w = with_plan();
    let t = town_index(KEY);
    let s = &w.settlements[t];
    assert!(!s.residents.is_empty());
    let homes: Vec<u16> = s.residents.iter().map(|&p| w.people[p as usize].dwelling.unwrap()).collect();
    // Nobody in the hearth or the hall; the first five in the north house.
    assert!(homes.iter().all(|&h| h == 1 || h == 3), "{homes:?}");
    assert!(homes.iter().take(5).all(|&h| h == 1), "{homes:?}");
    assert_eq!(homes.get(5).copied(), Some(3));
}

#[test]
fn drawn_workplaces_stand_where_drawn_and_the_rest_are_added() {
    let w = with_plan();
    let t = town_index(KEY);
    let s = &w.settlements[t];
    let places = &w.society.towns[t].places;
    let hall = places.iter().find(|p| p.kind == PlaceKind::Hall).expect("the hall");
    assert!(hall.pos.dist(s.pos.add(gahturiyu_sim::sim::geo::V2::new(45.0, 0.0))) < 0.01);
    let forge = places.iter().find(|p| p.kind == PlaceKind::Forge).expect("the smithy");
    assert!((forge.rot - 0.0).abs() < 1e-4, "facing as the plan says");
    assert_eq!(places.iter().filter(|p| p.kind == PlaceKind::Hall).count(), 1, "the drawn hall stands for the one the town wants");
    // The town still has what every town has.
    assert!(places.iter().any(|p| p.kind == PlaceKind::Shrine));
    assert!(places.iter().any(|p| p.kind == PlaceKind::Fields));
}

#[test]
fn the_pens_are_the_plans() {
    let w = with_plan();
    let t = town_index(KEY) as u16;
    let pens: Vec<_> = w.animals.pens.iter().filter(|p| p.town == Some(t)).collect();
    assert_eq!(pens.len(), 1, "only the drawn pen");
    assert!(pens[0].home.dist(w.settlements[t as usize].pos.add(gahturiyu_sim::sim::geo::V2::new(70.0, 30.0))) < 0.01);
    assert_eq!(pens[0].limit, 6.0);
    // Another town has its usual pens.
    let other = (t + 1) % atlas::town_marks().len() as u16;
    assert!(w.animals.pens.iter().any(|p| p.town == Some(other)));
}

#[test]
fn a_plan_round_trips_through_its_file_and_a_save() {
    let p = test_plan();
    let back = Plan::from_ron(&p.to_ron()).unwrap();
    assert_eq!(back, p);
    let w = with_plan();
    let bytes = w.save_bytes();
    let w2 = World::load_bytes(&bytes).expect("load");
    assert_eq!(w2.plans, w.plans);
}

#[test]
fn lazs_plans_read_and_name_real_variants_places_and_species() {
    // (The sketch plans alone: a place built in the place builder overrides
    // a plan for the same town in `Plans::load`, and Laz's outpost has one.)
    let plans = Plans::load_from(&gahturiyu_sim::sim::plan::plans_dir().expect("the plans folder"));
    assert!(plans.for_town("last_wave_outpost").is_some(), "the outpost's plan is in assets/world/plans");
    for (key, p) in &plans.towns {
        assert!(atlas::town_marks().iter().any(|m| m.key() == *key), "{key}: no such mark in towns.ron");
        assert!(!p.buildings.is_empty(), "{key}: no buildings");
        for b in &p.buildings {
            if let Some(v) = &b.variant {
                let var = gahturiyu_sim::sim::layout::VARIANTS.iter().find(|x| x.key == v).unwrap_or_else(|| panic!("{key}: {}: no variant {v}", b.label));
                assert_eq!(var.kind, b.kind, "{key}: {}: {v} is not a {:?}", b.label, b.kind);
            }
            assert!(b.size > 3.0 && b.size < 80.0, "{key}: {}: size {}", b.label, b.size);
        }
        for pen in &p.pens {
            assert!(pen.species().is_some(), "{key}: {}: no species {}", pen.label, pen.species);
        }
        // No two buildings on top of each other.
        for (i, a) in p.buildings.iter().enumerate() {
            for b in &p.buildings[i + 1..] {
                let d = ((a.at.0 - b.at.0).powi(2) + (a.at.1 - b.at.1).powi(2)).sqrt();
                assert!(d > (a.size + b.size) * 0.5, "{key}: {} and {} overlap ({d:.0} m apart)", a.label, b.label);
            }
        }
    }
    // The world builds with them.
    let w = worldgen::generate_with_plans(1, &Rosters::default(), &plans);
    let t = town_index("last_wave_outpost");
    assert_eq!(w.settlements[t].buildings.len(), 1 + plans.for_town("last_wave_outpost").unwrap().buildings.len());
}

#[test]
fn the_drawn_woods_are_read_off_the_plan() {
    let plans = Plans::load_from(&gahturiyu_sim::sim::plan::plans_dir().expect("the plans folder"));
    let w = worldgen::generate_with_plans(1, &Rosters::default(), &plans);
    let t = town_index("last_wave_outpost");
    let c = w.settlements[t].pos;
    let at = |x: f32, y: f32| w.plan_woods_at(c.add(gahturiyu_sim::sim::geo::V2::new(x, y)));
    // Open ground at the hearth; thick wood between the fletcher's and the temple.
    assert_eq!(at(0.0, 0.0), Some(0.0), "the hearth stands on open ground");
    let wood = at(-43.0, -43.0).expect("inside the drawing");
    assert!(wood > 0.5, "wood {wood}");
    assert!(at(400.0, 0.0).is_none(), "outside the drawing the land grows as usual");
    for (x, y) in [(0.0, 0.0), (-43.0, -43.0), (60.0, 40.0)] {
        let p = c.add(gahturiyu_sim::sim::geo::V2::new(x, y));
        eprintln!("ground at ({x}, {y}): {:?}, height {:.0}, slope {:.2}, road {}, map class {:?}", w.terrain.ground(p), w.terrain.height(p), w.terrain.slope(p), w.terrain.on_road(p), atlas::atlas().class_at(p));
    }
}
