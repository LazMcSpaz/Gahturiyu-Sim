//! What the player can do with what's under the mouse (after Baldur's Gate
//! 3 and Kenshi): every hoverable thing offers a list of choices. The first
//! is what a left-click does (and is named beside the cursor, in red when
//! it's a crime); a right-click opens the whole list. One place decides
//! both, so the label always tells the truth about the click.
//!
//! Window code: it only reads the world and sends the same orders the
//! keys and clicks always sent.

use gahturiyu_sim::sim::{
    buildings::DoorId,
    containers::ContainerId,
    geo::V2,
    items::{self, item},
    layout::Furn,
    person::PersonId,
    World,
};

use super::animals::{Seen, Thing};
use super::app::Hover;

/// Something the player can order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    /// Walk the selected to a spot.
    Walk(V2),
    Select(PersonId),
    Attack(PersonId),
    Talk(PersonId),
    Loot(PersonId),
    Carry(PersonId),
    Enter(DoorId),
    PickLock(DoorId),
    Search(ContainerId),
    PickChest(ContainerId),
    Gather(u32),
    PickUp(u32),
    Work(u32),
    Hunt(u32),
    Butcher(u32),
    /// Walk to a bed and sleep in it.
    Sleep(V2),
    TownPanel(u16),
    /// Pin the long description (what holding Alt shows).
    Examine,
    /// Walk somewhere sneaking.
    SneakTo(V2),
}

/// An open right-click menu: where it was opened, and what was there.
#[derive(Clone, Copy)]
pub struct Menu {
    pub at: bevy::math::Vec2,
    pub hover: Option<Hover>,
    pub wild: Option<Seen>,
    pub ground: Option<V2>,
}

/// The menu's choices: the thing's, or for bare ground, where to go.
pub fn menu_choices(w: &World, m: &Menu, who: &[PersonId]) -> (String, Vec<Choice>) {
    if let Some(h) = m.hover {
        let title = short_name(w, h).unwrap_or_else(|| "This".into());
        return (title, choices(w, h, who, false));
    }
    if let Some(s) = m.wild {
        let title = match s.thing {
            Thing::Carcass(_) => format!("{} carcass", s.sp.def().name),
            _ => s.sp.def().name.to_string(),
        };
        return (title, wild_choices(w, &s));
    }
    match m.ground {
        Some(p) => ("Here".into(), vec![ch("Walk here", Act::Walk(p)), ch("Sneak here", Act::SneakTo(p))]),
        None => ("Nothing here".into(), Vec::new()),
    }
}

/// Draw the right-click menu; returns the choice clicked and its box.
pub fn draw_menu(c: &super::hud::Canvas, w: &World, m: &Menu, who: &[PersonId], mouse: bevy::math::Vec2, click: Option<super::squadui::Click>) -> (Option<Act>, super::squadui::Bx) {
    use super::hud::Face;
    use super::palette::{eg, ega, BRASS, BRASS_LIGHT, TEXT};
    let (title, cs) = menu_choices(w, m, who);
    let row = 26.0;
    let wd = cs.iter().map(|x| c.styled_width(&x.label, 15.0, Face::Body, 0.0)).fold(c.styled_width(&title.to_uppercase(), 13.0, Face::Title, 2.0), f32::max) + 40.0;
    let h = 40.0 + cs.len() as f32 * row;
    let x = m.at.x.min(c.w - wd - 8.0).max(8.0);
    let y = m.at.y.min(c.h - h - 8.0).max(8.0);
    c.frame_box(x, y, wd, h);
    c.styled(&title.to_uppercase(), x + 16.0, y + 24.0, 13.0, eg(BRASS_LIGHT), Face::Title, 2.0);
    let mut picked = None;
    for (i, x_) in cs.iter().enumerate() {
        let ry = y + 34.0 + i as f32 * row;
        let b = super::squadui::Bx::new(x + 4.0, ry, wd - 8.0, row);
        let hot = b.contains(mouse);
        if hot {
            c.rect(b.x, b.y, b.w, b.h, ega(BRASS, 0.25));
        }
        c.diamond(x + 16.0, ry + row / 2.0, 3.5, ega(if x_.crime { [0.95, 0.38, 0.30] } else { BRASS_LIGHT }, if hot { 1.0 } else { 0.7 }));
        let col = if x_.crime { [0.98, 0.52, 0.42] } else { TEXT };
        c.styled(&x_.label, x + 28.0, ry + 18.0, 15.0, eg(col), Face::Body, 0.0);
        if click.is_some_and(|k| !k.right && b.contains(k.at)) {
            picked = Some(x_.act);
        }
    }
    (picked, super::squadui::Bx::new(x, y, wd, h))
}

