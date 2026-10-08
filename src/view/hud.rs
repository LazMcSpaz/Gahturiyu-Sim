//! Text and panels drawn over the view: the side panel, hover descriptions,
//! health bars and the help line.
//!
//! Everything flat is drawn with egui's painter through `Canvas`, a thin
//! layer that keeps the old window's coordinates (pixels from the top left,
//! text placed by its baseline) so the layouts carried over unchanged.

use bevy::math::Vec2;
use bevy_egui::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind};

use gahturiyu_sim::sim::{
    body::{self, Part, PARTS},
    group::Kind,
    items,
    person::PersonId,
    race::{trait_word, ALL_RACES},
    stats::{Attr, Skill, SKILLS},
    world::HOUR,
    World,
};

use super::app::Hover;
use super::palette::{self, eg, ega, race_color, Rgb, DIM, GOLD, SNEAK, TEXT, WARN};

pub const SPEEDS: [(f64, &str); 5] = [(1.0, "1×"), (10.0, "10×"), (60.0, "1 min/s"), (600.0, "10 min/s"), (3600.0, "1 hour/s")];
pub const PANEL: Color32 = Color32::from_rgba_premultiplied(4, 6, 6, 219);

pub struct Canvas {
    pub p: egui::Painter,
    pub w: f32,
    pub h: f32,
}

pub fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(Pos2::new(x, y), egui::vec2(w, h))
}

fn font(size: f32) -> FontId {
    FontId::proportional(size * 0.92)
}

impl Canvas {
    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color32) {
        self.p.rect_filled(r(x, y, w, h), 0.0, c);
    }
    pub fn rect_lines(&self, x: f32, y: f32, w: f32, h: f32, t: f32, c: Color32) {
        self.p.rect_stroke(r(x, y, w, h), 0.0, Stroke::new(t, c), StrokeKind::Inside);
    }
    /// Text with its baseline at `y`.
    pub fn text(&self, s: &str, x: f32, y: f32, size: f32, c: Rgb) {
        self.text_c(s, x, y, size, eg(c));
    }
    pub fn text_c(&self, s: &str, x: f32, y: f32, size: f32, c: Color32) {
        self.p.text(Pos2::new(x, y - size * 0.86), Align2::LEFT_TOP, s, font(size), c);
    }
    pub fn width(&self, s: &str, size: f32) -> f32 {
        self.p.layout_no_wrap(s.to_string(), font(size), Color32::WHITE).size().x
    }
    pub fn centred(&self, s: &str, x: f32, y: f32, size: f32, c: Rgb) {
        let w = self.width(s, size);
        self.text_c(s, x - w / 2.0 + 1.0, y + 1.0, size, Color32::from_black_alpha(150));
        self.text(s, x - w / 2.0, y, size, c);
    }
    /// A dark box of lines; returns where it went.
    pub fn panel(&self, lines: &[(String, Rgb)], x: f32, y: f32, size: f32) -> Rect {
        let lh = size * 1.35;
        let w = lines.iter().map(|(l, _)| self.width(l, size)).fold(0.0, f32::max) + 24.0;
        let h = lines.len() as f32 * lh + 16.0;
        let x = x.min(self.w - w - 8.0).max(8.0);
        let y = y.min(self.h - h - 8.0).max(8.0);
        // Nearly opaque: tooltips often sit over other panels.
        self.rect(x, y, w, h, Color32::from_rgba_premultiplied(4, 6, 6, 248));
        self.rect_lines(x, y, w, h, 1.0, Color32::from_rgba_premultiplied(60, 60, 60, 200));
        for (i, (l, c)) in lines.iter().enumerate() {
            self.text(l, x + 12.0, y + 8.0 + lh * (i as f32 + 0.78), size, *c);
        }
        r(x, y, w, h)
    }
    pub fn circle(&self, x: f32, y: f32, rad: f32, c: Color32) {
        self.p.circle_filled(Pos2::new(x, y), rad, c);
    }
    pub fn circle_lines(&self, x: f32, y: f32, rad: f32, t: f32, c: Color32) {
        self.p.circle_stroke(Pos2::new(x, y), rad, Stroke::new(t, c));
    }
    pub fn line(&self, a: Vec2, b: Vec2, t: f32, c: Color32) {
        self.p.line_segment([Pos2::new(a.x, a.y), Pos2::new(b.x, b.y)], Stroke::new(t, c));
    }
    pub fn triangle(&self, a: Vec2, b: Vec2, c: Vec2, col: Color32) {
        self.p.add(Shape::convex_polygon(vec![Pos2::new(a.x, a.y), Pos2::new(b.x, b.y), Pos2::new(c.x, c.y)], col, Stroke::NONE));
    }
    pub fn ellipse_lines(&self, x: f32, y: f32, rx: f32, ry: f32, t: f32, c: Color32) {
        let pts: Vec<Pos2> = (0..24).map(|i| {
            let a = i as f32 / 24.0 * std::f32::consts::TAU;
            Pos2::new(x + a.cos() * rx, y + a.sin() * ry)
        }).collect();
        self.p.add(Shape::closed_line(pts, Stroke::new(t, c)));
    }
}

