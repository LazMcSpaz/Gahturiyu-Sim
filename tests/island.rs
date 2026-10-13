//! Salt Isle, the tutorial island (`island.rs`): it's land in the sea, the
//! same in every world; its compound is a real settlement that no road or
//! caravan reaches, where slavery is allowed and the debtors are bound; and
//! nothing else about a world moved for it.

use gahturiyu_sim::sim::culture::Slavery;
use gahturiyu_sim::sim::geo::{self, V2};
use gahturiyu_sim::sim::island::{Spot, TEST_QUARTER_HALF};
use gahturiyu_sim::sim::jobs::{Job, PlaceKind};
use gahturiyu_sim::sim::worldgen;

#[test]
fn the_island_is_land_in_the_sea() {
    let c = geo::isle_centre();
    assert!(geo::isle_inland(c) > 200.0, "the centre is well inland: {}", geo::isle_inland(c));
    assert!(geo::is_land(c) && geo::inland(c) > 200.0);
    // Round it, sea; the mainland's shore is a long way east.
    for a in [0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
        let far = c.add(V2::new(a.cos() * 700.0, a.sin() * 700.0));
        assert!(!geo::is_land(far), "sea 700 m out at {far:?}");
        assert!(geo::inland(far) < 0.0);
    }
    // Off the mainland's shore by `ISLE_OFF`, in the atlas's sea (`atlas.rs`).
    let atlas = gahturiyu_sim::sim::atlas::atlas();
    assert!(atlas.inland(c) <= -(geo::ISLE_OFF + geo::ISLE_A) + 1.0, "the island's centre is {} m from the mainland", -atlas.inland(c));
    // Nothing else changed: a mainland point's inland distance is the atlas's.
    let p = atlas.land_spot(99);
    assert_eq!(geo::inland(p), atlas.inland(p));
    // The terrain agrees, and the island is low and flat where the compound stands.
    let w = worldgen::generate(7);
    let isle = w.island().expect("the world has Salt Isle");
    assert!(!w.terrain.is_sea(c));
    for s in Spot::ALL {
        let at = isle.spot(s);
        if s == Spot::Wreck {
            assert!(w.terrain.is_sea(at), "the wreck lies in the shallows");
        } else {
            assert!(!w.terrain.is_sea(at) && geo::isle_inland(at) > 10.0, "{s:?} is on the island, {} m in", geo::isle_inland(at));
        }
    }
    let (a, b) = isle.test_quarter();
    assert!((b.x - a.x - 2.0 * TEST_QUARTER_HALF).abs() < 0.01);
    for (x, y) in [(a.x, a.y), (b.x, a.y), (a.x, b.y), (b.x, b.y), ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)] {
        let p = V2::new(x, y);
        assert!(!w.terrain.is_sea(p), "the test quarter corner {p:?} is on land");
        assert!(w.terrain.slope(p) < 0.12, "the test quarter is flat at {p:?}: slope {}", w.terrain.slope(p));
    }
    assert!(isle.in_test_quarter(isle.spot(Spot::TestQuarter)) && !isle.in_test_quarter(isle.spot(Spot::Hall)));
    // The briar patch is fenced (solid), with a gate through.
    use gahturiyu_sim::sim::solid::{Id, What};
    let briar = isle.spot(Spot::Briar);
    let near = w.solids_near(briar, gahturiyu_sim::sim::island::BRIAR_RADIUS + 5.0);
    assert!(near.iter().any(|t| t.id == Id::Briar && t.what == What::Fence), "the briar's fence");
    assert!(near.iter().any(|t| t.id == Id::BriarGate && t.what == What::Gate), "and its gate");
    let outside = briar.add(V2::new(gahturiyu_sim::sim::island::BRIAR_RADIUS + 3.0, 0.0));
    assert!(w.blocked_at(briar, 0.4).is_none() && w.blocked_at(outside, 0.4).is_none(), "open ground inside and out");
    assert!(!w.way_clear(outside, briar, 0.4), "the fence stops a walker going straight through");
    assert!(w.route(outside, briar).0.len() >= 2, "a way in through the gate");
}

#[test]
fn the_compound_is_a_settlement_apart() {
    let w = worldgen::generate(3);
    let isle = w.island().unwrap();
    let t = isle.settlement;
    let s = &w.settlements[t as usize];
    assert!(s.island && t as usize == w.settlements.len() - 1, "the compound is the last settlement");
    assert_eq!(s.name, "Salt Isle");
    assert!(w.settlements[..t as usize].iter().all(|o| !o.island));
    assert_eq!(s.holder, Some(gahturiyu_sim::sim::factions::id("free_lords")));
    // Its people live there and nowhere else; all extra, after everyone else.
    assert!(s.residents.len() >= 30);
    for &p in &s.residents {
        assert_eq!(w.people[p as usize].home, Some(t));
        assert!(w.people[p as usize].dwelling.is_some());
    }
    // No road reaches it.
    for road in &w.routes.roads {
        for p in road {
            assert!(!isle.contains(*p), "a road runs onto the island at {p:?}");
        }
    }
    assert!(w.routes.between(0, t).len() < 2, "no route to the island");
    // Slavery is allowed, whatever the blend said, and the debtors are bound to the steward for life.
    let tl = &w.society.towns[t as usize];
    assert_eq!(w.society.communities[tl.shore as usize].customs.slavery, Slavery::Allowed);
    let bound: Vec<_> = w.bonds.iter().filter(|b| b.town == t).collect();
    assert_eq!(bound.len(), gahturiyu_sim::sim::island::DEBTORS.iter().sum::<usize>());
    assert!(bound.iter().all(|b| b.slave && b.holder == Some(isle.steward) && b.until.is_infinite()));
    let guards = s.residents.iter().filter(|&&p| w.society.lives[p as usize].job == Job::Guard).count();
    assert!(guards >= gahturiyu_sim::sim::island::GUARDS, "the Binders stand guard: {guards}");
    // The workplaces the brief names, where it puts them.
    for (kind, spot) in [(PlaceKind::Workyard, Spot::Workyard), (PlaceKind::Quarry, Spot::Quarry), (PlaceKind::Woodlot, Spot::Woodlot), (PlaceKind::Fields, Spot::Fields), (PlaceKind::Dock, Spot::Dock), (PlaceKind::GuardPost, Spot::Gate)] {
        let at = isle.spot(spot);
        assert!(tl.places.iter().any(|p| p.kind == kind && p.pos.dist(at) < 1.0), "{kind:?} at {spot:?}");
    }
    // Something to cut at the woodlot.
    assert!(w.deposits.iter().any(|d| d.town == t), "the island has a face to work");
}

#[test]
fn the_squad_still_starts_in_town() {
    let w = worldgen::generate(11);
    let isle = w.island().unwrap();
    assert!(!isle.contains(w.squad.pos), "the plain start is unchanged until the opening lands");
    let home = w.people[w.squad.members[0] as usize].home.unwrap();
    assert!(!w.settlements[home as usize].island);
}
