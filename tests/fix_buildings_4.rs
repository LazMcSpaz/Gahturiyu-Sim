//! Fixes from the known-issues list, batch 4 (the buildings agent): Laz's
//! answers B3 and B4, and the small job slips.

use gahturiyu_sim::sim::{
    chances::Chance,
    items::{self, item, SLOTS},
    quests::{Quest, QuestKind, Stage},
    recruit::RECRUIT_KIT_MAX,
    world::DAY,
    worldgen,
};

fn play(save: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_play")).arg(save).args(args).output().expect("the play tool runs");
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success() && !text.contains("panicked"), "`{}` crashed:\n{text}", args.join(" "));
    text
}

/// B4 (NM-65): nobody is listed as willing to join before a word is said.
#[test]
fn b4_who_would_join_is_found_out_by_asking() {
    let dir = std::env::temp_dir().join(format!("gaht-fb4-{}-b4", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let save = dir.join("t.save");
    play(&save, &["new", "1"]);
    let w = worldgen::generate(1);
    assert!((0..w.settlements.len() as u16).any(|t| !w.willing_in(t).is_empty()), "someone would join");
    for cmd in [&["look"][..], &["look", "people"], &["town"]] {
        let out = play(&save, cmd);
        assert!(!out.to_lowercase().contains("join"), "`{}` tells: {out}", cmd.join(" "));
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// B3 (NM-52): a recruit brings little of their own; the good things stay home.
#[test]
fn b3_a_recruit_comes_with_little() {
    let w0 = worldgen::generate(1);
    let worth = |g: &gahturiyu_sim::sim::inventory::Gear| SLOTS.iter().filter_map(|&s| g.in_slot(s)).map(|i| item(i).value).fold(0.0f32, f32::max);
    // Someone willing who'd turn up in something good.
    let mut w = worldgen::generate(1);
    let lead = w.squad.members[0];
    let mut found = None;
    for t in 0..w0.settlements.len() as u16 {
        for (p, _) in w0.willing_in(t) {
            w.people[p as usize].ensure_detail();
            if worth(&w.people[p as usize].detail.as_ref().unwrap().gear) > RECRUIT_KIT_MAX {
                found = Some(p);
                break;
            }
        }
        if found.is_some() {
            break;
        }
    }
    let npc = found.expect("a recruit with good kit");
    w.people[lead as usize].detail.as_mut().unwrap().gear.add(items::id("coin"), 500);
    w.recruit(npc, lead).expect("joins");
    let g = &w.people[npc as usize].detail.as_ref().unwrap().gear;
    assert!(worth(g) <= RECRUIT_KIT_MAX, "came in kit worth {}", worth(g));
    assert!(g.bag.iter().all(|e| item(e.0).key == "coin" || item(e.0).value <= RECRUIT_KIT_MAX));
}

/// NM-58: a debt is never worth less than collecting it pays; a job says
/// where its people are.
#[test]
fn nm_58_debt_jobs_pay_less_than_the_debt_and_jobs_say_where() {
    let mut w = worldgen::generate(1);
    let mut seen = 0;
    for _ in 0..(20.0 * DAY / 600.0) as usize {
        w.step(600.0);
        for o in w.society.opps.iter().filter(|o| o.kind == Chance::CollectDebt) {
            seen += 1;
            assert!(o.favour || (o.reward as f32) <= o.amount, "{} coin to collect {}", o.reward, o.amount);
        }
    }
    eprintln!("{seen} debt-job sightings");
    // A letter to someone in a far town says how far, and which way.
    let here = w.squad.pos;
    let far = w.settlements.iter().max_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().id;
    let to = w.settlements[far as usize].residents[0];
    let giver = w.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).unwrap().residents[0];
    let q = Quest { id: 0, giver, kind: QuestKind::Deliver { to }, stage: Stage::Active, coin: 5, bonus: None, opp: None };
    let line = w.quest_line(&q);
    assert!(line.contains(" km "), "{line}");
}