pub fn draw_hud(c: &Canvas, w: &World, speed_i: usize, paused: bool, sim_ms: f64, frame_ms: f64, view: &str) -> Rect {
    let mut lines: Vec<(String, Rgb)> = vec![
        (w.clock(), TEXT),
        (if paused { "Paused".into() } else { format!("Speed {}", SPEEDS[speed_i].1) }, DIM),
        (String::new(), TEXT),
        (format!("{} people  ·  {} on the road", w.people.len() - w.squad.members.len(), w.on_road()), TEXT),
        (format!("Named so far: {}", w.stats.detailed), TEXT),
        (format!("Light here: {}  ·  ambushes so far: {}", gahturiyu_sim::sim::stealth::light_word(w.light_at(w.squad.pos)), w.stats.ambushes), TEXT),
        (format!("Groups in band 1 / 2 / 3:  {} / {} / {}", w.stats.in_band[1], w.stats.in_band[2], w.stats.in_band[3]), TEXT),
        (format!("Simulation {:.2} ms  ·  frame {:.1} ms  ·  {:.0} fps", sim_ms, frame_ms, 1000.0 / frame_ms.max(0.1)), DIM),
        (format!("View: {view}  (V to switch)"), DIM),
        (String::new(), TEXT),
    ];
    for r in ALL_RACES {
        lines.push((format!("●  {}  ({})", r.name(), r.element()), race_color(r)));
    }
    for (town, b) in &w.bounty {
        let heard = w.towns_heard(*town);
        let spread = if heard > 1 { format!("  ·  word has reached {} towns", heard) } else { String::new() };
        lines.push((format!("Bounty in {}: {:.0}{spread}", w.settlements[*town as usize].name, b), [0.95, 0.45, 0.35]));
    }
    let fight = battle_lines(w);
    if !fight.is_empty() {
        lines.push((String::new(), TEXT));
        lines.extend(fight);
    }
    lines.push((String::new(), TEXT));
    for (t, l) in w.log.iter().take(8) {
        lines.push((format!("{}  {}", hhmm(*t), l), DIM));
    }
    c.panel(&lines, 12.0, 12.0, 16.0)
}

pub fn hhmm(secs: f64) -> String {
    let s = secs.rem_euclid(24.0 * HOUR);
    format!("{:02}:{:02}", (s / HOUR) as i64, ((s % HOUR) / 60.0) as i64)
}

