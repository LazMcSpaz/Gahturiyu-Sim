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
//!
//! Spells are judged by what their effects do (mend, ward, hinder, blast,
//! strike), never by name, so a new spell in the list is used sensibly
//! without touching this file.

use super::combat::{Act, Battle, Order, ARCHER_DRAW, ARCHER_STOW, SQUAD_SIDE};
use super::geo::V2;
use super::effects::{Does, Lasts, Reach};
use super::items::{item, ItemId, Kind};
use super::magic::{all_spells, Aim, Spell, Style};
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

    // Someone carrying a body only moves where they're told.
    if me.burdened {
        b.fighters[i].target = None;
        return;
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

    // Archers: hand weapon out when someone closes in (or the arrows run
    // out), bow back out once there's room again.
    let me = &b.fighters[i];
    let near = b.nearest_enemy(i).map(|j| me.pos.dist(b.fighters[j].pos)).unwrap_or(f32::MAX);
    if matches!(me.act, Act::Idle) {
        if me.weapon.range > 0.0 && me.sidearm.is_some() && (near < ARCHER_DRAW || me.ammo == 0) {
            b.swap_weapon(i);
            return;
        }
        if me.stowed.is_some() && me.ammo > 0 && near > ARCHER_STOW {
            b.swap_weapon(i);
            return;
        }
    }

    let me = &b.fighters[i];
    if !matches!(me.act, Act::Idle) {
        return;
    }
    // A potion when badly hurt, or a tonic when a caster runs dry.
    let heal = me.potions.iter().copied().filter(|&p| potion(p, Does::Heal) > 0.0).max_by(|a, b| potion(*a, Does::Heal).total_cmp(&potion(*b, Does::Heal)));
    if me.vitality() < 0.4 {
        if let Some(p) = heal {
            b.begin_drink(i, p);
            return;
        }
    }
    let tonic = me.potions.iter().copied().find(|&p| potion(p, Does::Energy) > 0.0);
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

/// How much of `does` a potion gives when drunk (0 if none).
fn potion(it: ItemId, does: Does) -> f32 {
    if item(it).kind != Kind::Potion {
        return 0.0;
    }
    item(it).effects.iter().filter(|e| e.does == does).map(|e| e.power).sum()
}

fn has_scroll(f: &super::combat::Fighter, s: Spell) -> bool {
    let key = s.def().key;
    f.scrolls.iter().any(|&x| matches!(item(x).kind, Kind::Scroll(y) if y == key))
}

/// Cast from memory if there's energy for it, else read a scroll.
fn cast(b: &mut Battle, i: usize, s: Spell, target: Option<usize>, point: V2) {
    let f = &b.fighters[i];
    if f.spells.contains(&s) && f.mana >= s.def().cost {
        b.begin_cast(i, s, target, point);
    } else {
        b.read_scroll(i, s, target, point);
    }
}

/// What a spell is good for in a fight, read off its effects.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Use {
    /// Mends a friend.
    Mend,
    /// Wards the caster against harm.
    Ward,
    /// Speeds the caster up (to close a gap).
    Quicken,
    /// Hinders one enemy for a while.
    Hinder,
    /// Damage over an area.
    Blast,
    /// Damage to one enemy.
    Strike,
    /// Nothing useful mid-fight.
    Other,
}

fn use_of(s: Spell) -> Use {
    let d = s.def();
    let has = |f: &dyn Fn(Does) -> bool| d.effects.iter().any(|e| f(e.does));
    if has(&|x| x == Does::Heal) {
        Use::Mend
    } else if d.effects.iter().any(|e| matches!(e.does, Does::Damage(_)) && matches!(e.reach, Reach::Area { .. })) {
        Use::Blast
    } else if has(&|x| matches!(x, Does::Damage(_))) {
        Use::Strike
    } else if d.aim == Aim::Foe && d.effects.iter().any(|e| e.does.harmful() && matches!(e.lasts, Lasts::Secs(_))) {
        Use::Hinder
    } else if d.aim == Aim::Caster && has(&|x| matches!(x, Does::Haste | Does::MoveSpeed)) {
        Use::Quicken
    } else if d.aim == Aim::Caster && d.effects.iter().all(|e| !e.does.harmful() && matches!(e.lasts, Lasts::Secs(_))) {
        Use::Ward
    } else {
        Use::Other
    }
}

