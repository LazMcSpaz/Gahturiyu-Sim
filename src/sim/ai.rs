//! What fighters decide to do, a few times a second.
//!
//! The same brain runs your squad and their enemies; the difference is that
//! your squad obeys orders (go there, hit that one) and never breaks and runs
//! on its own, while everyone else can lose their nerve when badly hurt —
//! bolder people hold on longer.
//!
//! Each think: should I run? → am I under orders? → who should I fight? →
//! is there a spell worth casting right now? Movement and swinging then follow
//! from the target in `Battle::idle`.

use super::combat::{Act, Battle, Order, SQUAD_SIDE};
use super::geo::V2;
use super::items::{item, ItemId, Kind, PotionDef};
use super::magic::{Spell, StatusKind};
use super::rng::Rng;
use super::stats::Calling;

pub fn think(b: &mut Battle, i: usize, rng: &mut Rng) {
    // Draw the dice up front so every think consumes the same amount.
    let (r_flee, r_spell, r_pick) = (rng.f32(), rng.f32(), rng.f32());
    let me = &b.fighters[i];
    if me.fleeing {
        return;
    }

    // Nerve. Your squad only runs if told to; others may break when hurt,
    // or when their side is clearly losing.
    if me.side != SQUAD_SIDE {
        let mine = b.fighters.iter().filter(|f| f.side == me.side && f.active()).map(|f| f.might).sum::<f32>();
        let theirs = b.fighters.iter().filter(|f| f.side != me.side && f.active()).map(|f| f.might).sum::<f32>();
        let losing = theirs > mine * 2.5;
        let hurt = me.vitality() < 0.35;
        if (hurt || losing) && r_flee < (1.0 - me.boldness) * if hurt && losing { 0.6 } else { 0.25 } {
            b.fighters[i].fleeing = true;
            b.fighters[i].order = None;
            let name = b.names[i].clone();
            b.log.push((b.time, format!("{name} turns and runs.")));
            return;
        }
    }

    // Orders come first.
    if let Some(Order::MoveTo(_)) = me.order {
        b.fighters[i].target = None;
        return;
    }
    let ordered = match me.order {
        Some(Order::Attack(j)) if b.fighters[j].active() => Some(j),
        Some(Order::Attack(_)) => {
            b.fighters[i].order = None;
            None
        }
        _ => None,
    };
    let target = ordered.or_else(|| choose_target(b, i, r_pick));
    b.fighters[i].target = target;

    let me = &b.fighters[i];
    if !matches!(me.act, Act::Idle) {
        return;
    }
    // A potion when badly hurt, or a tonic when a caster runs dry.
    let heal = me.potions.iter().copied().filter(|&p| matches!(item(p).kind, Kind::Potion(d) if d.heal > 0.0)).max_by(|a, b| potion(*a).heal.total_cmp(&potion(*b).heal));
    if me.vitality() < 0.4 {
        if let Some(p) = heal {
            b.begin_drink(i, p);
            return;
        }
    }
    let tonic = me.potions.iter().copied().find(|&p| potion(p).mana > 0.0);
    if !me.spells.is_empty() && me.mana < 15.0 {
        if let Some(p) = tonic {
            b.begin_drink(i, p);
            return;
        }
    }
    if !me.spells.is_empty() || !me.scrolls.is_empty() {
        try_spell(b, i, target, r_spell);
    }
}

fn potion(it: ItemId) -> PotionDef {
    match item(it).kind {
        Kind::Potion(d) => d,
        _ => PotionDef { heal: 0.0, mana: 0.0 },
    }
}

fn has_scroll(f: &super::combat::Fighter, s: Spell) -> bool {
    f.scrolls.iter().any(|&x| matches!(item(x).kind, Kind::Scroll(y) if y == s))
}

/// Cast from memory if there's mana for it, else read a scroll.
fn cast(b: &mut Battle, i: usize, s: Spell, target: Option<usize>, point: V2) {
    let f = &b.fighters[i];
    if f.spells.contains(&s) && f.mana >= s.def().cost {
        b.begin_cast(i, s, target, point);
    } else {
        b.read_scroll(i, s, target, point);
    }
}

/// Pick who to fight: near, already fighting me, and nearly beaten all count.
/// Sticks with the current target unless something is clearly better.
fn choose_target(b: &Battle, i: usize, jitter: f32) -> Option<usize> {
    let me = &b.fighters[i];
    let score = |j: usize| -> f32 {
        let them = &b.fighters[j];
        let mut s = me.pos.dist(them.pos);
        if them.target == Some(i) {
            s -= 3.0;
        }
        s += them.vitality() * 4.0;
        if them.fleeing {
            s += 30.0; // let runners go
        }
        s
    };
    let best = (0..b.fighters.len())
        .filter(|&j| b.hostile(i, j) && b.fighters[j].active())
        // Don't chase a runner who's already got a head start.
        .filter(|&j| !(b.fighters[j].fleeing && me.pos.dist(b.fighters[j].pos) > me.reach() + 4.0))
        .min_by(|&x, &y| score(x).total_cmp(&score(y)))?;
    match me.target {
        Some(cur) if cur != best && b.fighters[cur].active() && !b.fighters[cur].fleeing && b.hostile(i, cur) && score(cur) < score(best) + 2.0 + jitter => Some(cur),
        _ => Some(best),
    }
}