pub fn describe(w: &World, h: Hover) -> Vec<(String, Rgb)> {
    let mut out: Vec<(String, Rgb)> = Vec::new();
    match h {
        Hover::Person(pid) => {
            let p = &w.people[pid as usize];
            let name = p.name().unwrap_or("(not yet named)");
            out.push((format!("{}  ·  {}", name, p.race.name()), race_color(p.race)));
            let t = p.traits;
            out.push((
                format!(
                    "{}, {}, {}, {}",
                    trait_word(t.wanderlust, "a homebody", "settled", "restless"),
                    trait_word(t.boldness, "cautious", "steady", "bold"),
                    trait_word(t.sociability, "solitary", "easygoing", "gregarious"),
                    trait_word(t.patience, "hot-tempered", "even-tempered", "patient"),
                ),
                TEXT,
            ));
            let st = p.effective_stats();
            out.push((
                format!(
                    "{}  ·  STR {:.0}  AGI {:.0}  TOU {:.0}  INT {:.0}  WIL {:.0}",
                    st.calling.name(),
                    st.attr(Attr::Strength),
                    st.attr(Attr::Agility),
                    st.attr(Attr::Toughness),
                    st.attr(Attr::Intellect),
                    st.attr(Attr::Willpower)
                ),
                TEXT,
            ));
            let mut best: Vec<(Skill, f32)> = SKILLS.iter().map(|&k| (k, st.skill(k))).collect();
            best.sort_by(|a, b| b.1.total_cmp(&a.1));
            out.push((best.iter().take(4).map(|(k, v)| format!("{} {:.0}", k.name(), v)).collect::<Vec<_>>().join("  ·  "), DIM));
            if let Some(d) = &p.detail {
                let worn: Vec<&str> = d.gear.equipped().filter(|&id| items::item(id).slot != items::Slot::MainHand).map(|id| items::item(id).name).collect();
                out.push((format!("{}  ·  {}", d.gear.weapon_name(), worn.join(", ")), TEXT));
                if !d.spells.is_empty() {
                    out.push((format!("Spells: {}", d.spells.iter().map(|s| s.def().name).collect::<Vec<_>>().join(", ")), [0.65, 0.75, 1.0]));
                }
            }
            let (hp, mana, statuses) = match w.fighter(pid) {
                Some(f) => (f.hp, f.mana, f.statuses.iter().map(|s| format!("{:?}", s.does)).collect::<Vec<_>>()),
                None => (p.wounds.hp_at(&p.stats, w.time), p.mana_at(w.time), vec![]),
            };
            let missing = w.fighter(pid).map(|f| f.missing).unwrap_or(p.wounds.missing);
            let parts = PARTS
                .iter()
                .enumerate()
                .map(|(i, part)| if missing[i] { format!("{} lost", short_part(*part)) } else { format!("{} {:.0}/{:.0}", short_part(*part), hp[i], p.stats.max_hp(*part)) })
                .collect::<Vec<_>>()
                .join("  ");
            let ko = body::knocked_out(&hp);
            out.push((parts, if p.dead { [0.9, 0.3, 0.3] } else if ko { [0.95, 0.6, 0.3] } else { DIM }));
            let gone: Vec<&str> = PARTS.iter().enumerate().filter(|(i, _)| missing[*i]).map(|(_, p)| p.name()).collect();
            if !gone.is_empty() {
                out.push((format!("Lost for good: {}", gone.join(", ")), [0.9, 0.35, 0.3]));
            }
            out.push((
                format!(
                    "Mana {:.0}/{:.0}  ·  Might {:.0}{}{}",
                    mana,
                    p.max_mana(),
                    p.might,
                    if p.dead { "  ·  DEAD" } else if ko { "  ·  down" } else { "" },
                    if statuses.is_empty() { String::new() } else { format!("  ·  {}", statuses.join(", ")) }
                ),
                DIM,
            ));
            if p.bandit {
                let state = match w.group_of[pid as usize] {
                    _ if w.fighter(pid).is_some() => "fighting",
                    Some(g) if w.has_noticed(g) => "has seen you",
                    Some(g) if w.suspicion.iter().any(|((x, _), v)| *x == g && *v > 0.3) => "suspicious",
                    _ => "unaware of you",
                };
                out.push((format!("Bandit  ·  {state}  ·  click to attack"), [0.95, 0.35, 0.3]));
            }
            if let Some(c) = w.carrying(pid) {
                out.push((format!("Carrying {}", w.people[c as usize].name().unwrap_or("someone")), TEXT));
            }
            if let Some(c) = w.carried_by(pid) {
                out.push((format!("Carried by {}", w.people[c as usize].name().unwrap_or("someone")), TEXT));
            } else if body::knocked_out(&hp) && !p.dead {
                out.push(("Down. Select someone and click to carry them.".into(), DIM));
            }
            if let Some(hrs) = w.torch_hours_left(pid) {
                let lit = if w.torch_lit(pid) { "lit" } else { "out" };
                out.push((format!("Torch {lit}  ·  {:.1} h left  ·  T to light or put out", hrs), GOLD));
            }
            if p.in_squad && w.is_sneaking(pid) {
                out.push((format!("Sneaking  ·  noise {:.1}  ·  visibility {:.0}%", w.noise_of(pid), w.visibility_of(pid) * 100.0), SNEAK));
            }
            if p.in_squad {
                out.push(("Your squad".into(), TEXT));
            } else {
                out.extend(work_lines(w, pid));
                if let Some(g) = w.group_of[pid as usize].and_then(|g| w.group(g)) {
                    out.push((doing(w, g.id), DIM));
                } else if let Some(home) = p.home {
                    match w.doing_now(pid) {
                        Some((d, spot)) => out.push((format!("{} — {}", capital(d.word()), where_word(w, pid, spot, home)), GOLD)),
                        None => out.push((format!("At home in {}", w.settlements[home as usize].name), DIM)),
                    }
                }
            }
        }
        Hover::Group(gid) => {
            let Some(g) = w.group(gid) else { return out };
            let mut by = [0usize; 4];
            for &m in &g.members {
                by[w.people[m as usize].race.index()] += 1;
            }
            let mix: Vec<String> = ALL_RACES.iter().zip(by).filter(|(_, n)| *n > 0).map(|(r, n)| format!("{} {}", r.name(), n)).collect();
            let who = if g.members.len() == 1 { "1 traveller".to_string() } else { format!("{} travellers", g.members.len()) };
            out.push((format!("{who}  ({})", mix.join(", ")), race_color(w.people[g.members[0] as usize].race)));
            out.push((doing(w, gid), TEXT));
            let might: f32 = g.members.iter().map(|&m| w.people[m as usize].might).sum();
            out.push((format!("Combined might {:.0}", might), DIM));
            let named = g.members.iter().filter(|&&m| w.people[m as usize].detail.is_some()).count();
            out.push((
                format!("Band {}  ·  {}", g.band, if named == 0 { "nobody here has been named yet".to_string() } else { format!("{named} of {} named", g.members.len()) }),
                DIM,
            ));
        }
        Hover::Item(gid) => {
            let Some(g) = w.ground.iter().find(|g| g.id == gid) else { return out };
            out = super::squadui::item_lines(g.item);
            if g.count > 1 {
                out[0].0 = format!("{}  ×{}", out[0].0, g.count);
            }
            out.push(("Click to pick up".into(), TEXT));
        }
        Hover::Door(id) => {
            let s = &w.settlements[id.0 as usize];
            let Some(d) = gahturiyu_sim::sim::buildings::door_of(s, id.1) else { return out };
            use gahturiyu_sim::sim::settlement::BuildingKind as B;
            let kind = match s.buildings[id.1 as usize].kind {
                B::RoduroHome => "A Roduro home",
                B::QotiroBlock => "Qotiro living quarters",
                B::QotiroTemple => "The temple-fortress",
                B::QotiroHall => "The Qotiro hall",
                _ => "A building",
            };
            out.push((format!("{kind}  ·  {}", s.name), TEXT));
            if d.lock <= 0.0 {
                out.push(("No lock.  Click to go in.".into(), DIM));
            } else if w.is_locked(id) {
                out.push((format!("Locked for the night  ·  lock {:.0}", d.lock), WARN));
                out.push(("Click to pick the lock (needs a lockpick).".into(), DIM));
            } else {
                out.push((format!("Open  ·  lock {:.0}, locked 20:00–06:00", d.lock), DIM));
                out.push(("Click to go in.".into(), DIM));
            }
        }
        Hover::Node(id) => {
            let Some(n) = w.nodes.iter().find(|n| n.id == id) else { return out };
            out.push((format!("{}  ×{}", items::item(n.item).name, n.amount), GOLD));
            if n.ready(w.time) {
                out.push(("Click to gather".into(), TEXT));
            } else {
                out.push((format!("Picked; grows back at {}", hhmm(n.picked_at.unwrap_or(0.0) + 24.0 * HOUR)), DIM));
            }
        }
        Hover::Station(i) => {
            let (_, st) = w.stations[i];
            use gahturiyu_sim::sim::crafting::Station as S;
            out.push((st.name().to_string(), GOLD));
            let what = match st {
                S::Forge => "Smithing: weapons, smelting ore; Armoring: iron pieces",
                S::Bench => "Armoring: tanning, leather and hide pieces, bucklers",
                S::Desk => "Inscription: scrolls",
                S::AlchemyTable => "Alchemy: potions (a mortar and pestle works anywhere)",
            };
            out.push((what.into(), TEXT));
            out.push(("Stand by it and press K".into(), DIM));
        }
        Hover::Town(sid) => {
            let s = &w.settlements[sid as usize];
            out.push((s.name.clone(), race_color(s.founders)));
            out.push((
                format!("{} town, founded by the {}{}", if s.coastal { "Coastal" } else { "Inland" }, s.founders.name(), if s.coastal { ", Horaro stilts offshore" } else { "" }),
                TEXT,
            ));
            let mut by = [0usize; 4];
            let mut home = 0;
            for &p in &s.residents {
                by[w.people[p as usize].race.index()] += 1;
                if w.busy_until[p as usize] <= w.time {
                    home += 1;
                }
            }
            out.push((format!("{} people, {} home right now", s.residents.len(), home), TEXT));
            for (r, n) in ALL_RACES.iter().zip(by) {
                out.push((format!("   {} {}", r.name(), n), race_color(*r)));
            }
            if let Some(tl) = w.society.towns.get(sid as usize) {
                let k = w.society.communities[tl.shore as usize].customs;
                out.push((format!("{}  ·  {}", k.cooking.name(), k.rhythm.name()), TEXT));
                out.push((format!("Fed {:.0}% yesterday  ·  click for the town panel", w.town_food(sid) * 100.0), DIM));
            }
        }
        Hover::Torch(i) => {
            let Some(st) = w.standing.get(i) else { return out };
            out.push(("Standing torch".into(), GOLD));
            out.push((format!("Burns until {}", hhmm(st.out_at)), DIM));
        }
    }
    out
}