/// How much a spell's main effect does (for picking the strongest of a kind).
fn strength(s: Spell) -> f32 {
    s.def().effects.first().map(|e| e.power).unwrap_or(0.0)
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
    let usable = |s: Spell| s.def().style != Style::Ritual && ((me.spells.contains(&s) && me.mana >= s.def().cost) || has_scroll(&me, s));
    // Everything castable right now, by what it's for; strongest first.
    let mut castable: Vec<Spell> = all_spells().filter(|&s| usable(s)).collect();
    castable.sort_by(|a, c| strength(*c).total_cmp(&strength(*a)));
    let first = |u: Use| castable.iter().copied().find(|&s| use_of(s) == u);
    let enemies: Vec<usize> = (0..b.fighters.len()).filter(|&j| b.hostile(i, j) && b.fighters[j].active()).collect();
    if enemies.is_empty() {
        return;
    }
    let nearest = enemies.iter().map(|&j| me.pos.dist(b.fighters[j].pos)).fold(f32::MAX, f32::min);
    let is_mage = me.stats.calling == Calling::Mage;
    let in_range = |j: usize, s: Spell| me.pos.dist(b.fighters[j].pos) <= s.def().range;
    // Already in force on someone (the spell's first lasting effect).
    let has_it = |j: usize, s: Spell| s.def().effects.iter().any(|e| matches!(e.lasts, Lasts::Secs(_)) && b.fighters[j].has(e.does).is_some());

    // 0. Mend a friend who is down or badly hurt (or yourself).
    if let Some(s) = first(Use::Mend) {
        let range = s.def().range;
        let patient = (0..b.fighters.len())
            .filter(|&k| !b.hostile(i, k) && !b.fighters[k].dead && !b.fighters[k].fled)
            .filter(|&k| me.pos.dist(b.fighters[k].pos) <= range)
            .filter(|&k| b.fighters[k].ko || b.fighters[k].vitality() < 0.5)
            .min_by(|&x, &y| b.fighters[x].vitality().total_cmp(&b.fighters[y].vitality()));
        if let Some(k) = patient {
            cast(b, i, s, Some(k), b.fighters[k].pos);
            return;
        }
    }
    // 1. Ward yourself when the fighting starts or reaches you.
    if let Some(s) = castable.iter().copied().find(|&s| use_of(s) == Use::Ward && !has_it(i, s)) {
        if nearest < 6.0 || is_mage {
            cast(b, i, s, None, me.pos);
            return;
        }
    }
    // 2. Speed up to close a long gap.
    if let Some(s) = first(Use::Quicken).filter(|&s| !has_it(i, s)) {
        if let Some(t) = target.filter(|_| !is_mage) {
            if me.pos.dist(b.fighters[t].pos) > 10.0 {
                cast(b, i, s, None, me.pos);
                return;
            }
        }
    }
    // 3. Hinder the most dangerous enemy who isn't hindered that way yet.
    if r < 0.6 {
        for s in castable.iter().copied().filter(|&s| use_of(s) == Use::Hinder) {
            let pick = enemies.iter().copied().filter(|&j| !has_it(j, s) && in_range(j, s)).max_by(|&x, &y| b.fighters[x].might.total_cmp(&b.fighters[y].might));
            if let Some(j) = pick {
                if b.fighters[j].might > me.might * 0.5 {
                    cast(b, i, s, Some(j), b.fighters[j].pos);
                    return;
                }
            }
        }
    }
    // 4. Blast a knot of enemies, never with friends in it.
    if let Some(s) = first(Use::Blast) {
        let radius = s.def().radius();
        for &j in &enemies {
            let p = b.fighters[j].pos;
            if !in_range(j, s) {
                continue;
            }
            let caught = enemies.iter().filter(|&&k| b.fighters[k].pos.dist(p) <= radius).count();
            let friends = (0..b.fighters.len()).any(|k| !b.hostile(i, k) && !b.fighters[k].dead && b.fighters[k].pos.dist(p) <= radius + 0.8);
            if caught >= 2 && !friends {
                cast(b, i, s, Some(j), p);
                return;
            }
        }
    }
    // 5. Otherwise, strike the current target.
    if let Some(s) = first(Use::Strike) {
        if is_mage || r < 0.3 {
            if let Some(t) = target {
                if in_range(t, s) {
                    cast(b, i, s, Some(t), b.fighters[t].pos);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::magic::spell;

    #[test]
    fn spells_are_read_by_what_they_do() {
        assert_eq!(use_of(spell("mend")), Use::Mend);
        assert_eq!(use_of(spell("barrier")), Use::Ward);
        assert_eq!(use_of(spell("haste")), Use::Quicken);
        assert_eq!(use_of(spell("paralyze")), Use::Hinder);
        assert_eq!(use_of(spell("blind")), Use::Hinder);
        assert_eq!(use_of(spell("fireball")), Use::Blast);
        assert_eq!(use_of(spell("lightning_bolt")), Use::Strike);
    }
}