/// Cast something useful, if anything is.
fn try_spell(b: &mut Battle, i: usize, target: Option<usize>, r: f32) {
    let me = b.fighters[i].clone();
    let knows = |s: Spell| (me.spells.contains(&s) && me.mana >= s.def().cost) || has_scroll(&me, s);
    let enemies: Vec<usize> = (0..b.fighters.len()).filter(|&j| b.hostile(i, j) && b.fighters[j].active()).collect();
    if enemies.is_empty() {
        return;
    }
    let nearest = enemies.iter().map(|&j| me.pos.dist(b.fighters[j].pos)).fold(f32::MAX, f32::min);
    let is_mage = me.stats.calling == Calling::Mage;
    let in_range = |j: usize, s: Spell| me.pos.dist(b.fighters[j].pos) <= s.def().range;

    // 0. Mend a friend who is down or badly hurt (or yourself).
    if knows(Spell::Heal) {
        let range = Spell::Heal.def().range;
        let patient = (0..b.fighters.len())
            .filter(|&k| !b.hostile(i, k) && !b.fighters[k].dead && !b.fighters[k].fled)
            .filter(|&k| me.pos.dist(b.fighters[k].pos) <= range)
            .filter(|&k| b.fighters[k].ko || b.fighters[k].vitality() < 0.5)
            .min_by(|&x, &y| b.fighters[x].vitality().total_cmp(&b.fighters[y].vitality()));
        if let Some(k) = patient {
            cast(b, i, Spell::Heal, Some(k), b.fighters[k].pos);
            return;
        }
    }
    // 1. Ward yourself when the fighting starts or reaches you.
    if knows(Spell::MageArmor) && me.has(StatusKind::MageArmor).is_none() && (nearest < 6.0 || is_mage) {
        cast(b, i, Spell::MageArmor, None, me.pos);
        return;
    }
    // 2. Speed up to close a long gap.
    if knows(Spell::Haste) && me.has(StatusKind::Hasted).is_none() && !is_mage {
        if let Some(t) = target {
            if me.pos.dist(b.fighters[t].pos) > 10.0 {
                cast(b, i, Spell::Haste, None, me.pos);
                return;
            }
        }
    }
    // 3. Hold the most dangerous enemy still.
    if knows(Spell::Paralyze) && r < 0.7 {
        let strongest = enemies
            .iter()
            .copied()
            .filter(|&j| !b.fighters[j].paralyzed() && in_range(j, Spell::Paralyze))
            .max_by(|&x, &y| b.fighters[x].might.total_cmp(&b.fighters[y].might));
        if let Some(j) = strongest {
            if b.fighters[j].might > me.might * 0.8 {
                cast(b, i, Spell::Paralyze, Some(j), b.fighters[j].pos);
                return;
            }
        }
    }
    // 4. Fireball a knot of enemies, never with friends in it.
    if knows(Spell::Fireball) {
        let radius = Spell::Fireball.def().radius;
        for &j in &enemies {
            let p = b.fighters[j].pos;
            if !in_range(j, Spell::Fireball) {
                continue;
            }
            let caught = enemies.iter().filter(|&&k| b.fighters[k].pos.dist(p) <= radius).count();
            let friends = (0..b.fighters.len()).any(|k| !b.hostile(i, k) && !b.fighters[k].dead && b.fighters[k].pos.dist(p) <= radius + 0.8);
            if caught >= 2 && !friends {
                cast(b, i, Spell::Fireball, Some(j), p);
                return;
            }
        }
    }
    // 5. Blind a strong fighter who isn't blind yet.
    if knows(Spell::Blind) && r < 0.5 {
        let pick = enemies
            .iter()
            .copied()
            .filter(|&j| b.fighters[j].has(StatusKind::Blinded).is_none() && in_range(j, Spell::Blind))
            .max_by(|&x, &y| b.fighters[x].might.total_cmp(&b.fighters[y].might));
        if let Some(j) = pick {
            cast(b, i, Spell::Blind, Some(j), b.fighters[j].pos);
            return;
        }
    }
    // 6. Otherwise, lightning on the current target.
    if knows(Spell::LightningBolt) && (is_mage || r < 0.3) {
        if let Some(t) = target {
            if in_range(t, Spell::LightningBolt) {
                cast(b, i, Spell::LightningBolt, Some(t), b.fighters[t].pos);
            }
        }
    }
}