pub fn doing(w: &World, gid: u32) -> String {
    let Some(g) = w.group(gid) else { return String::new() };
    let t = w.time;
    let Some(leg) = g.current_leg(t) else { return String::new() };
    let at = |s: u16| w.settlements[s as usize].name.clone();
    match g.kind {
        Kind::Wanderer { .. } => {
            if t < leg.arrive && t >= leg.depart {
                "Wandering the wilds".into()
            } else {
                "Camped in the wilds".into()
            }
        }
        Kind::Journey { home } => {
            if t < leg.depart {
                format!("Setting out from {} at {}", at(home), hhmm(leg.depart))
            } else if t < leg.arrive {
                match leg.dest {
                    Some(d) if d == home => format!("Walking home to {}, due {}", at(d), hhmm(leg.arrive)),
                    Some(d) => format!("Walking to {}, due {}", at(d), hhmm(leg.arrive)),
                    None => "On the road".into(),
                }
            } else {
                match leg.dest {
                    Some(d) => format!("Visiting {}", at(d)),
                    None => "Resting".into(),
                }
            }
        }
    }
}

fn short_part(p: Part) -> &'static str {
    match p {
        Part::Head => "Hd",
        Part::Torso => "To",
        Part::LeftArm => "LA",
        Part::RightArm => "RA",
        Part::LeftLeg => "LL",
        Part::RightLeg => "RL",
    }
}

