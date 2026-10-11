//! The seven biomes (Laz, 11 Oct 2026): rolling plains, forests, coast, bog,
//! rocky ground (ravines and chasms too), snowy mountains and the
//! Overgrowth. No desert, steppe or dry plateau anywhere.

use gahturiyu_sim::sim::animals::{Habitat, Sp};
use gahturiyu_sim::sim::biome::{self, Biome, BIOMES};
use gahturiyu_sim::sim::geo::V2;
use gahturiyu_sim::sim::race::Race;
use gahturiyu_sim::sim::settlement::Settlement;
use gahturiyu_sim::sim::terrain::Ground;
use gahturiyu_sim::sim::{weather, worldgen};

#[test]
fn every_biome_is_on_the_map_and_nothing_else() {
    for seed in [1u64, 2, 3] {
        let w = worldgen::generate(seed);
        let s = biome::shares(&w, 150.0);
        assert!((s.iter().sum::<f32>() - 1.0).abs() < 1e-3);
        for b in Biome::ALL {
            assert!(s[b as usize] > 0.01, "world {seed}: hardly any {} ({:.1}%)", b.name(), s[b as usize] * 100.0);
        }
        // The plains are the commonest country.
        assert!((0..BIOMES).all(|k| s[Biome::Plains as usize] >= s[k]), "world {seed}: {s:?}");
    }
}

#[test]
fn the_raised_south_east_is_green_plains_with_a_rocky_rim() {
    let w = worldgen::generate(1);
    let (mut top, mut plains_or_forest, mut rim, mut rocky_rim) = (0, 0, 0, 0);
    for j in 0..60 {
        for i in 0..60 {
            let p = V2::new(10_000.0 + i as f32 * 150.0, 9_000.0 + j as f32 * 150.0);
            if w.terrain.is_sea(p) {
                continue;
            }
            let hl = w.terrain.highland(p);
            if hl > 0.9 && w.terrain.mountains(p) < 0.2 {
                top += 1;
                if matches!(w.biome_at(p), Some(Biome::Plains | Biome::Forest | Biome::Bog | Biome::Overgrowth)) {
                    plains_or_forest += 1;
                }
            }
            if w.terrain.on_rim(p) {
                rim += 1;
                rocky_rim += (w.terrain.ground(p) == Ground::Rock && matches!(w.biome_at(p), Some(Biome::Rocky | Biome::Overgrowth))) as u32;
            }
        }
    }
    assert!(top > 100, "the highland should be there: {top}");
    assert!(plains_or_forest * 10 >= top * 9, "{plains_or_forest} of {top} highland spots are plains or forest");
    assert!(rim > 5 && rocky_rim == rim, "{rocky_rim} of {rim} rim spots are rocky ground");
    // Its weather is the plains' (there is no plateau climate).
    assert_eq!(weather::region_at(&w.terrain, V2::new(14_500.0, 14_500.0)), weather::Region::Plains);
}

#[test]
fn qotiro_towns_still_stand_in_the_south_east() {
    for seed in [1u64, 2, 3] {
        let w = worldgen::generate(seed);
        // Founded thickest in the south-east, as before: a bigger share of
        // the towns there than of the inland towns elsewhere.
        let inland: Vec<_> = w.settlements.iter().filter(|s| !s.coastal).collect();
        let (se, rest): (Vec<_>, Vec<_>) = inland.iter().partition(|s| s.pos.x > 11_500.0 && s.pos.y > 10_500.0);
        let share = |v: &[&&Settlement]| v.iter().filter(|s| s.founders == Race::Qotiro).count() as f32 / v.len().max(1) as f32;
        assert!(!se.is_empty() && share(&se) > share(&rest), "world {seed}: Qotiro found {:.0}% of the south-east's towns, {:.0}% elsewhere", share(&se) * 100.0, share(&rest) * 100.0);
        // And each town stands in one of the biomes.
        for s in &w.settlements {
            assert!(w.biome_at(s.pos).is_some(), "{} stands in the sea", s.name);
        }
    }
}

#[test]
fn animals_live_in_the_biomes_laz_gave_them() {
    let has = |sp: Sp, hs: &[Habitat]| {
        let mine = sp.def().habitats;
        assert_eq!(mine.len(), hs.len(), "{}: {:?}", sp.name(), mine);
        for h in hs {
            assert!(mine.contains(h), "{}: {:?}", sp.name(), mine);
        }
    };
    has(Sp::Dustrunner, &[Habitat::Plains]);
    has(Sp::Wallowback, &[Habitat::Bog, Habitat::Coast]);
    has(Sp::Mirejaw, &[Habitat::Bog]);
    has(Sp::Brushleaper, &[Habitat::Plains, Habitat::Forest]);
    has(Sp::CragGrazer, &[Habitat::Rocky, Habitat::Massif]);
    has(Sp::Cragmaw, &[Habitat::Massif]);
    has(Sp::ChasmLurker, &[Habitat::Ledge]);
    // Nobody's description sends them to dry country.
    for sp in gahturiyu_sim::sim::animals::ALL_SPECIES {
        let d = sp.def().describe.to_lowercase();
        for word in ["plateau", "desert", "arid", "steppe", "dry country"] {
            assert!(!d.contains(word), "{}: \"{}\"", sp.name(), sp.def().describe);
        }
    }
}

#[test]
fn a_spots_biome_is_the_same_however_it_is_asked() {
    let a = worldgen::generate(2);
    let b = worldgen::generate(2);
    let path = std::env::temp_dir().join(format!("biomes_{}.sav", std::process::id()));
    a.save_to(&path).unwrap();
    let c = gahturiyu_sim::sim::World::load_from(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    for k in 0..400 {
        let p = V2::new(250.0 + (k % 20) as f32 * 1030.0, 250.0 + (k / 20) as f32 * 1030.0);
        assert_eq!(a.biome_at(p), b.biome_at(p));
        assert_eq!(a.biome_at(p), c.biome_at(p));
    }
}
