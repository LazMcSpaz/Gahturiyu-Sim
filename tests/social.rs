//! The social stats (Laz, 13 Oct): Personality is an attribute of its own,
//! Speech and Trade are skills; a charmer is taken to better, a haggler
//! pays less and gets more, and talking and dealing grow them.

use gahturiyu_sim::sim::stats::{Attr, Skill, ATTRS, N_ATTRS, N_SKILLS, SKILLS};
use gahturiyu_sim::sim::worldgen;

#[test]
fn everyone_has_a_personality_and_the_two_skills() {
    assert_eq!(ATTRS.len(), N_ATTRS);
    assert_eq!(ATTRS[N_ATTRS - 1], Attr::Personality);
    assert_eq!(SKILLS.len(), N_SKILLS);
    assert_eq!(&SKILLS[N_SKILLS - 2..], &[Skill::Speech, Skill::Trade]);
    assert_eq!(Skill::Speech.governed_by(), Attr::Personality);
    assert_eq!(Skill::Trade.governed_by(), Attr::Personality);
    let w = worldgen::generate(3);
    let (mut lo, mut hi, mut sum) = (f32::MAX, f32::MIN, 0.0);
    for p in w.people.iter().take(2000) {
        let v = p.stats.attr(Attr::Personality);
        lo = lo.min(v);
        hi = hi.max(v);
        sum += v;
        assert!(p.stats.skill(Skill::Speech) >= 0.0 && p.stats.skill(Skill::Trade) >= 0.0);
    }
    let mean = sum / 2000.0;
    assert!(hi - lo > 20.0 && (20.0..45.0).contains(&mean), "a spread round an ordinary 32: {lo}..{hi}, mean {mean}");
    // The other attributes rolled as they always did (the new one has its own roll).
    let before = gahturiyu_sim::sim::stats::Stats::generate(gahturiyu_sim::sim::race::Race::Roduro, &w.people[5].traits, w.people[5].seed);
    assert_eq!(before.attr(Attr::Strength), w.people[5].stats.attr(Attr::Strength));
}

#[test]
fn a_charmer_is_taken_to_better_and_a_haggler_pays_less() {
    let mut w = worldgen::generate(3);
    let m = w.squad.members[0];
    let npc = *w.settlements[w.people[m as usize].home.unwrap() as usize].residents.iter().find(|&&p| p != m && !w.people[p as usize].in_squad).unwrap();
    let plain = {
        let s = &mut w.people[m as usize].stats;
        s.set_attr(Attr::Personality, 32.0);
        s.set_skill(Skill::Speech, 10.0);
        w.disposition(npc, m)
    };
    assert!((w.charm(m)).abs() < 0.01, "an ordinary person adds nothing: {}", w.charm(m));
    {
        let s = &mut w.people[m as usize].stats;
        s.set_attr(Attr::Personality, 62.0);
        s.set_skill(Skill::Speech, 50.0);
    }
    let charming = w.disposition(npc, m);
    assert!(charming > plain + 10.0, "a charmer is taken to better: {charming} vs {plain}");
    {
        let s = &mut w.people[m as usize].stats;
        s.set_attr(Attr::Personality, 12.0);
        s.set_skill(Skill::Speech, 0.0);
    }
    assert!(w.disposition(npc, m) < plain, "a boor worse");
    // Haggling: the squad's best Trade trims the markup and lifts the share.
    for &x in &w.squad.members.clone() {
        w.people[x as usize].stats.set_skill(Skill::Trade, 0.0);
    }
    let (markup0, share0) = (w.buy_markup(), w.sell_share());
    assert!((markup0 - gahturiyu_sim::sim::economy::BUY_MARKUP).abs() < 1e-6 && (share0 - gahturiyu_sim::sim::economy::SELL_SHARE).abs() < 1e-6);
    w.people[m as usize].stats.set_skill(Skill::Trade, 100.0);
    assert!(w.buy_markup() < markup0 && w.sell_share() > share0, "{} {}", w.buy_markup(), w.sell_share());
    assert!(w.buy_markup() > 1.0 && w.sell_share() < 1.0, "still a merchant's living");
}

#[test]
fn talking_grows_speech_and_dealing_grows_trade() {
    use gahturiyu_sim::sim::dialogue::Topic;
    let mut w = worldgen::generate(3);
    let m = w.squad.members[0];
    let npc = *w.settlements[w.people[m as usize].home.unwrap() as usize].residents.iter().find(|&&p| p != m && !w.people[p as usize].in_squad && w.building_at(w.person_pos(p)).is_none()).unwrap();
    let before = w.people[m as usize].stats.skill(Skill::Speech);
    w.teleport_squad(w.person_pos(npc).add(gahturiyu_sim::sim::geo::V2::new(1.5, 0.0)));
    assert!(w.order_talk(m, npc));
    for _ in 0..40 {
        w.step(0.5);
        if w.talk.is_some() {
            break;
        }
    }
    assert!(w.talk.is_some(), "a talk opened");
    let topic = w.topics().into_iter().find(|t| *t != Topic::Goodbye).expect("something to ask");
    w.ask(topic);
    assert!(w.people[m as usize].stats.skill(Skill::Speech) > before, "Speech grew by asking");
    // Dealing: a deal through the table practises Trade for the best haggler.
    let coin = gahturiyu_sim::sim::items::id("coin");
    w.people[m as usize].detail.as_mut().unwrap().gear.add(coin, 200);
    // (The best haggler of those at the table practises: any of the squad.)
    let trade = |w: &gahturiyu_sim::sim::World| w.squad.members.iter().map(|&p| w.people[p as usize].stats.skill(Skill::Trade)).sum::<f32>();
    let t0 = trade(&w);
    let merchant = w.settlements[w.people[m as usize].home.unwrap() as usize].residents.iter().copied().find(|&p| !w.for_sale(p).is_empty());
    if let Some(mer) = merchant {
        let (it, _, _) = w.for_sale(mer)[0];
        let mut table = gahturiyu_sim::sim::economy::Table::default();
        table.get.push((it, 1, None));
        w.end_talk();
        w.teleport_squad(w.person_pos(mer).add(gahturiyu_sim::sim::geo::V2::new(1.5, 0.0)));
        let d = w.deal_table(mer, &table);
        if d.stuck.is_none() {
            assert!(trade(&w) > t0, "Trade grew by dealing");
        }
    }
}