pub struct Choice {
    pub label: String,
    pub act: Act,
    /// Doing it is a crime (red in the label and the menu).
    pub crime: bool,
}

fn ch(label: impl Into<String>, act: Act) -> Choice {
    Choice { label: label.into(), act, crime: false }
}

fn crime(label: impl Into<String>, act: Act) -> Choice {
    Choice { label: label.into(), act, crime: true }
}

fn has_lockpick(w: &World, who: &[PersonId]) -> bool {
    let pick = items::id("lockpick");
    who.iter().any(|&m| w.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| e.0 == pick)))
}

fn odds(p: f32) -> String {
    if p <= 0.0 {
        "unseen".into()
    } else {
        format!("{:.0}% seen", p * 100.0)
    }
}

/// The choices for a thing under the mouse, the left-click one first.
/// `shift` swaps a beaten foe's default to carrying them.
pub fn choices(w: &World, h: Hover, who: &[PersonId], shift: bool) -> Vec<Choice> {
    let lead = who.first().copied().unwrap_or(w.squad.members[0]);
    let mut out = Vec::new();
    match h {
        Hover::Person(pid) => {
            let p = &w.people[pid as usize];
            let name = p.name().unwrap_or("them");
            if w.squad.index(pid).is_some() {
                out.push(ch(format!("Select {name}"), Act::Select(pid)));
                if who.iter().any(|&m| m != pid && w.can_carry(m, pid)) {
                    out.push(ch(format!("Carry {name}"), Act::Carry(pid)));
                }
                return out;
            }
            if w.can_loot(pid) {
                let loot = ch(format!("Go through {name}'s things"), Act::Loot(pid));
                let carry = ch(format!("Carry {name}"), Act::Carry(pid));
                if shift {
                    out.push(carry);
                    out.push(loot);
                } else {
                    out.push(loot);
                    out.push(carry);
                }
                return out;
            }
            if who.iter().any(|&m| w.can_carry(m, pid)) {
                out.push(ch(format!("Carry {name}"), Act::Carry(pid)));
            }
            let hostile = p.bandit || w.group_of[pid as usize].and_then(|g| w.group(g)).is_some_and(|g| g.hostile);
            if hostile {
                out.push(ch(format!("Attack {name}"), Act::Attack(pid)));
            } else if !p.dead && !w.is_down(pid) {
                out.push(ch(format!("Talk to {name}"), Act::Talk(pid)));
                if w.squad_battle().is_some() {
                    out.push(ch(format!("Attack {name}"), Act::Attack(pid)));
                }
            }
        }
        Hover::Group(g) => {
            if let Some(m) = w.group(g).and_then(|g| g.members.first().copied()) {
                return choices(w, Hover::Person(m), who, shift);
            }
        }
        Hover::Door(id) | Hover::Building(id) => {
            let Some(d) = w.door(id) else { return out };
            if w.is_locked(id) {
                if has_lockpick(w, who) {
                    out.push(crime(format!("Pick the lock ({})", odds(w.catch_chance(lead, d.outside, id.0))), Act::PickLock(id)));
                } else {
                    out.push(ch("Locked (no lockpick): walk to the door", Act::Walk(d.outside)));
                }
            } else {
                out.push(ch("Go in", Act::Enter(id)));
            }
            out.push(ch("Walk to the door", Act::Walk(d.outside)));
        }
        Hover::Furniture(id, k) => {
            let Some(d) = w.door(id) else { return out };
            let Some(p) = d.variant().furniture.get(k as usize) else { return out };
            let at = d.to_world(p.at);
            if matches!(p.what, Furn::Bed | Furn::Bedroll) {
                out.push(ch(format!("Sleep in the {}", p.what.name()), Act::Sleep(at)));
            }
            out.push(ch(format!("Walk to the {}", p.what.name()), Act::Walk(at)));
        }
        Hover::Container(id) => {
            let Some(c) = w.container(id) else { return out };
            if w.container_locked(id) {
                if has_lockpick(w, who) {
                    out.push(crime(format!("Pick the {}'s lock ({})", c.what.name(), odds(w.catch_chance(lead, c.pos, id.0))), Act::PickChest(id)));
                } else {
                    out.push(ch(format!("Locked {} (no lockpick)", c.what.name()), Act::Walk(c.pos)));
                }
            } else {
                out.push(ch(format!("Open the {}", c.what.name()), Act::Search(id)));
            }
        }
        Hover::Town(t) => out.push(ch("Town panel", Act::TownPanel(t))),
        Hover::Node(n) => {
            if let Some(nd) = w.nodes.iter().find(|x| x.id == n) {
                out.push(ch(format!("Gather {}", item(nd.item).name.to_lowercase()), Act::Gather(n)));
            }
        }
        Hover::Item(g) => {
            if let Some(gi) = w.ground.iter().find(|x| x.id == g) {
                let name = item(gi.item).name.to_lowercase();
                match gi.owner {
                    Some(town) => out.push(crime(format!("Steal the {name} ({})", odds(w.catch_chance(lead, gi.pos, town))), Act::PickUp(g))),
                    None => out.push(ch(format!("Pick up the {name}"), Act::PickUp(g))),
                }
            }
        }
        Hover::Deposit(d) => {
            if let Some(dp) = w.deposit(d) {
                out.push(ch(format!("Work the {}", dp.face().name.to_lowercase()), Act::Work(d)));
                out.push(ch("Walk there", Act::Walk(dp.pos)));
            }
        }
        Hover::Station(i) => out.push(ch("Walk to it", Act::Walk(w.stations[i].0))),
        Hover::Torch(i) => {
            if let Some(t) = w.standing.get(i) {
                out.push(ch("Walk to it", Act::Walk(t.pos)));
            }
        }
        Hover::Ruin(r) => {
            if let Some(ru) = w.ruins.get(r as usize) {
                out.push(ch("Walk there", Act::Walk(ru.pos)));
            }
        }
        Hover::Camp(k) => {
            if let Some(c) = w.camps.get(k) {
                out.push(ch("Walk there (they'll see you)", Act::Walk(c.pos)));
            }
        }
        Hover::Place(t, k) => {
            if let Some(p) = w.society.towns.get(t as usize).and_then(|tl| tl.places.get(k as usize)) {
                out.push(ch(format!("Walk to the {}", p.kind.name().to_lowercase()), Act::Walk(p.pos)));
            }
        }
    }
    out.push(ch("Examine", Act::Examine));
    out
}