/// Health (vital share), mana share and whether down, for anyone worth a bar:
/// in a fight, or carrying wounds.
pub fn bar_for(w: &World, pid: PersonId) -> Option<(f32, Option<f32>, bool)> {
    let p = w.people.get(pid as usize)?;
    if p.dead {
        return None;
    }
    if let Some(f) = w.fighter(pid) {
        let mana = if f.spells.is_empty() { None } else { Some(f.mana / f.max_mana.max(1.0)) };
        return Some((f.vitality(), mana, f.ko));
    }
    if !p.wounds.is_hurt(w.time) {
        return None;
    }
    let hp = p.wounds.hp_at(&p.stats, w.time);
    let vit = (hp[0].max(0.0) / p.stats.max_hp(Part::Head)).min(hp[1].max(0.0) / p.stats.max_hp(Part::Torso));
    Some((vit, None, body::knocked_out(&hp)))
}

pub fn health_color(vit: f32, down: bool) -> Color32 {
    if down {
        eg([0.9, 0.55, 0.2])
    } else {
        eg([0.85 - vit * 0.6, 0.25 + vit * 0.6, 0.25])
    }
}

/// A small health bar (and mana bar under it) centred at x, sitting at y.
pub fn draw_bar(c: &Canvas, x: f32, y: f32, vit: f32, mana: Option<f32>, down: bool) {
    let w = 30.0;
    c.rect(x - w / 2.0 - 1.0, y - 1.0, w + 2.0, 6.0, Color32::from_black_alpha(178));
    c.rect(x - w / 2.0, y, w * vit.clamp(0.0, 1.0), 4.0, health_color(vit, down));
    if let Some(m) = mana {
        c.rect(x - w / 2.0 - 1.0, y + 5.0, w + 2.0, 4.0, Color32::from_black_alpha(178));
        c.rect(x - w / 2.0, y + 6.0, w * m.clamp(0.0, 1.0), 2.0, eg(palette::MANA));
    }
}

