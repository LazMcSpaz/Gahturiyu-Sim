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

use super::combat::{Act, Battle, Fighter, Order, ARCHER_DRAW, ARCHER_STOW, SQUAD_SIDE};
use super::geo::V2;
use super::effects::{Does, Lasts, Reach};
use super::items::{item, ItemId, Kind};
use super::magic::{all_spells, Aim, Spell, Style};
use super::rng::Rng;
use super::stats::Calling;

/// Below this share of health on head or torso, someone is too far gone to
/// run: they fight on until they drop (so a won fight leaves bodies).
pub const COLLAPSE: f32 = 0.22;
/// The share of a squad caster's energy kept back for ordered casts.
pub const RESERVE: f32 = 0.34;

pub fn think(b: &mut Battle, i: usize, rng: &mut Rng) {
    // Draw the dice up front so every think consumes the same amount.
    let (r_flee, r_spell, r_pick) = (rng.f32(), rng.f32(), rng.f32());
    let me = &b.fighters[i];
    if me.fleeing || me.is_decoy() {
        return;
    }
    // The mindless (raised dead) go for whoever is nearest, and that's all.
    if me.summon.map(|s| s.mindless).unwrap_or(false) {
        b.fighters[i].target = b.nearest_enemy(i);
        return;
    }

    // Nerve. Your squad only runs if told to; others may break when hurt,
    // or when their side is clearly losing.
    if me.side != SQUAD_SIDE && me.is_person() {
        let mine = b.fighters.iter().filter(|f| f.side == me.side && f.active()).map(|f| f.might).sum::<f32>();
        let theirs = b.fighters.iter().filter(|f| f.side != me.side && f.active()).map(|f| f.might).sum::<f32>();
        let vit = me.vitality();
        // Too badly hurt to get away: they fight on until they drop.
        let spent = vit < COLLAPSE;
        // A friend running doesn't send the fresh off too: only the hurt, or
        // everyone when it's hopeless.
        let losing = theirs > mine * 2.5 && (vit < 0.75 || theirs > mine * 5.0);
        let hurt = vit < 0.35;
        // Whether they're one to run is settled once for the fight (one roll
        // keyed to the fight and the fighter), not re-rolled every think,
        // or everyone hurt would run sooner or later.
        let nerve = Rng::from_keys(&[b.seed, i as u64, 0x4E45_5256]).f32();
        let _ = r_flee;
        let gives = (1.0 - me.boldness) * if hurt && losing { 0.9 } else if hurt { 0.6 } else { 0.4 };
        if !spent && (hurt || losing) && nerve < gives {
            b.fighters[i].fleeing = true;
            b.fighters[i].order = None;
            let name = b.names[i].clone();
            b.log.push((b.time, format!("{name} turns and runs.")));
            return;
        }
    }

    // Calm: they stand there. Afraid: they run (see `Battle::idle`).
    if me.has(Does::Calm).is_some() || me.has(Does::Fear).is_some() {
        b.fighters[i].target = None;
        return;
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
    // An order to go for someone lapses once they've turned and run beyond
    // arm's reach: a runner is a little quicker than a chaser, so a chase
    // that hasn't landed a blow at once never will (it only drags the fight
    // out to its time limit).
    // (A beast that can still be got is another matter: `still_quarry`.)
    let runaway = |j: usize| b.fighters[j].fleeing && !still_quarry(me, &b.fighters[j]) && me.pos.dist(b.fighters[j].pos) > me.reach();
    let ordered = match me.order {
        Some(Order::Attack(j)) if b.fighters[j].active() && !runaway(j) => Some(j),
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
    if !me.spells.is_empty() || !me.scrolls.is_empty() || me.held.is_some() {
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

/// Release it if it's the ritual held ready, cast it from memory if there's
/// energy for it, else read a scroll.
fn cast(b: &mut Battle, i: usize, s: Spell, target: Option<usize>, point: V2) {
    let f = &b.fighters[i];
    if f.held == Some(s) {
        b.release(i, target, point);
    } else if s.def().style != Style::Ritual && f.spells.contains(&s) && f.mana >= s.def().cost {
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
    /// Lets the caster see in the dark.
    See,
    /// Calls up help.
    Summon,
    /// Gives back stamina.
    Breath,
    /// Ends spells on someone.
    Unravel,
    /// Raises the dead.
    Raise,
    /// Strengthens a friend for a while.
    Bolster,
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
    } else if has(&|x| matches!(x, Does::Summon(_))) {
        Use::Summon
    } else if has(&|x| x == Does::Stamina) {
        Use::Breath
    } else if has(&|x| x == Does::Dispel) {
        Use::Unravel
    } else if has(&|x| x == Does::Raise) {
        Use::Raise
    } else if d.effects.iter().any(|e| (matches!(e.does, Does::Damage(_)) && matches!(e.reach, Reach::Area { .. })) || e.does == Does::Blight) {
        Use::Blast
    } else if has(&|x| matches!(x, Does::Damage(_) | Does::Drain | Does::Wither)) {
        Use::Strike
    } else if d.aim == Aim::Foe && d.effects.iter().any(|e| e.does.harmful()) {
        Use::Hinder
    } else if d.aim == Aim::Caster && has(&|x| matches!(x, Does::Haste | Does::MoveSpeed)) {
        Use::Quicken
    } else if has(&|x| x == Does::Nightsight) {
        Use::See
    } else if d.aim == Aim::Caster && d.effects.iter().all(|e| e.does.guards() && matches!(e.lasts, Lasts::Secs(_))) {
        Use::Ward
    } else if d.aim == Aim::Friend && d.effects.iter().all(|e| !e.does.harmful() && matches!(e.lasts, Lasts::Secs(_)) && matches!(e.does, Does::Attr(_) | Does::Carry | Does::Enlarge | Does::ResistElements | Does::Haste | Does::Barrier | Does::Toughen)) {
        Use::Bolster
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
/// Whoever is after a beast keeps after it this far once it has turned
/// tail, metres (further with a bow: as far as it shoots).
pub const QUARRY_CHASE: f32 = 15.0;

/// A wild animal that has turned tail is still worth going after while it
/// can be got: it's close, or in bowshot, or too lame to outrun its hunter.
/// So a hunt has a chase in it: slow or hobbled prey can be run down, and
/// quick prey is gone in a few strides (BL-24). People who run are let go
/// (see `choose_target`).
fn still_quarry(me: &Fighter, them: &Fighter) -> bool {
    if them.home != super::animals::ANIMAL_SIDE {
        return false;
    }
    let far = me.pos.dist(them.pos);
    far <= me.attack_range().max(QUARRY_CHASE) || them.speed() < me.base_speed * super::body::leg_factor(&me.hp) * 0.95
}

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
        // An illusion is hard to ignore.
        if them.is_decoy() {
            s -= 6.0;
        }
        s
    };
    let best = (0..b.fighters.len())
        .filter(|&j| b.hostile(i, j) && b.fighters[j].active())
        // Someone hidden by a spell is lost beyond arm's reach.
        .filter(|&j| b.fighters[j].has(Does::Hide).is_none() || me.pos.dist(b.fighters[j].pos) <= 4.0)
        // Don't chase a runner who's already got a head start (unless it's
        // a beast that can still be got).
        .filter(|&j| !(b.fighters[j].fleeing && !still_quarry(me, &b.fighters[j]) && me.pos.dist(b.fighters[j].pos) > me.reach()))
        .min_by(|&x, &y| score(x).total_cmp(&score(y)))?;
    match me.target {
        Some(cur) if cur != best && b.fighters[cur].active() && !b.fighters[cur].fleeing && b.hostile(i, cur) && score(cur) < score(best) + 2.0 + jitter => Some(cur),
        _ => Some(best),
    }
}

/// Cast something useful, if anything is.
fn try_spell(b: &mut Battle, i: usize, target: Option<usize>, r: f32) {
    let me = b.fighters[i].clone();
    // Your squad's casters keep a reserve for what you order them to cast;
    // left to themselves they don't spend below it (except to mend).
    let reserve = if me.side == SQUAD_SIDE && me.is_person() { me.max_mana * RESERVE } else { 0.0 };
    let own = |s: Spell| me.spells.contains(&s) && me.mana >= s.def().cost + if use_of(s) == Use::Mend { 0.0 } else { reserve };
    // A scroll is the squad's to spend: you order those read.
    let scroll = |s: Spell| has_scroll(&me, s) && me.side != SQUAD_SIDE;
    let usable = |s: Spell| me.held == Some(s) || (s.def().style != Style::Ritual && (own(s) || scroll(s)));
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
    // Already in force on someone (the spell's first lasting effect), or —
    // for something done at once — nothing left for it to do.
    let has_it = |j: usize, s: Spell| {
        let f = &b.fighters[j];
        s.def().effects.iter().any(|e| match (e.lasts, e.does) {
            (Lasts::Secs(_), d) => f.has(d).is_some(),
            // Already reeling.
            (Lasts::Now, Does::Daze) => f.think_at > b.time + 0.05 || matches!(f.act, Act::Recover { .. }),
            // Nothing in their hands to break.
            (Lasts::Now, Does::Shatter) => f.weapon_name == "bare hands" && f.shield <= 0.0,
            _ => false,
        })
    };
    // A ritual your squad holds took hours to make: let it go only when it
    // really counts (others' AI doesn't hold rituals at all).
    let precious = |s: Spell| me.held == Some(s) && me.side == SQUAD_SIDE;

    // 0. Mend a friend who is down or badly hurt (or yourself).
    if let Some(s) = first(Use::Mend) {
        // (A spell that mends everyone round the caster reaches as far as its area.)
        let range = s.def().range.max(s.def().radius());
        let patient = (0..b.fighters.len())
            .filter(|&k| !b.hostile(i, k) && !b.fighters[k].dead && !b.fighters[k].fled && !b.fighters[k].is_decoy())
            .filter(|&k| me.pos.dist(b.fighters[k].pos) <= range)
            .filter(|&k| b.fighters[k].ko || (b.fighters[k].vitality() < 0.5 && !precious(s)))
            .min_by(|&x, &y| b.fighters[x].vitality().total_cmp(&b.fighters[y].vitality()));
        if let Some(k) = patient {
            cast(b, i, s, Some(k), b.fighters[k].pos);
            return;
        }
    }
    // 0b. Winded: get your breath back.
    if let Some(s) = first(Use::Breath) {
        if me.fatigue < me.max_fatigue * 0.3 {
            cast(b, i, s, Some(i), me.pos);
            return;
        }
    }
    // 1. Ward yourself when the fighting starts or reaches you. (Mages do
    //    it as a matter of course; fighters now and then. A brace only goes
    //    up against a blow on its way.)
    let blow_coming = (0..b.fighters.len()).any(|j| matches!(b.fighters[j].act, Act::Swing { target, .. } if target == i) && b.hostile(i, j));
    let ward_wanted = |s: Spell| {
        let brace = s.def().effects.iter().any(|e| e.does == Does::Brace);
        if brace {
            blow_coming && (is_mage || r < 0.3)
        } else {
            is_mage || (nearest < 6.0 && r < 0.15)
        }
    };
    if let Some(s) = castable.iter().copied().find(|&s| use_of(s) == Use::Ward && !has_it(i, s) && ward_wanted(s) && !precious(s)) {
        cast(b, i, s, None, me.pos);
        return;
    }
    // 1b. In the dark, see.
    if let Some(s) = first(Use::See).filter(|&s| !has_it(i, s)) {
        if let Some(t) = target {
            if b.light_at(b.fighters[t].pos) < 0.5 {
                cast(b, i, s, None, me.pos);
                return;
            }
        }
    }
    // 1c. Call up help, if none of ours is about yet.
    if let Some(s) = first(Use::Summon) {
        let ours = b.fighters.iter().any(|f| f.summon.is_some() && f.side == me.side && f.active());
        if !ours && nearest < 20.0 && (!precious(s) || enemies.len() >= 2) {
            // A couple of metres towards the enemy.
            let j = enemies.iter().copied().min_by(|&x, &y| me.pos.dist(b.fighters[x].pos).total_cmp(&me.pos.dist(b.fighters[y].pos))).unwrap();
            let dir = b.fighters[j].pos.sub(me.pos);
            let at = me.pos.add(dir.scale(2.5 / dir.len().max(0.01)));
            cast(b, i, s, None, at);
            return;
        }
    }
    // 1d. Unravel: send back an enemy's creature, strip an enemy's wards,
    //     or free a friend from a hostile spell.
    if let Some(s) = first(Use::Unravel) {
        let range = s.def().range;
        let pick = (0..b.fighters.len())
            .filter(|&k| b.fighters[k].active() && me.pos.dist(b.fighters[k].pos) <= range)
            .find(|&k| {
                let f = &b.fighters[k];
                if b.hostile(i, k) {
                    (f.summon.is_some() && !f.is_decoy()) || f.statuses.iter().filter(|st| !st.does.harmful()).count() >= 2
                } else {
                    f.statuses.iter().any(|st| matches!(st.does, Does::Paralyze | Does::Blind | Does::Dominate))
                }
            });
        if let Some(k) = pick {
            cast(b, i, s, Some(k), b.fighters[k].pos);
            return;
        }
    }
    // 1e. Raise the enemy dead (one body, or every body round the caster).
    if let Some(s) = first(Use::Raise) {
        let reach = s.def().range.max(s.def().radius());
        let bodies: Vec<usize> = (0..b.fighters.len()).filter(|&j| b.raisable(i, j) && me.pos.dist(b.fighters[j].pos) <= reach).collect();
        let area = s.def().radius() > 0.0;
        if (area && bodies.len() >= 2) || (!area && !bodies.is_empty()) {
            let p = b.fighters[bodies[0]].pos;
            cast(b, i, s, None, if area { me.pos } else { p });
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
    //    (Fighters who aren't mages seldom bother.)
    if r < if is_mage { 0.6 } else { 0.12 } {
        // Vary which one: start somewhere in the list by the dice.
        let hinders: Vec<Spell> = castable.iter().copied().filter(|&s| use_of(s) == Use::Hinder).collect();
        let start = (r * 7919.0) as usize;
        for k in 0..hinders.len() {
            let s = hinders[(start + k) % hinders.len()];
            let pick = enemies.iter().copied().filter(|&j| !has_it(j, s) && in_range(j, s)).max_by(|&x, &y| b.fighters[x].might.total_cmp(&b.fighters[y].might));
            if let Some(j) = pick {
                let worth = if precious(s) { me.might * 1.0 } else { me.might * 0.5 };
                if b.fighters[j].might > worth {
                    cast(b, i, s, Some(j), b.fighters[j].pos);
                    return;
                }
            }
        }
    }
    // 3b. Bolster a friend in the thick of it (mages).
    if is_mage && r < 0.4 {
        if let Some(s) = castable.iter().copied().find(|&s| use_of(s) == Use::Bolster) {
            let range = s.def().range;
            let friend = (0..b.fighters.len())
                .filter(|&k| k != i && !b.hostile(i, k) && b.fighters[k].active() && b.fighters[k].is_person())
                .filter(|&k| me.pos.dist(b.fighters[k].pos) <= range && !has_it(k, s))
                .filter(|&k| b.nearest_enemy(k).map(|e| b.fighters[k].pos.dist(b.fighters[e].pos) < 4.0).unwrap_or(false))
                .max_by(|&x, &y| b.fighters[x].might.total_cmp(&b.fighters[y].might));
            if let Some(k) = friend {
                cast(b, i, s, Some(k), b.fighters[k].pos);
                return;
            }
        }
    }
    // 4. Blast a knot of enemies, never with friends in it (unless the
    //    spell spares them).
    if let Some(s) = first(Use::Blast) {
        let radius = s.def().radius();
        let spares_friends = !s.def().effects.iter().any(|e| e.does.harmful() && matches!(e.reach, Reach::Area { who: super::effects::Who::All, .. } | Reach::Ground { .. }));
        for &j in &enemies {
            // Where the blast centres: on the caster for spells worked round them.
            let p = if s.def().aim == Aim::Caster { me.pos } else { b.fighters[j].pos };
            if s.def().aim != Aim::Caster && !in_range(j, s) {
                continue;
            }
            let caught = enemies.iter().filter(|&&k| b.fighters[k].pos.dist(p) <= radius).count();
            let friends = !spares_friends && (0..b.fighters.len()).any(|k| !b.hostile(i, k) && !b.fighters[k].dead && b.fighters[k].pos.dist(p) <= radius + 0.8);
            if caught >= if precious(s) { 3 } else { 2 } && !friends {
                cast(b, i, s, Some(j), p);
                return;
            }
        }
    }
    // 5. Otherwise, strike the current target. (Fighters who aren't mages
    //    only throw a spell at someone still out of reach of their weapon.)
    if let Some(s) = first(Use::Strike) {
        if let Some(t) = target {
            let far = me.pos.dist(b.fighters[t].pos) > me.attack_range() * 1.5;
            if (is_mage || (far && r < 0.3)) && in_range(t, s) {
                cast(b, i, s, Some(t), b.fighters[t].pos);
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