/// The choices for a wild animal or carcass under the mouse.
pub fn wild_choices(w: &World, s: &Seen) -> Vec<Choice> {
    let mut out = Vec::new();
    match s.thing {
        Thing::Animal(h, _) | Thing::Herd(h) => {
            let name = w.animals.herds.get(h as usize).map(|x| x.def().name).unwrap_or("them");
            out.push(ch(format!("Hunt the {name}s"), Act::Hunt(h)));
            out.push(ch("Walk there", Act::Walk(s.at)));
        }
        Thing::Carcass(c) => {
            out.push(ch("Cut it up", Act::Butcher(c)));
            out.push(ch("Walk there", Act::Walk(s.at)));
        }
        _ => out.push(ch("Walk there", Act::Walk(s.at))),
    }
    out
}

fn nearest(w: &World, who: &[PersonId], at: V2) -> Option<PersonId> {
    who.iter().copied().min_by(|&a, &b| w.person_pos(a).dist(at).total_cmp(&w.person_pos(b).dist(at)))
}

/// Carry out a choice for the selected. Returns a line to show if it
/// couldn't be done.
pub fn perform(w: &mut World, who: &[PersonId], all: bool, act: Act) -> Option<String> {
    let ok = match act {
        Act::Walk(p) => {
            if all {
                w.order_squad(p);
            } else {
                w.order_members(who, p);
            }
            true
        }
        Act::Select(_) | Act::TownPanel(_) | Act::Examine => true,
        Act::SneakTo(p) => {
            for &m in who {
                w.set_sneaking(m, true);
            }
            if all {
                w.order_squad(p);
            } else {
                w.order_members(who, p);
            }
            true
        }
        Act::Attack(p) => w.attack(who, p),
        Act::Talk(p) => {
            let lead = who.first().copied();
            lead.is_some_and(|m| w.squad_battle().is_none() && w.order_talk(m, p))
        }
        Act::Loot(p) => nearest(w, who, w.person_pos(p)).is_some_and(|m| w.order_loot(m, p)),
        Act::Carry(p) => {
            let at = w.body_pos(p);
            let carrier = who.iter().copied().filter(|&m| w.can_carry(m, p)).min_by(|&a, &b| w.person_pos(a).dist(at).total_cmp(&w.person_pos(b).dist(at)));
            carrier.is_some_and(|c| w.order_carry(c, p))
        }
        Act::Enter(id) => match w.door(id) {
            Some(d) => {
                w.order_members(who, d.centre);
                true
            }
            None => false,
        },
        Act::PickLock(id) => {
            let pick = items::id("lockpick");
            let picker = who.iter().copied().filter(|&m| w.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| e.0 == pick))).max_by(|&a, &b| w.pick_chance(a, 50.0).total_cmp(&w.pick_chance(b, 50.0)));
            picker.is_some_and(|p| w.order_pick(p, id))
        }
        Act::Search(id) => {
            let at = w.container(id).map(|c| c.pos);
            at.and_then(|at| nearest(w, who, at)).is_some_and(|m| w.order_search(m, id))
        }
        Act::PickChest(id) => {
            let pick = items::id("lockpick");
            let lock = w.container(id).map(|c| c.lock).unwrap_or(50.0);
            let picker = who.iter().copied().filter(|&m| w.people[m as usize].detail.as_ref().is_some_and(|d| d.gear.bag.iter().any(|e| e.0 == pick))).max_by(|&a, &b| w.pick_chance(a, lock).total_cmp(&w.pick_chance(b, lock)));
            picker.is_some_and(|p| w.order_pick_container(p, id))
        }
        Act::Gather(n) => {
            let at = w.nodes.iter().find(|x| x.id == n).map(|x| x.pos);
            at.and_then(|at| nearest(w, who, at)).is_some_and(|m| w.order_gather(m, n))
        }
        Act::PickUp(g) => {
            let at = w.ground.iter().find(|x| x.id == g).map(|x| x.pos);
            at.and_then(|at| nearest(w, who, at)).is_some_and(|m| w.order_pickup(m, g))
        }
        Act::Work(d) => {
            let mut any = false;
            for &m in who {
                any |= w.order_labour(m, d);
            }
            any
        }
        Act::Hunt(h) => w.order_hunt(who, h),
        Act::Butcher(c) => {
            let at = w.animals.carcasses.iter().find(|x| x.id == c).map(|x| x.pos);
            at.and_then(|at| nearest(w, who, at)).is_some_and(|m| w.order_butcher(m, c))
        }
        Act::Sleep(at) => {
            // The nearest selected goes to the bed and beds down there.
            match nearest(w, who, at) {
                Some(m) => {
                    w.order_sleep_at(m, at);
                    true
                }
                None => false,
            }
        }
    };
    (!ok).then(|| match act {
        Act::Attack(_) => "Can't attack them from here.".into(),
        Act::Talk(_) => "They won't talk now.".into(),
        Act::PickLock(_) | Act::PickChest(_) => "Nobody selected has a lockpick.".into(),
        Act::Work(_) => "Can't work it now.".into(),
        Act::Hunt(_) => "Can't go after them now.".into(),
        _ => "Can't do that.".into(),
    })
}