/// The squad's fight, as a few lines for the side panel.
pub fn battle_lines(w: &World) -> Vec<(String, Rgb)> {
    let Some(b) = w.squad_battle() else { return vec![] };
    let mut out = vec![(format!("FIGHT  ·  {:.0}s", b.time - b.start), [0.95, 0.45, 0.35])];
    for (_, l) in b.log.iter().rev().take(7).collect::<Vec<_>>().into_iter().rev() {
        out.push((l.clone(), DIM));
    }
    out
}

pub fn shadow(a: f32) -> Color32 {
    ega([0.0, 0.0, 0.0], a)
}


fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Someone's work: what they do, where, and the hours they keep.
fn work_lines(w: &World, pid: PersonId) -> Vec<(String, Rgb)> {
    use gahturiyu_sim::sim::jobs::Job;
    let l = w.life(pid);
    if l.job == Job::None {
        return vec![];
    }
    let mut line = l.job.name().to_string();
    if l.job == Job::Guard && l.shift == 1 {
        line += " (night watch)";
    }
    if let Some(wp) = w.workplace_of(pid) {
        line += &format!("  ·  {}", wp.kind.name());
    }
    if l.job == Job::StoneTender {
        line += "  ·  rounds of the homes";
    }
    let mut extra = Vec::new();
    if let Some(r) = l.habits.own_rhythm.filter(|&r| w.community_of(pid).map(|c| c.customs.rhythm != r).unwrap_or(true)) {
        extra.push(format!("keeps {} whatever the town does", r.name().to_lowercase()));
    }
    if l.habits.lodger {
        extra.push("lodges with a host household".to_string());
    }
    if w.gardener_of(pid).is_some() {
        extra.push("tends the garden".to_string());
    }
    let mut out = vec![(line, TEXT)];
    if !extra.is_empty() {
        out.push((capital(&extra.join(", ")), DIM));
    }
    out
}

fn where_word(w: &World, pid: PersonId, spot: gahturiyu_sim::sim::routine::Spot, home: u16) -> String {
    use gahturiyu_sim::sim::routine::Spot;
    let town = &w.settlements[home as usize].name;
    match spot {
        Spot::Home => format!("home in {town}"),
        Spot::Work => w.workplace_of(pid).map(|p| format!("the {}", p.kind.name().to_lowercase())).unwrap_or_else(|| town.clone()),
        Spot::Place(i) => format!("the {}", w.society.towns[home as usize].places[i as usize].kind.name().to_lowercase()),
        Spot::Hearth => "by the hearth".into(),
        Spot::Visit(_) => "at a neighbour's".into(),
        Spot::Garden => "the garden".into(),
        Spot::Rounds => "going from home to home".into(),
        Spot::Boat => "on the dawn boat".into(),
        Spot::RunRound => "out to the fields and yards".into(),
    }
}
