//! Laz's play notes of 11 Oct, magic (P16, P18): Douse reaches any fire by
//! one rule, not a list of special cases; a fire put out stays out by the
//! clock and comes back with Kindle; every cast outside a fight is noted for
//! the window.

use gahturiyu_sim::sim::{magic, solid, torch::FlameAt, worldgen, World};

/// The first squad member, knowing `keys`, with a full pool.
fn caster(w: &mut World, keys: &[&str]) -> u32 {
    let m = w.squad.members[0];
    let t = w.time;
    for k in keys {
        let s = magic::spell(k);
        let d = w.people[m as usize].detail.as_mut().unwrap();
        if !d.spells.contains(&s) {
            d.spells.push(s);
        }
    }
    let full = w.people[m as usize].max_mana();
    w.people[m as usize].set_mana(full, t);
    m
}

/// A town fire (not a camp's) near where the squad starts, burning.
fn town_fire(w: &World) -> (solid::Id, gahturiyu_sim::sim::geo::V2) {
    let at = w.squad.pos;
    w.flames_near(at, 15.0)
        .into_iter()
        .find_map(|(f, p, lit)| match f {
            FlameAt::Fire(id) if lit && !matches!(id, solid::Id::CampFire(_)) => Some((id, p)),
            _ => None,
        })
        .expect("a town fire burning near the start (world 1 starts by its hearth)")
}

/// Cast until it takes (a fizzle is a keyed roll; the next cast rolls anew).
fn cast(w: &mut World, m: u32, key: &str, at: gahturiyu_sim::sim::geo::V2) -> Result<(), String> {
    for _ in 0..12 {
        let t = w.time;
        let full = w.people[m as usize].max_mana();
        w.people[m as usize].set_mana(full, t);
        w.use_spell(m, magic::spell(key), None, Some(at)).map_err(|e| e.0.to_string())?;
        if w.cast_marks.last().unwrap().ok {
            return Ok(());
        }
    }
    panic!("{key} fizzled twelve times running");
}

#[test]
fn douse_puts_out_a_town_fire_and_kindle_lights_it_again() {
    let mut w = worldgen::generate(1);
    let m = caster(&mut w, &["douse", "kindle"]);
    let (id, at) = town_fire(&w);
    let seq = w.cast_seq;
    cast(&mut w, m, "douse", at).expect("douse at the town's fire");
    assert!(w.cast_seq > seq, "the cast is noted for the window");
    assert!(!w.fire_lit(id, w.time), "the fire is out");
    assert!(w.doused.iter().any(|d| d.0 == id), "and that's kept (and saved)");
    // Douse again: nothing burning there now.
    assert!(cast(&mut w, m, "douse", at).is_err(), "nothing left to put out");
    // Kindle brings it back.
    cast(&mut w, m, "kindle", at).expect("kindle at the cold fire");
    assert!(w.fire_lit(id, w.time), "lit again");
}

#[test]
fn a_doused_fire_lights_itself_again_by_the_clock() {
    let mut w = worldgen::generate(1);
    let m = caster(&mut w, &["douse"]);
    let (id, at) = town_fire(&w);
    cast(&mut w, m, "douse", at).unwrap();
    assert!(!w.fire_lit(id, w.time));
    let later = w.time + gahturiyu_sim::sim::torch::DOUSE_HOURS * gahturiyu_sim::sim::world::HOUR + 1.0;
    assert!(w.fire_lit(id, later), "out only for DOUSE_HOURS");
}