/// The tooltip for a thing under the mouse: what it is (the long version
/// with Alt held, for people), then what a left-click does (red for a crime)
/// and whether a right-click offers more.
pub fn tooltip(w: &World, h: Hover, who: &[PersonId], shift: bool, alt: bool) -> Vec<(String, super::palette::Rgb)> {
    use super::palette::{BRASS_LIGHT, DIM, GOLD, TEXT};
    let mut lines: Vec<(String, super::palette::Rgb)> = super::hud::describe(w, h).into_iter().filter(|(l, _)| !l.to_lowercase().contains("click")).collect();
    if let Some(first) = lines.first_mut() {
        if first.1 == TEXT {
            first.1 = GOLD;
        }
    }
    let long = lines.len() > 4 && matches!(h, Hover::Person(_) | Hover::Group(_));
    if long && !alt {
        lines.truncate(3);
    }
    // The watch on a chase, and whoever they're after.
    if let Hover::Person(p) = h {
        let warn = [0.95, 0.38, 0.30];
        if let Some(c) = w.pursuits.iter().find(|x| x.guard == p && x.pos.is_some()) {
            lines.insert(1.min(lines.len()), (format!("Chasing {} for {}!", w.name_of(c.culprit), c.wrong.name()), warn));
        }
        if let Some(g) = w.chased_by(p) {
            lines.insert(1.min(lines.len()), (format!("{} of the watch is after them", w.name_of(g)), warn));
        }
    }
    action_lines(&mut lines, choices(w, h, who, shift));
    if long && !alt {
        lines.push(("Hold Alt for more".into(), DIM));
    }
    let _ = BRASS_LIGHT;
    lines
}

