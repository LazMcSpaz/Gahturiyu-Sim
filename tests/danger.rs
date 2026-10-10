//! Reading danger before committing (play-test 3, item 12): what the game
//! calls a fight beforehand should match how such fights go.

use gahturiyu_sim::sim::{animals::Sp, geo::V2, worldgen, World};

fn squad_won(w: &mut World) -> bool {
    let mut n = 0;
    let herd_fight = !w.animals.frays.is_empty();
    while w.squad_battle().is_some() && n < 60_000 {
        w.step(0.1);
        n += 1;
    }
    if herd_fight {
        w.animals.attacks.last().is_some_and(|a| a.over && !a.animals_won && w.squad_fit().len() > 0)
    } else {
        w.log.iter().any(|l| l.1.starts_with("The fight is over: you won") || l.1.starts_with("The fight is over: they broke"))
    }
}

/// Prints the strength ratio against how the fight went, for setting the
/// words' thresholds. `cargo test --release --test danger calibrate -- --ignored --nocapture`
#[test]
#[ignore]
fn calibrate() {
    for n in [1usize, 2, 3, 4, 6, 8] {
        let mut wins = 0;
        let mut ratio = 0.0;
        for seed in 1..=6u64 {
            let mut w = worldgen::generate(seed);
            let squad = w.squad.members.clone();
            let us: f32 = squad.iter().map(|&m| w.people[m as usize].might).sum();
            let band = w.spawn_bandits(w.squad.pos.add(V2::new(15.0, 0.0)), n, n >= 3);
            let foes = w.group(band).unwrap().members.clone();
            let them: f32 = foes.iter().map(|&m| w.people[m as usize].might).sum();
            ratio += them / us / 6.0;
            let clock = std::time::Instant::now();
            let r = w.reading_vs_group(band);
            eprintln!("  seed {seed}: reading {r:?} -> {:?} in {:?}", r.odds(), clock.elapsed());
            assert!(w.attack(&squad, foes[0]));
            if squad_won(&mut w) {
                wins += 1;
            }
        }
        eprintln!("{n} bandits: ratio {ratio:.2}, squad won {wins}/6");
    }
    for sp in [Sp::Ridgehound, Sp::Wallowback, Sp::Mirejaw, Sp::ChasmLurker, Sp::Cragmaw, Sp::Briarback, Sp::SilkMother] {
        let mut wins = 0;
        let mut tries = 0;
        let mut ratio = 0.0;
        for seed in 1..=6u64 {
            let mut w = worldgen::generate(seed);
            let t = w.time;
            let Some(h) = w.animals.herds.iter().find(|h| h.sp == sp && h.alive(t) > 0).map(|h| h.id) else { continue };
            let at = w.herd_pos(h, t);
            w.teleport_squad(at.add(V2::new(12.0, 0.0)));
            let squad = w.squad.members.clone();
            let us: f32 = squad.iter().map(|&m| w.people[m as usize].might).sum();
            let them = w.animals.herds[h as usize].might(t);
            let clock = std::time::Instant::now();
            let r = w.reading_vs_herd(h);
            eprintln!("  seed {seed}: reading {r:?} -> {:?} in {:?}", r.odds(), clock.elapsed());
            if !w.hunt(&squad, h) {
                continue;
            }
            tries += 1;
            ratio += them / us;
            if squad_won(&mut w) {
                wins += 1;
            }
        }
        if tries > 0 {
            eprintln!("{sp:?}: ratio {:.2}, squad won {wins}/{tries}", ratio / tries as f32);
        }
    }
}

/// The readings say what the fights then do: one bandit is easy, eight are
/// far beyond a starting squad, and so is a Cragmaw (round 3: six went after
/// one marked only "huntable" and were all down in two minutes).
#[test]
fn readings_match_the_fights() {
    use gahturiyu_sim::sim::danger::Odds;
    for seed in 1..=2u64 {
        let mut w = worldgen::generate(seed);
        let one = w.spawn_bandits(w.squad.pos.add(V2::new(15.0, 0.0)), 1, false);
        assert_eq!(w.reading_vs_group(one).odds(), Odds::Easy, "seed {seed}: one bandit");
        let foes = w.group(one).unwrap().members.clone();
        let squad = w.squad.members.clone();
        assert!(w.attack(&squad, foes[0]));
        assert!(squad_won(&mut w), "seed {seed}: and the squad beats one bandit");

        let mut w = worldgen::generate(seed);
        let eight = w.spawn_bandits(w.squad.pos.add(V2::new(15.0, 0.0)), 8, true);
        assert_eq!(w.reading_vs_group(eight).odds(), Odds::Beyond, "seed {seed}: eight bandits");
        let foes = w.group(eight).unwrap().members.clone();
        let squad = w.squad.members.clone();
        assert!(w.attack(&squad, foes[0]));
        assert!(!squad_won(&mut w), "seed {seed}: and eight beat the squad");

        let mut w = worldgen::generate(seed);
        let t = w.time;
        let maw = w.animals.herds.iter().find(|h| h.sp == Sp::Cragmaw && h.alive(t) > 0).map(|h| h.id).expect("a Cragmaw");
        let at = w.herd_pos(maw, t);
        w.teleport_squad(at.add(V2::new(12.0, 0.0)));
        assert!(w.reading_vs_herd(maw).odds() >= Odds::Risky, "seed {seed}: a Cragmaw");
    }
}

/// A band of bandits coming into sight says it's bandits, and how a fight
/// would go, rather than "wandering, crosses your path".
#[test]
fn bandits_in_sight_say_so() {
    let mut w = worldgen::generate(3);
    let band = w.spawn_bandits(w.squad.pos.add(V2::new(400.0, 0.0)), 5, false);
    let line = w.describe_group(band);
    assert!(line.contains(": bandits"), "{line}");
    assert!(["Easy for you.", "A fair fight.", "Risky", "Far beyond you."].iter().any(|s| line.contains(s)), "{line}");
}
