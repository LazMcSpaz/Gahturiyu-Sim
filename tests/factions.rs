//! Factions F1 and F2 (Laz, `docs/factions.md`, `docs/leaders.md`): the
//! factions, their sub-factions and the guilds as data; the squad's standing
//! with each, moved by deeds (half again with a parent, rippling to friends
//! and foes); the Great House's lords drifting on their own at dawn; and what
//! standing does in a town that has a holder.

use gahturiyu_sim::names::{self, Tongue};
use gahturiyu_sim::sim::{
    factions::{self, def, id, parent, relation, Size, FACTIONS, HONOURED, LEADERS, PRICE_HONOURED, PRICE_UNWELCOME},
    jobs::Job,
    person::PersonId,
    world::{DAY, HOUR},
    worldgen, World,
};

#[test]
fn every_major_has_three_and_every_name_holds_together() {
    for (k, f) in FACTIONS.iter().enumerate() {
        assert_eq!(id(f.key) as usize, k);
        match f.parent {
            Some(p) => {
                assert_eq!(f.size, Size::Sub);
                assert_eq!(def(id(p)).size, Size::Major, "{}'s parent is a major", f.name);
            }
            None => assert_ne!(f.size, Size::Sub),
        }
        if f.size == Size::Major {
            let subs = FACTIONS.iter().filter(|s| s.parent == Some(f.key)).count();
            assert_eq!(subs, 3, "{} has three", f.name);
        }
        // Each has its native name from the naming module (Laz chose them),
        // but the Dueling Society, which is English only.
        if f.key != "dueling_society" {
            let t = match f.people {
                gahturiyu_sim::sim::race::Race::Roduro => Tongue::Roduro,
                gahturiyu_sim::sim::race::Race::Qotiro => Tongue::Qotiro,
                gahturiyu_sim::sim::race::Race::Horaro => Tongue::Horaro,
                gahturiyu_sim::sim::race::Race::Tadoro => Tongue::Tadoro,
            };
            assert!(names::native_name(f.name, t).is_some(), "{} has a native name", f.name);
        }
    }
    assert_eq!(names::native_name("The First Peoples", Tongue::Roduro).as_deref(), Some("ḍuʻaro"));
    assert_eq!(names::native_name("The Rooted", Tongue::Roduro).as_deref(), Some("hiqìro"));
    for l in LEADERS {
        let _ = id(l.faction);
    }
    // How they regard each other is the same both ways.
    for a in 0..FACTIONS.len() as u8 {
        for b in 0..FACTIONS.len() as u8 {
            assert_eq!(relation(a, b), relation(b, a));
        }
    }
    assert_eq!(relation(id("old_hands"), id("night_market")), -1.0, "the two syndicates");
    assert_eq!(relation(id("rooted"), id("writers")), -1.0, "the Rooted against everyone");
}

#[test]
fn a_deed_counts_with_the_parent_and_ripples_to_friends_and_foes() {
    let mut w = worldgen::generate(1);
    let (tl, fp, sh, hh, sw) = (id("true_line"), id("first_peoples"), id("seated_houses"), id("holy_house"), id("writers"));
    assert_eq!(parent(tl), fp);
    w.deed(tl, -10.0);
    assert_eq!(w.standing_with(tl), -10.0);
    assert_eq!(w.standing_with(fp), -5.0, "half again with the First Peoples");
    assert!((w.standing_with(sh) - 1.5).abs() < 1e-4, "the Seated Houses don't mind ({})", w.standing_with(sh));
    assert!((w.standing_with(hh) + 1.2).abs() < 1e-4, "the Holy House, their friend, minds a little");
    assert_eq!(w.standing_with(sw), 0.0, "the Writers don't care");
    for _ in 0..30 {
        w.deed(tl, -10.0);
    }
    assert_eq!(w.standing_with(tl), -100.0, "it bottoms out");
    assert_eq!(factions::standing_word(w.standing_with(tl)), "Hostile");
}

#[test]
fn the_lords_drift_on_their_own_and_change_sides_by_their_standing() {
    let mut w = worldgen::generate(1);
    assert_eq!(w.factions.lords.len(), 3);
    let start: Vec<f32> = w.factions.lords.iter().map(|l| l.throne).collect();
    let end = w.time + 20.0 * DAY;
    while w.time < end {
        w.step(HOUR);
    }
    let now: Vec<f32> = w.factions.lords.iter().map(|l| l.throne).collect();
    assert_ne!(start, now, "they drift");
    for l in &w.factions.lords {
        if l.throne > factions::BANNER_AT {
            assert_eq!(l.group, id("banner_lords"), "{}", l.name);
        }
        if l.throne < -factions::BANNER_AT {
            assert_eq!(l.group, id("free_lords"), "{}", l.name);
        }
    }
}

#[test]
fn the_lords_drift_the_same_however_the_world_is_stepped() {
    let w0 = worldgen::generate(2);
    let run = |step: f64| {
        let mut w = w0.clone();
        let end = w.time + 4.0 * DAY;
        while w.time < end - 1e-6 {
            w.step(step.min(end - w.time));
        }
        w.factions.clone()
    };
    assert_eq!(run(HOUR), run(600.0));
}

/// A merchant at work at 11 in the morning, and their town.
fn merchant(w: &mut World) -> PersonId {
    while w.time.rem_euclid(DAY) / HOUR < 11.0 || w.time.rem_euclid(DAY) / HOUR > 12.0 {
        w.step(HOUR / 2.0);
    }
    w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Merchant && w.at_work(p, w.time) && !w.for_sale(p).is_empty()).expect("a merchant at work")
}

#[test]
fn a_holder_who_likes_the_squad_sells_cheaper_and_one_who_loathes_it_wont_deal() {
    let mut w = worldgen::generate(1);
    let m = merchant(&mut w);
    let town = w.people[m as usize].home.unwrap();
    let plain = w.for_sale(m)[0];
    let it = plain.0;
    let sells = w.offer(m, it, None);
    // Nobody holds it: nothing changes.
    assert_eq!(w.regard_price(town), 1.0);
    let gh = id("great_house");
    w.settlements[town as usize].holder = Some(id("banner_lords"));
    assert_eq!(w.for_sale(m)[0], plain, "held, but nobody thinks anything of the squad yet");
    // Honoured by the Great House (entry reads the parent): cheaper.
    w.deed(gh, HONOURED + 5.0);
    assert_eq!(w.regard_price(town), PRICE_HONOURED);
    let cheap = w.for_sale(m).into_iter().find(|x| x.0 == it).unwrap();
    assert!(cheap.2 < plain.2 || plain.2 <= 2, "{} then {}", plain.2, cheap.2);
    // Unwelcome: dearer, and they pay less.
    w.deed(gh, -(HONOURED + 5.0) - 30.0);
    assert_eq!(w.regard_price(town), PRICE_UNWELCOME);
    let dear = w.for_sale(m).into_iter().find(|x| x.0 == it).unwrap();
    assert!(dear.2 > plain.2, "{} then {}", plain.2, dear.2);
    if let (Some(a), Some(b)) = (sells, w.offer(m, it, None)) {
        assert!(b <= a);
    }
    // Hostile: they won't deal at all.
    w.deed(gh, -60.0);
    assert!(!w.deals_with_squad(town));
    assert!(w.for_sale(m).is_empty());
    assert_eq!(w.offer(m, it, None), None);
}