/// The left-click and right-click lines added to a tooltip.
pub fn action_lines(lines: &mut Vec<(String, super::palette::Rgb)>, cs: Vec<Choice>) {
    use super::palette::{BRASS_LIGHT, DIM};
    let real: Vec<&Choice> = cs.iter().filter(|c| c.act != Act::Examine).collect();
    if let Some(c) = real.first() {
        let col = if c.crime { [0.95, 0.38, 0.30] } else { BRASS_LIGHT };
        lines.push((format!("Left-click: {}", c.label), col));
    }
    if real.len() > 1 {
        lines.push(("Right-click: more".into(), DIM));
    }
}

/// While aiming a spell: what a click on what's under the mouse would do,
/// and whether it can be done.
pub fn aim_label(w: &World, who: PersonId, s: gahturiyu_sim::sim::magic::Spell, h: Option<Hover>, scroll: bool) -> (String, bool) {
    use gahturiyu_sim::sim::magic::Aim;
    let d = s.def();
    let from = w.person_pos(who);
    let caster = w.people[who as usize].name().unwrap_or("They").to_string();
    if !scroll && w.people[who as usize].mana_at(w.time) < d.cost && !w.fighting.contains_key(&who) {
        return (format!("{caster} hasn't the energy ({:.0} needed)", d.cost), false);
    }
    let reach = |at: V2| {
        let dist = from.dist(at);
        if dist <= d.range.max(2.0) {
            format!("{:.0} m: in range", dist)
        } else {
            format!("{:.0} m: {caster} will walk closer first", dist)
        }
    };
    let person = match h {
        Some(Hover::Person(p)) => Some(p),
        Some(Hover::Group(g)) => w.group(g).and_then(|g| g.members.first().copied()),
        _ => None,
    };
    match d.aim {
        Aim::Foe => match person {
            Some(p) if w.is_enemy(p) => (format!("Cast on {}  ·  {}", w.people[p as usize].name().unwrap_or("them"), if w.fighting.contains_key(&who) { reach(w.person_pos(p)) } else { "starts the fight".into() }), true),
            Some(p) => (format!("{} isn't an enemy", w.people[p as usize].name().unwrap_or("They")), false),
            None => ("Point at an enemy".into(), false),
        },
        Aim::Friend => match person {
            Some(p) if w.people[p as usize].in_squad => (format!("Cast on {}  ·  {}", w.people[p as usize].name().unwrap_or("them"), reach(w.person_pos(p))), true),
            Some(_) => ("Only on your squad".into(), false),
            None => ("Point at one of your squad".into(), false),
        },
        Aim::Anyone => match person {
            Some(p) => (format!("Cast on {}  ·  {}", w.people[p as usize].name().unwrap_or("them"), reach(w.person_pos(p))), true),
            None => ("Point at someone".into(), false),
        },
        Aim::Door => match h {
            Some(Hover::Container(id)) => match w.container(id) {
                Some(c) if w.container_locked(id) => (format!("Open the {}'s lock  ·  {}", c.what.name().to_lowercase(), reach(c.pos)), true),
                Some(_) => ("That isn't locked".into(), false),
                None => ("Point at a lock".into(), false),
            },
            Some(Hover::Door(id)) | Some(Hover::Building(id)) => match w.door(id) {
                Some(dd) => (format!("Cast on the door  ·  {}", reach(dd.outside)), true),
                None => ("Point at a door".into(), false),
            },
            _ => ("Point at a locked door or chest".into(), false),
        },
        Aim::Corpse => ("Click by a body".into(), true),
        Aim::Point if !d.works_outside_fights() && !w.fighting.contains_key(&who) => ("Click by enemies: starts the fight".into(), true),
        Aim::Point => ("Click a spot".into(), true),
        Aim::Caster => ("Click anywhere".into(), true),
    }
}

/// A short name for anything hoverable (Alt's labels).
pub fn short_name(w: &World, h: Hover) -> Option<String> {
    Some(match h {
        Hover::Person(p) => {
            let pp = &w.people[p as usize];
            let n = pp.name().map(|s| s.to_string()).unwrap_or_else(|| pp.race.name().to_string());
            if w.can_loot(p) {
                format!("{n} (beaten)")
            } else {
                n
            }
        }
        Hover::Group(_) => return None,
        Hover::Town(t) => w.settlements.get(t as usize)?.name.clone(),
        Hover::Item(g) => item(w.ground.iter().find(|x| x.id == g)?.item).name.to_string(),
        Hover::Door(id) | Hover::Building(id) => {
            let b = &w.settlements.get(id.0 as usize)?.buildings[id.1 as usize];
            gahturiyu_sim::sim::layout::variant_of(b).map(|v| v.name.to_string()).unwrap_or_else(|| "Building".into())
        }
        Hover::Node(n) => item(w.nodes.iter().find(|x| x.id == n)?.item).name.to_string(),
        Hover::Station(i) => w.stations.get(i)?.1.name().to_string(),
        Hover::Torch(_) => "Standing torch".into(),
        Hover::Deposit(d) => w.deposit(d)?.face().name.to_string(),
        Hover::Ruin(r) => w.ruins.get(r as usize)?.name(w),
        Hover::Container(id) => {
            let c = w.container(id)?;
            if w.container_locked(id) {
                format!("{} (locked)", c.what.name())
            } else {
                c.what.name().to_string()
            }
        }
        Hover::Furniture(id, k) => w.door(id)?.variant().furniture.get(k as usize)?.what.name().to_string(),
        Hover::Camp(_) => "Bandit camp".into(),
        Hover::Place(t, k) => w.society.towns.get(t as usize)?.places.get(k as usize)?.kind.name().to_string(),
    })
}

/// What's on the ground under the mouse when nothing smaller is: a piece of
/// furniture, a building, a camp, a workplace.
pub fn ground_hover(w: &World, at: V2) -> Option<Hover> {
    for d in w.doors_near(at, 2.0) {
        if !d.contains(at) {
            continue;
        }
        // Furniture shows (and counts) only inside, where it's drawn.
        let inside = w.squad.inside.iter().flatten().any(|&i| i == d.id);
        if inside {
            for (k, p) in d.variant().furniture.iter().enumerate() {
                let (wd, dp, _) = p.what.size();
                if d.to_world(p.at).dist(at) < (wd.max(dp) * 0.6).max(0.6) {
                    return Some(Hover::Furniture(d.id, k as u8));
                }
            }
        }
        return Some(Hover::Building(d.id));
    }
    if let Some(k) = w.camps.iter().position(|c| c.pos.dist(at) < 14.0) {
        return Some(Hover::Camp(k));
    }
    for st in w.settlements.iter().filter(|s| s.pos.dist(at) < s.radius() + 500.0) {
        if let Some(tl) = w.society.towns.get(st.id as usize) {
            if let Some(k) = tl.places.iter().position(|p| p.pos.dist(at) < p.kind.size() * 0.5) {
                return Some(Hover::Place(st.id, k as u16));
            }
        }
    }
    None
}
