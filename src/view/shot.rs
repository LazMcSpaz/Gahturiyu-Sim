//! Development aid: headless screenshots, set up by environment variables.
//!
//! `GAHT_SHOT=out.png GAHT_FRAMES=120 cargo run` saves a screenshot after that
//! many frames and quits. The other `GAHT_` flags set the scene up first (see
//! CLAUDE.md for the list).

use bevy::math::{vec2, Vec2};

use gahturiyu_sim::sim::{
    body::{self, Part},
    dialogue::Topic,
    geo::V2,
    items::{self, Kind, Slot},
    World,
};

pub struct Shot {
    pub path: String,
    pub frames: u32,
    pub hover: Option<Vec2>,
    /// `GAHT_DRAG=x,y`: a selection box held from there to the `GAHT_HOVER` point.
    pub drag: Option<Vec2>,
    pub zoom: Option<f32>,
    pub speed: Option<usize>,
    pub view: Option<String>,
    pub pitch: Option<f32>,
    pub yaw: Option<f32>,
    pub nudge: Option<(f32, f32)>,
    /// `GAHT_BANDITS=n`: start with n bandits right next to the squad.
    pub bandits: Option<usize>,
    /// `GAHT_CAMP=k`: start the squad 70 m from bandit camp k.
    pub camp: Option<usize>,
    /// `GAHT_FORGE=1`: start the squad at the forged town (between its
    /// landing and the Eldest Home's site).
    pub forge: bool,
    /// `GAHT_WAIT=h`: run until a fight is on nearby, at most h game hours.
    pub wait: Option<f64>,
    /// `GAHT_HOURS=h`: run h game hours first (coarsely).
    pub hours: Option<f64>,
    /// `GAHT_SNEAK=1`: the squad starts sneaking.
    pub sneak: bool,
    /// `GAHT_ENTER=1`: the first squad member walks into the nearest home.
    pub enter: bool,
    /// `GAHT_CRAFT=k`: open squad member k's crafting panel.
    pub craft: Option<usize>,
    /// `GAHT_MAKE=forge|bench|loom|workbench|desk|bed|table|campfire|stove|wild`: the squad at
    /// the nearest such station with things to make and the craft for it
    /// (member 0 practised, member 1 new to it and carrying some of the
    /// materials, member 2 a little way off), its making screen open.
    pub make: Option<String>,
    /// `GAHT_DRINK=1`: the squad in drink and smoke: member 0 very drunk, 1 drunk, 2
    /// tipsy and hazy from a pipe, 3 hungover (the meters on the squad list, the sway, the puff).
    pub drink: bool,
    /// `GAHT_FLASK=1`: three bandits bunched near the squad, the first of
    /// them with a slumber brew and a flask; held the moment it bursts.
    pub flask: bool,
    /// `GAHT_TALK=1`: talk to the nearest townsperson (`roduro`, `qotiro`,
    /// `horaro` or `tadoro`: the nearest of that people; `merchant`,
    /// `official`, `healer`...: the nearest with that job; `indoors`: the
    /// nearest who is inside a building).
    pub talk: bool,
    pub talk_to: Option<String>,
    /// `GAHT_BYE=1`: with `GAHT_TALK`, the goodbye is said: their parting
    /// words, held on the screen.
    pub bye: bool,
    /// `GAHT_STARVE=1`: the squad is starving (no food, hunger 92).
    pub starve: bool,
    /// `GAHT_REST=1`: the squad beds down where it stands (after
    /// `GAHT_HOURS`), so the hunter's tent goes up.
    pub rest: bool,
    /// `GAHT_EXHAUST=1`: the squad is exhausted and out of breath.
    pub exhaust: bool,
    /// `GAHT_CARRY=1`: member 2 is down and member 0 is carrying them.
    pub carry: bool,
    /// `GAHT_LIMB=1`: member 0 has lost their left arm.
    pub limb: bool,
    /// `GAHT_RANGED=1`: bandit archers open up from 25 m away.
    pub ranged: bool,
    /// `GAHT_TORCH=1`: members 0 and 1 light torches, and a standing torch
    /// is set down.
    pub torch: bool,
    /// `GAHT_DEBUG=1`: show the detail-level readout.
    pub debug: bool,
    /// `GAHT_FOREST=1`: point the camera at the nearest big wood.
    pub forest: bool,
    /// `GAHT_SELECT=k`: select squad member k.
    pub select: Option<usize>,
    /// `GAHT_INV=k`: open squad member k's pack.
    pub inventory: Option<usize>,
    /// `GAHT_DROP=k`: squad member k drops a few things.
    pub drop: Option<usize>,
    /// `GAHT_BOOK=k`: open squad member k's spell book.
    pub book: Option<usize>,
    /// `GAHT_SUMMON=1`: a fight in the open where the squad's mage calls up a
    /// spirit beast and raises a fallen bandit as a thrall.
    pub summon: bool,
    /// `GAHT_HELD=1`: the squad's mage holds a Restore ritual ready.
    pub held: bool,
    /// `GAHT_TOWN=1`: open the town panel for the nearest town;
    /// `GAHT_TOWN=roduro|qotiro|horaro|mixed`: go to the town most of that
    /// people (or the most mixed) first.
    pub town: bool,
    pub town_kind: Option<String>,
    /// `GAHT_BUILD=1`: a demo outpost in the wilds near the start (a hut and
    /// lean-to up, a palisade with a gate, more under way), the Build panel
    /// open and a hut's ghost on the cursor.
    pub build: Option<String>,
    /// `GAHT_LOOT=1`: two bandits lie beaten beside the squad and member 0
    /// is going through the first one's things (the loot panel).
    pub loot: bool,
    /// `GAHT_CHASE=1`: mid-morning in a town with its watch out, member 0 is
    /// seen stealing, it's told, and the guard is on their way.
    pub chase: bool,
    /// `GAHT_RUIN=k`: midday, the squad 45 m from ruin (or lair) k.
    pub ruin: Option<usize>,
    /// `GAHT_GRIND=1|mine`: two of the squad at work at the nearest town
    /// woodlot (or iron seam).
    pub grind: Option<String>,
    /// `GAHT_RECRUIT=n`: n willing townsfolk from the nearest town join the
    /// squad (the squad's purse covers their fees); the last is asked in
    /// conversation, which stays open.
    pub recruit: Option<usize>,
    /// `GAHT_DUEL=1`: squad member 0 is judged by duel in the nearest town.
    pub duel: bool,
    /// `GAHT_SHUN=1`: the nearest coastal town's stilt village withdraws;
    /// the world runs on past the next dawn.
    pub shun: bool,
    /// `GAHT_TRADE=1`: talk to the nearest merchant at work and look at their wares.
    pub trade: bool,
    /// `GAHT_TRADE=table`: the same, with a few things laid on the table.
    pub trade_table: bool,
    /// `GAHT_CONVO=1`: a local who was robbed last night talks about it
    /// (a conversation put together from the dialogue pieces).
    pub convo: bool,
    /// `GAHT_GUARD=1`: squad member 0 on a guard contract, standing at a
    /// merchant's stall in its hours.
    pub guard: bool,
    /// `GAHT_FACTIONS=1`: the nearest town held by the Banner Lords, a few
    /// deeds done for and against factions, and the journal open on Standing.
    pub factions: bool,
    /// `GAHT_FEUD=1`: two households in the nearest town fall out and the
    /// world runs on; the town panel's debug readout shows how it went.
    pub feud: bool,
    /// `GAHT_EDIT=1`: the land editor open, with a few strokes of each brush
    /// made beside the squad (raised and terraced ground, painted snow, sand
    /// and mud, more trees, rocks).
    pub edit: bool,
    /// `GAHT_SOCIETY=runners|boats|tides`: go and watch the midday meal run,
    /// the dawn boats, or a stilt village keeping tide hours.
    pub society: Option<String>,
    /// `GAHT_INTERIOR=1`: member 0 walks into the nearest open building with
    /// containers (a workshop or shop if there is one) and opens one;
    /// `tadoro` / `crowded` / a variant's key pick the building differently
    /// (`walk_in`).
    pub interior: bool,
    /// `GAHT_VARIANTS=1|roduro|qotiro|horaro[,open]`: every building variant
    /// (or one people's) laid out side by side on open ground near the squad.
    pub variants: Option<String>,
    /// `GAHT_CUTAWAY=1` (or `open` in `GAHT_VARIANTS`): every building near
    /// the camera drawn cut open.
    pub cutaway: bool,
    /// `GAHT_SIGNS=1|roduro|qotiro`: frame the street in the nearest town
    /// (or the most Roduro / Qotiro one) with the most trade signs together.
    pub signs: Option<String>,
    /// Where the camera should look for the scene set up (target, distance,
    /// pitch, yaw), filled in by `prepare`.
    pub framing: std::sync::Mutex<Option<(V2, f32, f32, Option<f32>)>>,
}

impl Shot {
    pub fn from_env() -> Option<Shot> {
        let path = std::env::var("GAHT_SHOT").ok()?;
        let var = |k: &str| std::env::var(k).ok();
        let pair = |k: &str| {
            var(k).and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some((x.trim().parse::<f32>().ok()?, y.trim().parse::<f32>().ok()?))
            })
        };
        Some(Shot {
            path,
            frames: var("GAHT_FRAMES").and_then(|v| v.parse().ok()).unwrap_or(120),
            hover: pair("GAHT_HOVER").map(|(x, y)| vec2(x, y)),
            drag: pair("GAHT_DRAG").map(|(x, y)| vec2(x, y)),
            zoom: var("GAHT_ZOOM").and_then(|v| v.parse().ok()),
            speed: var("GAHT_SPEED").and_then(|v| v.parse().ok()),
            view: var("GAHT_VIEW"),
            pitch: var("GAHT_PITCH").and_then(|v| v.parse().ok()),
            yaw: var("GAHT_YAW").and_then(|v| v.parse().ok()),
            bandits: var("GAHT_BANDITS").and_then(|v| v.parse().ok()),
            camp: var("GAHT_CAMP").and_then(|v| v.parse().ok()),
            forge: var("GAHT_FORGE").is_some(),
            wait: var("GAHT_WAIT").and_then(|v| v.parse().ok()),
            hours: var("GAHT_HOURS").and_then(|v| v.parse().ok()),
            sneak: var("GAHT_SNEAK").is_some(),
            enter: var("GAHT_ENTER").is_some(),
            make: var("GAHT_MAKE"),
            drink: var("GAHT_DRINK").is_some(),
            flask: var("GAHT_FLASK").is_some(),
            craft: var("GAHT_CRAFT").and_then(|v| v.parse().ok()),
            talk: var("GAHT_TALK").is_some(),
            talk_to: var("GAHT_TALK").filter(|v| v != "1"),
            bye: var("GAHT_BYE").is_some(),
            starve: var("GAHT_STARVE").is_some(),
            rest: var("GAHT_REST").is_some(),
            exhaust: var("GAHT_EXHAUST").is_some(),
            carry: var("GAHT_CARRY").is_some(),
            limb: var("GAHT_LIMB").is_some(),
            ranged: var("GAHT_RANGED").is_some(),
            torch: var("GAHT_TORCH").is_some(),
            debug: var("GAHT_DEBUG").is_some(),
            forest: var("GAHT_FOREST").is_some(),
            select: var("GAHT_SELECT").and_then(|v| v.parse().ok()),
            inventory: var("GAHT_INV").and_then(|v| v.parse().ok()),
            drop: var("GAHT_DROP").and_then(|v| v.parse().ok()),
            book: var("GAHT_BOOK").and_then(|v| v.parse().ok()),
            summon: var("GAHT_SUMMON").is_some(),
            held: var("GAHT_HELD").is_some(),
            town: var("GAHT_TOWN").is_some(),
            build: var("GAHT_BUILD"),
            loot: var("GAHT_LOOT").is_some(),
            chase: var("GAHT_CHASE").is_some(),
            recruit: var("GAHT_RECRUIT").and_then(|v| v.parse().ok()),
            grind: var("GAHT_GRIND"),
            ruin: var("GAHT_RUIN").and_then(|v| v.parse().ok()),
            town_kind: var("GAHT_TOWN").filter(|v| v != "1"),
            duel: var("GAHT_DUEL").is_some(),
            shun: var("GAHT_SHUN").is_some(),
            trade: var("GAHT_TRADE").is_some(),
            trade_table: var("GAHT_TRADE").as_deref() == Some("table"),
            convo: var("GAHT_CONVO").is_some(),
            guard: var("GAHT_GUARD").is_some(),
            factions: var("GAHT_FACTIONS").is_some(),
            feud: var("GAHT_FEUD").is_some(),
            edit: var("GAHT_EDIT").is_some(),
            society: var("GAHT_SOCIETY"),
            nudge: pair("GAHT_NUDGE"),
            interior: var("GAHT_INTERIOR").is_some(),
            cutaway: var("GAHT_CUTAWAY").is_some_and(|v| v != "0") || var("GAHT_VARIANTS").is_some_and(|v| v.split(',').any(|t| t == "open")),
            variants: var("GAHT_VARIANTS"),
            signs: var("GAHT_SIGNS"),
            framing: std::sync::Mutex::new(None),
        })
    }

    /// Where the camera should look instead of at the squad, if anywhere.
    pub fn focus(&self, world: &World) -> Option<V2> {
        use gahturiyu_sim::sim::routine::Doing;
        // A ruin scene looks at the ruin (its chest and crate).
        if let Some(k) = self.ruin {
            return world.ruins.get(k).map(|r| r.pos);
        }
        match self.society.as_deref()? {
            "runners" | "boats" => {
                let want = if self.society.as_deref() == Some("runners") { Doing::Run } else { Doing::Ferry };
                let here = world.squad.pos;
                let p = world.people.iter().map(|p| p.id).filter(|&p| world.doing_now(p).map(|d| d.0) == Some(want)).min_by(|&a, &b| world.person_pos(a).dist(here).total_cmp(&world.person_pos(b).dist(here)))?;
                Some(world.person_pos(p))
            }
            "tides" => {
                let here = world.squad.pos;
                let t = (0..world.society.towns.len()).filter(|&t| world.society.towns[t].stilts.is_some()).min_by(|&a, &b| world.settlements[a].pos.dist(here).total_cmp(&world.settlements[b].pos.dist(here)))?;
                world.society.towns[t].places.iter().find(|p| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::DivePlatform).map(|p| p.pos)
            }
            _ => None,
        }
    }

    /// Set up one of the society scenes: step to the right moment and stand
    /// the squad nearby.
    fn society_scene(&self, world: &mut World, what: &str) {
        use gahturiyu_sim::sim::{jobs::Job, routine::Doing, world::{DAY, HOUR}};
        let step_to = |world: &mut World, t: f64| {
            while world.time < t {
                world.step(60.0f64.min(t - world.time).max(0.01));
            }
        };
        match what {
            "runners" => {
                // A runner working today, in a town with a hearth kitchen.
                let today = World::day_of(world.time);
                let found = (today..today + 6).find_map(|d| {
                    world.people.iter().map(|p| p.id).find_map(|p| {
                        if world.life(p).job != Job::Runner {
                            return None;
                        }
                        let plan = world.day_plan(p, d);
                        let run = plan.segs().iter().position(|s| s.doing == Doing::Run)?;
                        let at = d as f64 * DAY + (plan.segs[run].from as f64 + 0.4) * HOUR;
                        (at > world.time).then_some((p, at))
                    })
                });
                if let Some((p, at)) = found {
                    step_to(world, at);
                    let pos = world.person_pos(p);
                    world.teleport_squad(pos.add(V2::new(30.0, 12.0)));
                    world.step(0.1);
                }
            }
            "boats" => {
                let t = world.society.towns.iter().position(|tl| tl.stilts.is_some());
                if let Some(t) = t {
                    let next = World::day_of(world.time) as f64 * DAY + (gahturiyu_sim::sim::routine::BOAT_OUT as f64 + 0.35) * HOUR;
                    let at = if next > world.time { next } else { next + DAY };
                    step_to(world, at);
                    let dock = world.society.towns[t].places.iter().find(|p| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::Dock).map(|p| p.pos);
                    if let Some(d) = dock {
                        world.teleport_squad(d.add(V2::new(18.0, 14.0)));
                        world.step(0.1);
                    }
                }
            }
            "tides" => {
                let t = world.society.towns.iter().position(|tl| tl.stilts.is_some());
                if let Some(t) = t {
                    let dock = world.society.towns[t].places.iter().find(|p| p.kind == gahturiyu_sim::sim::jobs::PlaceKind::Dock).map(|p| p.pos);
                    if let Some(d) = dock {
                        world.teleport_squad(d.add(V2::new(20.0, 10.0)));
                        world.step(0.1);
                    }
                }
            }
            _ => {}
        }
    }

    /// Set the world up for the picture: everything that changes the world
    /// (as opposed to the camera) happens here, before the first frame.
    pub fn prepare(&self, world: &mut World) {
        if self.forge {
            if let Some(c) = world.forge.as_ref().and_then(|f| f.centre()) {
                world.teleport_squad(c);
                world.step(0.001);
            }
        }
        if let Some((dx, dy)) = self.nudge {
            world.teleport_squad(world.squad.pos.add(V2::new(dx, dy)));
            world.step(0.001);
        }
        if let Some(k) = self.ruin {
            while world.time < 12.0 * gahturiyu_sim::sim::world::HOUR {
                world.step(60.0);
            }
            if let Some(r) = world.ruins.get(k).map(|r| r.pos) {
                world.teleport_squad(r.add(V2::new(45.0, 20.0)));
            }
        }
        if let Some(g) = self.grind.as_deref() {
            let want = gahturiyu_sim::sim::items::id(if g == "mine" { "iron_ore" } else { "timber" });
            // Mid-morning, in good light.
            while world.time < 10.0 * gahturiyu_sim::sim::world::HOUR {
                world.step(60.0);
            }
            let here = world.squad.pos;
            let d = world.deposits.iter().filter(|d| d.item == want).min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).copied();
            if let Some(d) = d {
                world.teleport_squad(d.pos.add(V2::new(7.0, 4.0)));
                for k in 0..2.min(world.squad.members.len()) {
                    let m = world.squad.members[k];
                    // (A seam needs a mining pick.)
                    if let Some(dt) = world.people[m as usize].detail.as_mut() {
                        dt.gear.add(gahturiyu_sim::sim::items::id("mining_pick"), 1);
                    }
                    world.order_labour(m, d.id);
                }
                let end = world.time + 600.0;
                while world.time < end {
                    world.step(0.5);
                }
            }
        }
        if self.chase {
            chase_demo(world);
        }
        if self.loot {
            let at = world.squad.pos.add(V2::new(2.5, 1.0));
            let band = world.spawn_bandits(at, 2, false);
            let foes = world.group(band).map(|g| g.members.clone()).unwrap_or_default();
            let t = world.time;
            for &f in &foes {
                let p = &mut world.people[f as usize];
                p.wounds.lost[1] = p.stats.max_hp(gahturiyu_sim::sim::body::Part::Torso) + 10.0;
                p.wounds.at = t;
            }
            if let (Some(&m), Some(&f)) = (world.squad.members.first(), foes.first()) {
                world.order_loot(m, f);
            }
            for _ in 0..200 {
                world.step(0.05);
            }
        }
        if let Some(what) = self.build.clone() {
            demo_base(world, what == "base" || what == "store");
            // `store`: whoever is there goes through the base's store.
            if what == "store" {
                if let Some(b) = world.bases.first() {
                    let (bid, at) = (b.id, b.at);
                    world.teleport_squad(at.add(V2::new(0.0, -2.0)));
                    world.step(1.0);
                    let lead = world.squad.members[0];
                    world.order_store(lead, bid);
                }
            }
        }
        if let Some(h) = self.hours {
            let end = world.time + h * 3600.0;
            while world.time < end {
                world.step(60.0);
            }
        }
        if self.rest {
            // Whoever carries the tent pitches it beside them; the rest lay
            // out bedrolls round it; then they all lie down (once it's up).
            use gahturiyu_sim::sim::bedding::BedKind;
            let all = world.squad.members.clone();
            if let Some(&m) = all.iter().find(|&&m| world.can_lay(m, BedKind::Tent)) {
                let at = world.person_pos(m).add(V2::new(3.0, 1.5));
                let _ = world.order_lay(m, BedKind::Tent, at, true);
            }
            for (i, &m) in all.iter().enumerate() {
                if world.bed_jobs.iter().any(|j| matches!(j, gahturiyu_sim::sim::bedding::BedJob::Lay { who, .. } if *who == m)) {
                    continue;
                }
                let a = i as f32 * 1.7;
                let at = world.person_pos(m).add(V2::new(a.cos() * 2.5, a.sin() * 2.5));
                let _ = world.order_lay(m, BedKind::Bedroll, at, true);
            }
            for _ in 0..90 {
                world.step(10.0);
            }
        }
        if let Some(what) = self.make.clone() {
            make_scene(world, &what);
        }
        if self.flask {
            use gahturiyu_sim::sim::items::herbs::{Alch, Kit, Mix};
            let m = world.squad.members[0];
            if let Some(d) = world.people[m as usize].detail.as_mut() {
                for _ in 0..2 {
                    d.gear.add_piece(gahturiyu_sim::sim::items::id("brewed_potion"), gahturiyu_sim::sim::materials::Piece::bottle(Mix::brew(&[Alch::Slumber], 60.0, Kit::default())));
                }
                d.gear.add(gahturiyu_sim::sim::items::id("sandglass_flask"), 2);
            }
            let at = world.squad.pos.add(V2::new(11.0, 3.0));
            world.spawn_bandits(at, 3, false);
            for _ in 0..600 {
                world.step(0.1);
                let burst = world.battles.iter().any(|b| b.fx.iter().any(|f| matches!(f.kind, gahturiyu_sim::sim::combat::FxKind::Splash { .. }) && b.time - f.at > 0.4));
                if burst {
                    break;
                }
            }
        }
        if self.drink {
            let spirit = gahturiyu_sim::sim::items::id("stone_spirit");
            let m = world.squad.members.clone();
            // The fourth drinks first and sleeps it off into a hangover.
            if let Some(&p) = m.get(3) {
                if let Some(d) = world.people[p as usize].detail.as_mut() {
                    d.gear.add(spirit, 2);
                }
                world.drink(p, spirit);
                world.drink(p, spirit);
                for _ in 0..80 {
                    world.step(600.0);
                    if world.hungover(p) {
                        break;
                    }
                }
            }
            for (k, n) in [(0usize, 3u16), (1, 2), (2, 1)] {
                if let Some(&p) = m.get(k) {
                    if let Some(d) = world.people[p as usize].detail.as_mut() {
                        d.gear.add(spirit, n);
                    }
                    for _ in 0..n {
                        world.drink(p, spirit);
                    }
                    if let Some(i) = world.squad.index(p) {
                        world.squad.resting[i] = false;
                    }
                }
            }
            // The tipsy one has a pipe or two as well (hazy, the puff).
            if let Some(&p) = m.get(2) {
                let blend = gahturiyu_sim::sim::items::id("elders_blend");
                if let Some(d) = world.people[p as usize].detail.as_mut() {
                    d.gear.add(gahturiyu_sim::sim::items::id("pipe"), 1);
                    d.gear.add(blend, 2);
                }
                world.smoke(p, blend);
                world.smoke(p, blend);
            }
            world.step(1.0);
        }
        if let Some(what) = self.society.clone() {
            self.society_scene(world, &what);
        }
        if let Some(kind) = self.town_kind.clone() {
            let n = world.settlements.len();
            let share = |w: &World, t: usize, r: usize| {
                let c = w.town_counts(t as u16);
                c[r] as f32 / c.iter().sum::<u32>().max(1) as f32
            };
            let pick = match kind.as_str() {
                "roduro" => (0..n).max_by(|&a, &b| share(world, a, 0).total_cmp(&share(world, b, 0))),
                "qotiro" => (0..n).max_by(|&a, &b| share(world, a, 1).total_cmp(&share(world, b, 1))),
                "horaro" => (0..n).max_by(|&a, &b| share(world, a, 2).total_cmp(&share(world, b, 2))),
                _ => (0..n).min_by(|&a, &b| {
                    let top = |t: usize| (0..4).map(|r| share(world, t, r)).fold(0.0, f32::max);
                    top(a).total_cmp(&top(b))
                }),
            };
            if let Some(t) = pick {
                let at = world.settlements[t].pos.add(V2::new(25.0, 10.0));
                world.teleport_squad(at);
                world.step(0.001);
            }
        }
        if self.shun {
            let here = world.squad.pos;
            if let Some(t) = world.settlements.iter().filter(|s| s.coastal).min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|s| s.id) {
                let at = world.settlements[t as usize].pos.add(V2::new(25.0, 10.0));
                world.teleport_squad(at);
                let now = world.time;
                world.wrong_village(t, now);
                // On past the next dawn, when the boats don't come.
                let day = gahturiyu_sim::sim::World::day_of(now);
                let dawn = (day + 1) as f64 * 86400.0 + 7.0 * 3600.0;
                while world.time < dawn {
                    world.step(60.0);
                }
            }
        }
        if self.duel {
            let here = world.squad.pos;
            if let Some(t) = world.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|s| s.id) {
                let m = world.squad.members[0];
                world.judge(m, t, gahturiyu_sim::sim::law::Wrong::Theft, 40.0, gahturiyu_sim::sim::culture::Justice::Duel);
                for _ in 0..40 {
                    world.step(0.1);
                }
            }
        }
        if self.sneak {
            for m in world.squad.members.clone() {
                world.set_sneaking(m, true);
            }
        }
        if let Some(k) = self.camp {
            if let Some(c) = world.camps.get(k) {
                let at = c.pos.add(V2::new(70.0, 0.0));
                world.teleport_squad(at);
            }
        }
        if let Some(h) = self.wait {
            let end = world.time + h * 3600.0;
            let near = |w: &World| w.battles.iter().any(|b| b.fighters.iter().any(|f| f.pos.dist(w.squad.pos) < 300.0));
            while world.time < end && !near(world) {
                world.step(1.0);
            }
        }
        if self.starve || self.exhaust || self.carry || self.limb || self.ranged || self.summon {
            // Out of town, where it's quiet.
            world.teleport_squad(world.squad.pos.add(V2::new(260.0, 40.0)));
            world.step(0.1);
        }
        if self.starve {
            let t = world.time;
            for m in world.squad.members.clone() {
                let p = &mut world.people[m as usize];
                if let Some(d) = p.detail.as_mut() {
                    d.gear.bag.retain(|e| !matches!(items::item(e.0).kind, Kind::Food(_)));
                }
                if let Some(c) = p.cond.as_mut() {
                    c.hunger = 92.0;
                    c.at = t;
                }
            }
            for _ in 0..240 {
                world.step(60.0);
            }
        }
        if self.exhaust {
            let t = world.time;
            for m in world.squad.members.clone() {
                if let Some(c) = world.people[m as usize].cond.as_mut() {
                    c.tired = 92.0;
                    c.stamina = 4.0;
                    c.at = t;
                }
            }
            world.step(1.0);
        }
        if self.carry {
            let (a, b) = (world.squad.members[0], world.squad.members[2]);
            let t = world.time;
            let p = &mut world.people[b as usize];
            p.wounds.lost[1] = p.stats.max_hp(Part::Torso) + 25.0;
            p.wounds.at = t;
            world.order_carry(a, b);
            for _ in 0..120 {
                world.step(0.5);
            }
            let to = world.person_pos(a).add(V2::new(-14.0, 10.0));
            world.order_members(&[a], to);
            for _ in 0..40 {
                world.step(0.5);
            }
        }
        if self.limb {
            let m = world.squad.members[0];
            let p = &mut world.people[m as usize];
            let max = p.stats.max_hp(Part::LeftArm);
            p.wounds.lost[2] = max * (1.0 + body::LIMB_LOSS);
            p.wounds.missing[2] = true;
        }
        if self.ranged {
            let at = world.squad.pos.add(V2::new(25.0, 8.0));
            world.spawn_bandits(at, 5, false);
            for _ in 0..80 {
                world.step(0.25);
                if world.squad_battle().map(|b| b.fighters.iter().any(|f| f.shots > 0)).unwrap_or(false) {
                    break;
                }
            }
        }
        let mage = world.squad.members.iter().copied().find(|&m| world.people[m as usize].stats.calling == gahturiyu_sim::sim::stats::Calling::Mage);
        // `GAHT_CAST=glow,haste,douse`: the mage learns these and casts each
        // in turn (on themselves, on the first member, or at the nearest
        // flame), so the casts and the badges they leave show.
        if let Ok(keys) = std::env::var("GAHT_CAST") {
            if let Some(m) = mage.or(world.squad.members.first().copied()) {
                use gahturiyu_sim::sim::magic::{self, Aim};
                for key in keys.split(',') {
                    let s = magic::spell(key.trim());
                    if let Some(d) = world.people[m as usize].detail.as_mut() {
                        if !d.spells.contains(&s) {
                            d.spells.push(s);
                        }
                    }
                    let t = world.time;
                    let full = world.people[m as usize].max_mana();
                    world.people[m as usize].set_mana(full, t);
                    let first = world.squad.members[0];
                    let (target, point) = match s.def().aim {
                        Aim::Friend | Aim::Anyone => (Some(first), None),
                        Aim::Point => {
                            let at = world.person_pos(m);
                            let douse = s.def().effects.iter().any(|e| e.does == gahturiyu_sim::sim::effects::Does::Douse);
                            (None, world.flames_near(at, 14.0).into_iter().find(|f| f.2 == douse).map(|f| f.1).or(Some(at)))
                        }
                        _ => (None, None),
                    };
                    if let Err(e) = world.use_spell(m, s, target, point) {
                        eprintln!("GAHT_CAST {key}: {}", e.0);
                    }
                }
            }
        }
        if self.held {
            if let Some(m) = mage {
                let t = world.time;
                world.set_holding(m, t, Some(gahturiyu_sim::sim::magic::spell("restore")));
            }
        }
        if self.summon {
            let at = world.squad.pos.add(V2::new(16.0, 5.0));
            world.spawn_bandits(at, 4, true);
            for _ in 0..200 {
                world.step(0.25);
                if world.squad_battle().is_some() {
                    break;
                }
            }
            if let Some(m) = mage {
                let toward = world.squad.pos.add(V2::new(5.0, 2.0));
                let _ = world.cast_in_fight(m, gahturiyu_sim::sim::magic::spell("spirit_beast"), None, Some(toward));
            }
            // One of theirs falls, and the squad's mage raises them.
            let raised = (|| {
                let b = world.battles.iter_mut().find(|b| b.fighters.iter().any(|f| f.side == 0))?;
                let corpse = b.fighters.iter().position(|f| f.side == 1 && f.is_person() && f.summon.is_none() && f.stats.calling != gahturiyu_sim::sim::stats::Calling::Mage)?;
                b.fighters[corpse].dead = true;
                b.fighters[corpse].ko = true;
                let caster = b.fighters.iter().position(|f| f.side == 0 && f.stats.calling == gahturiyu_sim::sim::stats::Calling::Mage)?;
                let until = b.time + 45.0;
                b.raise(caster, corpse, until);
                Some(())
            })();
            let _ = raised;
            for _ in 0..40 {
                world.step(0.1);
            }
        }
        if self.torch {
            let (a, b) = (world.squad.members[0], world.squad.members[1]);
            world.toggle_torch(a);
            world.toggle_torch(b);
            // The hunter carries the standing torches.
            let hunter = world.squad.members.iter().copied().find(|&m| world.people[m as usize].detail.as_ref().map(|d| d.gear.bag.iter().any(|e| e.0 == items::id("standing_torch"))).unwrap_or(false));
            if let Some(h) = hunter {
                world.place_torch(h);
            }
            world.step(0.5);
        }
        if self.talk {
            let lead = world.squad.members[0];
            let here = world.squad.pos;
            let town = world.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|t| t.id).unwrap();
            let want = self.talk_to.clone();
            // Someone who can be talked to: awake, and out of doors before
            // someone indoors (RG-12: at dawn the nearest local is abed). If
            // nobody is up yet, wait until someone is.
            let pick = |world: &World| {
                world
                    .residents_in_band1(town)
                    .into_iter()
                    .filter(|&p| !world.people[p as usize].dead && !world.is_indoors_asleep(p))
                    .filter(|&p| want.as_deref().map(|w| world.people[p as usize].race.name().to_lowercase().replace('ṭ', "t").replace('ḍ', "d") == w || world.life(p).job.name().to_lowercase() == w || (w == "indoors" && world.building_at(world.person_pos(p)).is_some())).unwrap_or(true))
                    .min_by(|&a, &b| {
                        let key = |p| (world.building_at(world.person_pos(p)).is_some(), world.person_pos(p).dist(here));
                        let (ka, kb) = (key(a), key(b));
                        ka.0.cmp(&kb.0).then(ka.1.total_cmp(&kb.1))
                    })
            };
            let mut npc = pick(world);
            for _ in 0..36 {
                if npc.is_some() {
                    break;
                }
                world.step(300.0);
                npc = pick(world);
            }
            if let Some(npc) = npc {
                world.order_talk(lead, npc);
                for _ in 0..240 {
                    if world.talk.is_some() {
                        break;
                    }
                    world.step(0.5);
                }
                world.ask(Topic::Background);
                world.ask(Topic::ThisTown);
                world.ask(Topic::Bandits);
                if world.topics().contains(&Topic::Work) {
                    world.ask(Topic::Work);
                }
            }
        }
        if let Some(n) = self.recruit {
            let lead = world.squad.members[0];
            let here = world.squad.pos;
            let mut town = world.settlements.iter().map(|t| t.id).min_by(|&a, &b| world.settlements[a as usize].pos.dist(here).total_cmp(&world.settlements[b as usize].pos.dist(here))).unwrap();
            for k in 0..n {
                // The nearest town with anyone willing left.
                let pick = |w: &World, town: u16| w.settlements[town as usize].residents.iter().copied().filter(|&p| w.join_terms(p).is_some()).min_by(|&a, &b| w.person_pos(a).dist(here).total_cmp(&w.person_pos(b).dist(here)));
                if pick(&world, town).is_none() {
                    let near = world.settlements.iter().filter(|t| pick(&world, t.id).is_some()).min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|t| t.id);
                    match near {
                        Some(t) => town = t,
                        None => break,
                    }
                }
                let Some(npc) = pick(&world, town) else { break };
                let fee = world.join_terms(npc).unwrap_or(0);
                world.people[lead as usize].detail.as_mut().unwrap().gear.add(gahturiyu_sim::sim::items::id("coin"), fee);
                world.teleport_squad(world.person_pos(npc).add(V2::new(2.0, 0.0)));
                if k + 1 < n {
                    let _ = world.recruit(npc, lead);
                    continue;
                }
                world.order_talk(lead, npc);
                for _ in 0..240 {
                    if world.talk.is_some() {
                        break;
                    }
                    world.step(0.5);
                }
                world.ask(Topic::Background);
                world.ask(Topic::Join(fee));
            }
        }
        if self.convo {
            use gahturiyu_sim::sim::{history::Deed, memory::Who};
            let lead = world.squad.members[0];
            let here = world.squad.pos;
            let town = world.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|t| t.id).unwrap();
            let folk = world.residents_in_band1(town);
            let near = folk.iter().copied().filter(|&p| !world.is_indoors_asleep(p)).min_by(|&a, &b| world.person_pos(a).dist(here).total_cmp(&world.person_pos(b).dist(here)));
            let thief = folk.iter().copied().find(|&p| Some(p) != near);
            if let (Some(npc), Some(thief)) = (near, thief) {
                let t = world.time - 10.0 * 3600.0;
                world.note(Deed::Theft, Some(thief), Some(npc), town, t, true);
                let day = World::day_of(world.time) as i32;
                world.remember(npc, Who::Someone, Deed::Theft, -0.6, day);
                world.order_talk(lead, npc);
                for _ in 0..240 {
                    if world.talk.is_some() {
                        break;
                    }
                    world.step(0.5);
                }
                let opts = world.topics();
                if let Some(t) = opts.iter().find(|t| matches!(t, Topic::Say(_))) {
                    world.ask(*t);
                }
            }
        }
        if self.factions {
            use gahturiyu_sim::sim::factions::id;
            let here = world.squad.pos;
            if let Some(t) = (0..world.settlements.len()).min_by(|&a, &b| world.settlements[a].pos.dist(here).total_cmp(&world.settlements[b].pos.dist(here))) {
                world.settlements[t].holder = Some(id("banner_lords"));
            }
            world.deed(id("banner_lords"), 40.0);
            world.deed(id("true_line"), -30.0);
            world.deed(id("writers"), 12.0);
            world.lord_deed(0, 25.0);
            world.deed(id("binders"), -60.0);
        }
        if self.guard {
            use gahturiyu_sim::sim::{chances::Chance, jobs::Job, world::{DAY, HOUR}};
            let here = world.squad.pos;
            let merchant = world.people.iter().map(|p| p.id).filter(|&p| world.life(p).job == Job::Merchant && world.life(p).place.is_some() && world.people[p as usize].home.is_some()).min_by(|&a, &b| world.person_pos(a).dist(here).total_cmp(&world.person_pos(b).dist(here)));
            if let Some(mer) = merchant {
                let town = world.people[mer as usize].home.unwrap();
                let mut o = world.opp(Chance::Guard, mer, town);
                o.place = world.life(mer).place;
                o.amount = 14.0;
                o.reward = 56;
                let t = world.time;
                if let Some(id) = world.post_opp(o, t) {
                    let m = world.squad.members[0];
                    world.take_opportunity(id, m);
                    // On to the next day's shift.
                    let c = world.contract_of(m).unwrap().clone();
                    let at = (c.first_day as f64) * DAY + (c.hours.0 as f64 + 1.0) * HOUR;
                    while world.time < at - 300.0 {
                        world.step(600.0f64.min(at - 300.0 - world.time).max(1.0));
                    }
                    let pos = world.contract_pos(&c);
                    world.teleport_squad(pos.add(V2::new(25.0, 8.0)));
                    // Until they've walked over to it.
                    for _ in 0..900 {
                        world.step(1.0);
                        if world.member_pos(0).dist(pos) < gahturiyu_sim::sim::chances::AT_POST * 0.7 && world.time >= at {
                            break;
                        }
                    }
                }
            }
        }
        if self.feud {
            use gahturiyu_sim::sim::{history::Deed, memory::Who};
            let here = world.squad.pos;
            let town = world.settlements.iter().min_by(|a, b| a.pos.dist(here).total_cmp(&b.pos.dist(here))).map(|t| t.id).unwrap();
            for tl in &mut world.society.towns {
                tl.drama = 0.8;
            }
            // The two smallest households of the town.
            let mut hhs: Vec<u32> = (0..world.society.households.len() as u32).filter(|&h| {
                let hh = &world.society.households[h as usize];
                world.society.communities[hh.community as usize].town == town && !hh.members.is_empty() && hh.members.iter().all(|&m| !world.people[m as usize].in_squad)
            }).collect();
            hhs.sort_by_key(|&h| (world.society.households[h as usize].members.len(), h));
            if hhs.len() >= 2 {
                let (a, b) = (hhs[0], hhs[1]);
                let them = world.society.households[b as usize].members[0];
                for _ in 0..16 {
                    let day = World::day_of(world.time) as i32;
                    for m in world.society.households[a as usize].members.clone() {
                        world.remember(m, Who::Person(them), Deed::Slight, -0.25, day);
                    }
                    for _ in 0..24 {
                        world.step(3600.0);
                    }
                }
            }
        }
        if self.edit {
            let at = super::editor::open_ground(world, world.squad.pos);
            super::editor::demo(world, at);
            world.teleport_squad(at.add(V2::new(-60.0, 70.0)));
        }
        if self.trade {
            let here = world.squad.pos;
            let npc = world.people.iter().map(|p| p.id).filter(|&p| world.is_trading(p)).min_by(|&a, &b| world.person_pos(a).dist(here).total_cmp(&world.person_pos(b).dist(here)));
            if let Some(npc) = npc {
                let lead = world.squad.members[0];
                let at = world.person_pos(npc);
                world.teleport_squad(at.add(V2::new(1.5, 0.0)));
                world.squad.at[0] = at.add(V2::new(1.0, 0.0));
                world.order_talk(lead, npc);
                for _ in 0..20 {
                    if world.talk.is_some() {
                        break;
                    }
                    world.step(0.5);
                }
                world.ask(Topic::Trade);
            }
        }
        if self.enter {
            let m = world.squad.members[0];
            let here = world.squad.pos;
            let d = world.doors_near(here, 200.0).into_iter().filter(|d| !world.is_locked(d.id)).min_by(|a, b| a.centre.dist(here).total_cmp(&b.centre.dist(here)));
            if let Some(d) = d {
                world.order_members(&[m], d.centre);
                for _ in 0..600 {
                    world.step(0.5);
                }
            }
        }
        if let Some(n) = self.bandits {
            let at = world.squad.pos.add(V2::new(14.0, 6.0));
            world.spawn_bandits(at, n, true);
        }
        if let Some(k) = self.drop {
            if let Some(&m) = world.squad.members.get(k) {
                for slot in [Slot::MainHand, Slot::BodyOver, Slot::Back] {
                    let it = world.people[m as usize].detail.as_ref().and_then(|d| d.gear.in_slot(slot));
                    if let Some(it) = it {
                        world.unequip(m, slot);
                        world.drop_item(m, it);
                    }
                }
                let to = world.person_pos(m).add(V2::new(-12.0, 8.0));
                world.order_members(&[m], to);
            }
        }
        if let Some(which) = self.variants.clone() {
            *self.framing.lock().unwrap() = variants_row(world, &which, self.cutaway);
        }
        if self.interior {
            *self.framing.lock().unwrap() = walk_in(world);
        }
        if let Some(which) = self.signs.clone() {
            *self.framing.lock().unwrap() = signs_street(world, &which);
        }
        if let Ok(how) = std::env::var("GAHT_WAY") {
            *self.framing.lock().unwrap() = way_round(world, &how);
        }
        super::animals::prepare(world);
    }
}

/// `GAHT_INTERIOR`: member 0 walks into the nearest building that's open
/// and has containers (a workshop or shop if one is near), then opens an
/// unlocked container there. Returns the camera's framing.
fn walk_in(world: &mut World) -> Option<(V2, f32, f32, Option<f32>)> {
    use gahturiyu_sim::sim::buildings::door_of;
    use gahturiyu_sim::sim::layout::Use;
    use gahturiyu_sim::sim::layout::Furn;
    use gahturiyu_sim::sim::race::Race;
    // `GAHT_INTERIOR=tadoro`: the nearest open building a Ṭaḍoro lodges in;
    // `crowded`: the one with the most residents per sleeping place;
    // `slope`: one on a slope, its floor about 1.5 m up (anywhere);
    // a variant's key (e.g. `roduro_longhouse`): the nearest open building
    // of that variant.
    let mode = std::env::var("GAHT_INTERIOR").unwrap_or_default();
    let want_key = gahturiyu_sim::sim::layout::by_key(&mode).map(|v| v.key);
    let m = *world.squad.members.first()?;
    let here = world.squad.pos;
    let mut best: Option<(f32, gahturiyu_sim::sim::buildings::Door)> = None;
    for s in &world.settlements {
        // (`crowded` and `slope` look everywhere: overfull quarters are rare.)
        if s.pos.dist(here) > 3000.0 && mode != "crowded" && mode != "slope" && want_key.is_none() {
            continue;
        }
        for i in 0..s.buildings.len() as u16 {
            let Some(d) = door_of(s, i) else { continue };
            let v = d.variant();
            if world.is_locked(d.id) || (v.holders.is_empty() && mode != "tadoro" && mode != "crowded") {
                continue;
            }
            if want_key.is_some_and(|k| k != v.key) {
                continue;
            }
            let trade = matches!(v.use_, Use::Workshop | Use::Shop);
            let score = match mode.as_str() {
                "tadoro" => {
                    let lodger = world.residents_of(d.id).iter().any(|&p| world.people[p as usize].race == Race::Tadoro);
                    if !lodger {
                        continue;
                    }
                    d.centre.dist(here) + if v.walls.is_empty() { 300.0 } else { 0.0 }
                }
                "crowded" => {
                    let beds = v.furniture.iter().filter(|p| matches!(p.what, Furn::Bed | Furn::Bedroll)).count().max(1);
                    let extra = world.residents_of(d.id).len().saturating_sub(beds);
                    if extra == 0 || v.furniture.is_empty() {
                        continue;
                    }
                    // Living quarters, then homes, before workshops; then the
                    // most extra sleepers (up to the 16 drawn), then nearest.
                    let living = match v.use_ {
                        Use::Quarters => 0.0,
                        Use::Home => 10_000.0,
                        _ => 20_000.0,
                    };
                    living - (extra.min(16) as f32) * 1000.0 + d.centre.dist(here) * 0.1
                }
                // `slope`: an open building on a slope (floor about 1.5 m over the lowest ground).
                "slope" => {
                    let (hi, lo) = super::interiors::floor_height(&d, &|p| world.terrain.height(p));
                    d.centre.dist(here) * 0.00005 + (hi - lo - 1.5).abs()
                }
                _ if want_key.is_some() => d.centre.dist(here),
                _ => d.centre.dist(here) + if trade { 0.0 } else { 400.0 } + if v.walls.is_empty() { 200.0 } else { 0.0 },
            };
            if best.as_ref().is_none_or(|b| score < b.0) {
                best = Some((score, d));
            }
        }
    }
    let (_, d) = best?;
    if mode == "slope" {
        let (hi, lo) = super::interiors::floor_height(&d, &|p| world.terrain.height(p));
        eprintln!("slope: {} ({:?}), floor {:.2} m over the lowest ground", d.variant().key, d.id, hi - lo);
    }
    if mode == "crowded" {
        eprintln!("crowded: {} ({:?}), {} residents, {} sleeping places drawn", d.variant().key, d.variant().use_, world.residents_of(d.id).len(), super::interiors::sleeping_places(world, &d));
    }
    if d.centre.dist(here) > 120.0 {
        let out = d.outside.sub(d.centre);
        let out = out.scale(1.0 / out.len().max(0.01));
        world.teleport_squad(d.outside.add(out.scale(6.0)));
        world.step(0.001);
    }
    world.order_members(&[m], d.inside);
    let k = world.squad.index(m)?;
    for _ in 0..2000 {
        world.step(0.1);
        if world.squad.inside[k] == Some(d.id) && world.squad.route[k].is_empty() {
            break;
        }
    }
    // `tadoro`: no container opened; look close at the lodger's corner.
    if mode == "tadoro" {
        let corner = super::interiors::lodger_corner(world, &d).unwrap_or(d.centre);
        return Some((corner.lerp(d.centre, 0.15), 8.0, 0.75, Some(d.rot + 0.5)));
    }
    // An unlocked container, chests and cupboards first, something in it.
    let pick = world
        .containers_in(d.id)
        .filter(|c| !world.container_locked(c.id))
        .min_by_key(|c| (c.items.is_empty(), !matches!(c.what, gahturiyu_sim::sim::layout::Holder::Chest | gahturiyu_sim::sim::layout::Holder::Cupboard), c.id))
        .map(|c| c.id);
    if let Some(id) = pick {
        world.order_search(m, id);
        for _ in 0..2000 {
            world.step(0.1);
            if world.searching_now(m) == Some(id) {
                break;
            }
        }
    }
    // Look from the front, down into the rooms.
    let yaw = d.rot + 0.5;
    Some((d.centre, 17.0, 0.78, Some(yaw)))
}

/// `GAHT_WAY=1`: the first two squad members are sent to the far side of
/// the nearest building and caught a few seconds into the walk (the order
/// line follows their way round it, UI-6). Returns the camera's framing,
/// from overhead.
/// `GAHT_WAY=sea`: the same two are sent out to sea from the nearest shore
/// (UI-7: the line ends at the water's edge, a grey cross out at the spot).
pub static WAY_SHORT: std::sync::Mutex<Option<V2>> = std::sync::Mutex::new(None);

fn way_round(world: &mut World, how: &str) -> Option<(V2, f32, f32, Option<f32>)> {
    let here = world.squad.pos;
    if how == "sea" {
        // The nearest open water, looking outward in rings.
        let mut sea = None;
        'look: for r in (20..6000).step_by(20) {
            for a in 0..64 {
                let ang = a as f32 / 64.0 * std::f32::consts::TAU;
                let p = here.add(V2::new(ang.cos(), ang.sin()).scale(r as f32));
                if world.open_water(p) {
                    sea = Some(p);
                    break 'look;
                }
            }
        }
        let sea = sea?;
        // Back to dry land along the line, then 20 m more inland.
        let back = here.sub(sea);
        let back = back.scale(1.0 / back.len().max(0.01));
        let mut shore = sea;
        while world.open_water(shore) {
            shore = shore.add(back.scale(1.0));
        }
        world.teleport_squad(shore.add(back.scale(20.0)));
        world.step(0.001);
        let to = shore.sub(back.scale(30.0));
        let who: Vec<_> = world.squad.members.iter().copied().take(2).collect();
        world.order_members(&who, to);
        for _ in 0..30 {
            world.step(0.1);
        }
        *WAY_SHORT.lock().unwrap() = Some(to);
        let from = world.squad.pos;
        eprintln!("sea: squad {:?}, shore {:?}, to {:?} ({:.0} m)", from, shore, to, from.dist(to));
        return Some((from.lerp(to, 0.5), from.dist(to) * 1.6 + 20.0, 1.2, None));
    }
    let d = world.doors_near(here, 300.0).into_iter().min_by(|a, b| a.centre.dist(here).total_cmp(&b.centre.dist(here)))?;
    // Stand on one side of it, then go to the other.
    let side = here.sub(d.centre);
    let side = side.scale(1.0 / side.len().max(0.01));
    world.teleport_squad(d.centre.add(side.scale(d.radius + 7.0)));
    world.step(0.001);
    let to = d.centre.sub(side.scale(d.radius + 7.0));
    let who: Vec<_> = world.squad.members.iter().copied().take(2).collect();
    world.order_members(&who, to);
    for _ in 0..30 {
        world.step(0.1);
    }
    Some((d.centre, 30.0, 1.25, None))
}

/// `GAHT_SIGNS`: the spot in a town where the most signs (trade buildings
/// and workplaces) stand within 40 m of each other; the squad is moved off
/// to one side. Returns the camera's framing, looking at the first
/// building's front.
fn signs_street(world: &mut World, which: &str) -> Option<(V2, f32, f32, Option<f32>)> {
    use gahturiyu_sim::sim::layout::variant_in;
    let here = world.squad.pos;
    let n = world.settlements.len();
    let share = |w: &World, t: usize, r: usize| {
        let c = w.town_counts(t as u16);
        c[r] as f32 / c.iter().sum::<u32>().max(1) as f32
    };
    let town = match which {
        "roduro" => (0..n).max_by(|&a, &b| share(world, a, 0).total_cmp(&share(world, b, 0)))?,
        "qotiro" => (0..n).max_by(|&a, &b| share(world, a, 1).total_cmp(&share(world, b, 1)))?,
        _ => (0..n).min_by(|&a, &b| world.settlements[a].pos.dist(here).total_cmp(&world.settlements[b].pos.dist(here)))?,
    };
    let s = &world.settlements[town];
    // Every sign: (where, the facing of the building it's on, if a building).
    let mut sites: Vec<(V2, Option<f32>)> = (0..s.buildings.len() as u16)
        .filter(|&i| variant_in(s, i).is_some_and(super::signs::signed))
        .map(|i| (s.buildings[i as usize].pos, Some(s.buildings[i as usize].rot)))
        .collect();
    if let Some(tl) = world.society.towns.get(town) {
        sites.extend(tl.places.iter().filter(|p| super::signs::for_place(p.kind).is_some()).map(|p| (p.pos, None)));
    }
    // Building signs count three times a signpost.
    let near = |c: V2| sites.iter().filter(|o| o.0.dist(c) < 40.0).map(|o| if o.1.is_some() { 3 } else { 1 }).sum::<usize>();
    let best = sites.iter().filter(|o| o.1.is_some()).max_by_key(|o| near(o.0)).or(sites.first())?;
    let group: Vec<V2> = sites.iter().filter(|o| o.0.dist(best.0) < 40.0).map(|o| o.0).collect();
    let mean = group.iter().fold(V2::new(0.0, 0.0), |a, &p| a.add(p)).scale(1.0 / group.len().max(1) as f32);
    // Mostly on the best building (its front and sign in the middle of the
    // picture), a little toward the rest.
    let centre = best.0.lerp(mean, 0.3);
    eprintln!("signs: town {town}, {} building signs, {} in view", sites.iter().filter(|o| o.1.is_some()).count(), group.len());
    let yaw = best.1.unwrap_or(0.0);
    world.teleport_squad(centre.add(V2::new(yaw.cos(), yaw.sin()).scale(-45.0)));
    world.step(0.001);
    Some((centre, 38.0, 0.6, Some(yaw)))
}

/// `GAHT_VARIANTS`: lay every building variant (or one people's) out in rows
/// on open ground near the squad, as buildings of the nearest town (a
/// screenshot-only change to the world). Returns the camera's framing.
fn variants_row(world: &mut World, which: &str, open: bool) -> Option<(V2, f32, f32, Option<f32>)> {
    use gahturiyu_sim::sim::layout::{example_of, VARIANTS};
    use gahturiyu_sim::sim::settlement::{Building, BuildingKind as K};
    let tokens: Vec<&str> = which.split(',').map(|t| t.trim()).collect();
    let want = |k: K| {
        let any = !tokens.iter().any(|t| matches!(*t, "roduro" | "qotiro" | "horaro"));
        any || match k {
            K::RoduroHome => tokens.contains(&"roduro"),
            K::QotiroBlock | K::QotiroTemple | K::QotiroHall => tokens.contains(&"qotiro"),
            K::HoraroStilt => tokens.contains(&"horaro"),
            K::Hearth => false,
        }
    };
    let list: Vec<_> = VARIANTS.iter().filter(|v| want(v.kind)).collect();
    if list.is_empty() {
        return None;
    }
    // Rows facing the camera (+y), across the x axis.
    let cols = if list.len() <= 6 { list.len() } else { list.len().div_ceil(list.len().div_ceil(6)) };
    let gap = 6.0;
    let mut placed: Vec<(V2, f32, &gahturiyu_sim::sim::layout::Variant, u64)> = Vec::new();
    let mut y = 0.0f32;
    let mut width = 0.0f32;
    for row in list.chunks(cols) {
        let sizes: Vec<(f32, u64)> = row.iter().map(|v| example_of(v)).collect();
        let across: Vec<f32> = row.iter().zip(&sizes).map(|(v, s)| 2.0 * v.half.1 * s.0).collect();
        let deep = row.iter().zip(&sizes).map(|(v, s)| 2.0 * v.half.0 * s.0).fold(0.0, f32::max);
        let total: f32 = across.iter().sum::<f32>() + gap * (row.len() as f32 - 1.0);
        width = width.max(total);
        let mut x = -total * 0.5;
        for ((v, s), a) in row.iter().zip(&sizes).zip(&across) {
            placed.push((V2::new(x + a * 0.5, y - deep * 0.5), s.0, v, s.1));
            x += a + gap;
        }
        y -= deep + gap * 1.5;
    }
    let depth = -y;
    // Open, fairly flat land near the squad, clear of towns.
    let t = &world.terrain;
    let here = world.squad.pos;
    let clear = |c: V2| -> bool {
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for i in 0..=8 {
            for j in 0..=5 {
                let p = c.add(V2::new((i as f32 / 8.0 - 0.5) * (width + 20.0), 10.0 - j as f32 / 5.0 * (depth + 20.0)));
                if t.is_sea(p) {
                    return false;
                }
                let h = t.height(p);
                lo = lo.min(h);
                hi = hi.max(h);
            }
        }
        let busy = world.settlements.iter().any(|s| s.buildings.iter().any(|b| (b.pos.x - c.x).abs() < width * 0.5 + 25.0 && b.pos.y < c.y + 25.0 && b.pos.y > c.y - depth - 25.0));
        let squad = (here.x - c.x).abs() < width * 0.5 + 10.0 && here.y < c.y + 10.0 && here.y > c.y - depth - 10.0;
        hi - lo < 3.0 && !busy && !squad
    };
    let mut origin = here.add(V2::new(0.0, 60.0));
    'search: for r in 1..40 {
        for k in 0..16 {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            let c = here.add(V2::new(a.cos(), a.sin()).scale(r as f32 * 40.0));
            if clear(c) {
                origin = c;
                break 'search;
            }
        }
    }
    let town = world.settlements.iter().min_by(|a, b| a.pos.dist(origin).total_cmp(&b.pos.dist(origin))).map(|s| s.id)?;
    let s = &mut world.settlements[town as usize];
    for (at, size, v, seed) in placed {
        let pos = origin.add(at);
        s.buildings.push(Building { pos, kind: v.kind, size, rot: std::f32::consts::FRAC_PI_2, seed });
        s.reach = s.reach.max(pos.dist(s.pos) + size);
    }
    world.solid_changed();
    let centre = origin.add(V2::new(0.0, -depth * 0.5 + 4.0));
    let dist = (width * 0.8).max(depth * 1.5) + 12.0;
    Some((centre, dist, if open { 0.95 } else { 0.62 }, Some(std::f32::consts::FRAC_PI_2)))
}

/// The squad at a station with work for it (for `GAHT_MAKE`).
fn make_scene(world: &mut World, what: &str) {
    use gahturiyu_sim::sim::crafting::{Station, RECIPES};
    use gahturiyu_sim::sim::items;
    let station = match what {
        "bench" => Station::Bench,
        "loom" => Station::Loom,
        "workbench" => Station::Workbench,
        "desk" => Station::Desk,
        "bed" => Station::GrowerBed,
        "table" | "mortar" => Station::AlchemyTable,
        "campfire" | "wild" => Station::Campfire,
        "stove" => Station::Stove,
        _ => Station::Forge,
    };
    let here = world.squad.pos;
    let m = world.squad.members.clone();
    if what == "wild" {
        // Out in the open: the first of them lays a campfire of their own.
        let mut spot = here;
        for k in 0..40 {
            let p = here.add(V2::new(150.0 + 40.0 * k as f32, 90.0));
            if world.town_at(p).is_none() && !world.terrain.is_sea(p) && world.can_stand(p, 0.3) {
                spot = p;
                break;
            }
        }
        world.teleport_squad(spot);
        if let Some(d) = world.people[m[0] as usize].detail.as_mut() {
            d.gear.add(items::id("timber"), 2);
            d.gear.add(items::id("tinderbox"), 1);
        }
        let fire = spot.add(V2::new(2.5, 0.0));
        let _ = world.order_campfire(m[0], fire);
        for _ in 0..60 {
            world.step(0.25);
        }
        world.step(gahturiyu_sim::sim::campfire::TINDER_SECS + 1.0);
        // The rest stand round it.
        for (j, &p) in m.iter().enumerate().skip(1) {
            if let Some(k) = world.squad.index(p) {
                let a = j as f32 * 1.9;
                let at = fire.add(V2::new(a.cos(), a.sin()).scale(2.2));
                world.squad.at[k] = at;
                world.squad.goal[k] = at;
            }
        }
    } else if station == Station::Campfire {
        // The nearest town fire, clear of its workshop's benches.
        let mut spot = None;
        let mut towns: Vec<usize> = (0..world.settlements.len()).collect();
        towns.sort_by(|&a, &b| world.settlements[a].pos.dist(here).total_cmp(&world.settlements[b].pos.dist(here)));
        'find: for s in towns {
            let (c, r) = (world.settlements[s].pos, world.settlements[s].reach);
            for th in world.solids_near(c, r) {
                if th.what != gahturiyu_sim::sim::solid::What::Fire || !world.fire_lit(th.id, world.time) {
                    continue;
                }
                let f = th.shape.centre();
                for k in 0..16 {
                    let a = k as f32 * 0.4;
                    let p = f.add(V2::new(a.cos(), a.sin()).scale(2.6));
                    if world.stations.iter().all(|s| s.0.dist(p) > f.dist(p)) && world.fire_at(p).is_some() && world.can_stand(p, 0.3) {
                        spot = Some(p);
                        break 'find;
                    }
                }
            }
        }
        let Some(p) = spot else {
            eprintln!("GAHT_MAKE: no fire in this world");
            return;
        };
        world.teleport_squad(p);
    } else if station == Station::Stove {
        let inside = (0..world.settlements.len() as u16)
            .flat_map(|s| (0..world.settlements[s as usize].buildings.len() as u16).map(move |b| (s, b)))
            .filter_map(|(s, b)| gahturiyu_sim::sim::buildings::door_of(&world.settlements[s as usize], b))
            .filter(|d| world.station_near(d.centre, Station::Stove).is_some())
            .min_by(|a, b| a.centre.dist(here).total_cmp(&b.centre.dist(here)));
        let Some(d) = inside else {
            eprintln!("GAHT_MAKE: no hearth in this world");
            return;
        };
        let p = world.free_spot(d.centre);
        world.teleport_squad(p);
    } else {
        let Some(at) = world.stations.iter().filter(|(_, k)| *k == station).map(|(p, _)| *p).min_by(|a, b| a.dist(here).total_cmp(&b.dist(here))) else {
            eprintln!("GAHT_MAKE: no {what} in this world");
            return;
        };
        world.teleport_squad(at.add(V2::new(1.2, 0.8)));
        // One of them a little way off: named on the screen as too far.
        if let Some(k) = m.get(2).and_then(|&p| world.squad.index(p)) {
            let off = at.add(V2::new(22.0, 6.0));
            world.squad.at[k] = off;
            world.squad.goal[k] = off;
        }
    }
    let recipes: Vec<usize> = (0..RECIPES.len()).filter(|&i| RECIPES[i].station == station).collect();
    for (j, &p) in m.iter().take(3).enumerate() {
        let person = &mut world.people[p as usize];
        for &i in &recipes {
            let sk = RECIPES[i].skill;
            if let Some(d) = person.detail.as_mut() {
                if !d.crafts.contains(&sk) {
                    d.crafts.push(sk);
                }
            }
            person.stats.set_skill(sk, [42.0, 12.0, 30.0][j]);
        }
    }
    // Materials for the first few things, shared between two packs.
    for (n, &i) in recipes.iter().take(4).enumerate() {
        for &(k, need) in RECIPES[i].inputs {
            let to = m[if n % 2 == 0 { 0 } else { 1.min(m.len() - 1) }];
            if let Some(d) = world.people[to as usize].detail.as_mut() {
                d.gear.add(items::id(k), need);
            }
        }
    }
    // At an alchemy table: ingredients to brew with, a few tasted, and one
    // mix brewed already (so it's remembered).
    if station == Station::AlchemyTable {
        if let Some(d) = world.people[m[0] as usize].detail.as_mut() {
            for (k, n) in [("bloodleaf", 4), ("wild_berries", 3), ("sweetroot", 2), ("nightberry", 2), ("mirebloom", 2), ("ghostcap", 2)] {
                d.gear.add(items::id(k), n);
            }
        }
        world.taste(m[0], items::id("nightberry"));
        let _ = world.brew(m[0], &[items::id("bloodleaf"), items::id("wild_berries")], 1);
        world.step(gahturiyu_sim::sim::brewing::BREW_SECS + 5.0);
    }
    // At a fire or a stove: a pot, and the second of them already cooking
    // (the stirring cue).
    if matches!(station, Station::Campfire | Station::Stove) {
        if let Some(d) = world.people[m[0] as usize].detail.as_mut() {
            d.gear.add(items::id("cooking_pot"), 1);
        }
        if let (Some(&cook), Some(&ri)) = (m.get(1), recipes.first()) {
            if let Some(d) = world.people[cook as usize].detail.as_mut() {
                for &(k, need) in RECIPES[ri].inputs {
                    d.gear.add(items::id(k), need);
                }
            }
            let _ = world.start_craft(cook, ri);
        }
    }
    world.step(1.0);
}

/// Lay out a demo outpost near the squad (for `GAHT_BUILD`).
fn demo_base(world: &mut World, living: bool) {
    use gahturiyu_sim::sim::base::{def_index, Land, Plan, BUILDINGS};
    use gahturiyu_sim::sim::stats::Skill;
    let start = world.squad.pos;
    for ring in 0..60 {
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let p = start.add(V2::new(a.cos(), a.sin()).scale(80.0 + ring as f32 * 30.0));
            let mut w = world.clone();
            let Ok(bid) = w.found_base(p) else { continue };
            if w.base(bid).unwrap().land != Land::Wilds {
                continue;
            }
            let plan = |key: &str, off: V2, rot: f32| Plan { def: def_index(key), at: p.add(off), rot, w: BUILDINGS[def_index(key)].w, replaces: None };
            let layout = [plan("hut", V2::new(-9.0, 4.0), 0.0), plan("lean_to", V2::new(7.0, 5.0), 0.4), plan("hut", V2::new(-9.0, -6.0), 0.0), plan("field_plot", V2::new(10.0, -10.0), 0.0)];
            if !layout.iter().all(|pl| w.check_place(&mut pl.clone()).is_ok()) {
                continue;
            }
            let wall = [p.add(V2::new(-18.0, 14.0)), p.add(V2::new(18.0, 14.0))];
            if !World::wall_plans(def_index("palisade"), &wall).into_iter().all(|mut q| w.check_place(&mut q).is_ok()) {
                continue;
            }
            w.teleport_squad(p.add(V2::new(0.0, -2.0)));
            // Everyone can lend a hand with timber.
            let hunter = w.squad.members.iter().copied().find(|&m| w.people[m as usize].detail.as_ref().is_some_and(|d| d.crafts.contains(&Skill::Carpentry))).unwrap_or(w.squad.members[0]);
            for &m in &w.squad.members.clone() {
                if let Some(d) = w.people[m as usize].detail.as_mut() {
                    if !d.crafts.contains(&Skill::Carpentry) {
                        d.crafts.push(Skill::Carpentry);
                    }
                }
            }
            {
                let d = w.people[hunter as usize].detail.as_mut().unwrap();
                for (k, n) in [("timber", 60), ("seareed", 16), ("iron_ingot", 1)] {
                    d.gear.add(gahturiyu_sim::sim::items::id(k), n);
                }
            }
            for pl in layout {
                let _ = w.place_building(pl);
            }
            let _ = w.place_wall(def_index("palisade"), &wall);
            let mut g = plan("gate", V2::new(0.0, 14.0), 0.0);
            if w.check_place(&mut g).is_ok() {
                let _ = w.place_building(g);
            }
            w.store_materials(bid);
            // A few hours' work.
            let end = w.time + 7.0 * 3600.0;
            while w.time < end {
                w.step(60.0);
            }
            // Two of the squad stay on: a farmer and a builder, with food.
            if living {
                use gahturiyu_sim::sim::baselife::Job;
                let i = w.bases.iter().position(|b| b.id == bid).unwrap();
                w.bases[i].add_to_store(gahturiyu_sim::sim::items::id("flatbread"), 10);
                let m = w.squad.members.clone();
                for (k, job) in [(1usize, Job::Farmer), (2, Job::Builder)] {
                    if w.leave_at_base(m[k], bid).is_ok() {
                        w.set_base_job(m[k], job);
                    }
                }
                // And a hand hired from the nearest town, paid from the squad's purse.
                let squad_lead = w.squad.members[0];
                if let Some(d) = w.people[squad_lead as usize].detail.as_mut() {
                    d.gear.add(gahturiyu_sim::sim::items::id("coin"), 120);
                }
                let base_at = w.bases[i].at;
                let hand = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).filter(|&p| w.hire_terms(p).is_some()).min_by(|&a, &b| w.person_pos(a).dist(base_at).total_cmp(&w.person_pos(b).dist(base_at)));
                if let Some(h) = hand {
                    if w.hire(h).is_ok() {
                        w.set_base_job(h, Job::Hauler);
                    }
                }
                let end = w.time + 30.0 * 3600.0;
                while w.time < end {
                    w.step(120.0);
                }
            }
            *world = w;
            return;
        }
    }
    eprintln!("GAHT_BUILD: nowhere to lay a demo base");
}

/// Member 0 seen stealing in a town with its watch out, the theft told, and
/// the guard set off after them.
fn chase_demo(world: &mut World) {
    use gahturiyu_sim::sim::{containers::Owner, jobs::Job, law::Wrong};
    while world.time < 10.0 * gahturiyu_sim::sim::world::HOUR {
        world.step(60.0);
    }
    let t = world.time;
    let near = world.squad.pos;
    let Some(town) = world
        .settlements
        .iter()
        .filter(|s| s.residents.iter().any(|&p| !world.people[p as usize].dead && world.life(p).job == Job::Guard && world.at_work(p, t)) && s.residents.len() > 40)
        .min_by(|a, b| a.pos.dist(near).total_cmp(&b.pos.dist(near)))
        .map(|s| s.id)
    else {
        return;
    };
    world.teleport_squad(world.settlements[town as usize].pos);
    world.step(0.5);
    let me = world.squad.members[0];
    let folk: Vec<_> = world.residents_in_band1(town).into_iter().filter(|&p| world.building_at(world.person_pos(p)).is_none() && world.life(p).job != Job::Guard).collect();
    for f in folk {
        let Some(h) = world.life(f).household else { continue };
        let at = world.person_pos(f).add(V2::new(2.0, 0.0));
        world.teleport_squad(at.add(V2::new(4.0, 3.0)));
        if let Some(k) = world.squad.index(me) {
            world.squad.at[k] = at;
            world.squad.goal[k] = at;
            world.squad.route[k].clear();
        }
        world.wrong_seen(me, town, Wrong::Theft, 30.0, "Seen stealing!".into(), Some(f), Some(Owner::Household(h)), at);
        if !world.pursuits.is_empty() {
            break;
        }
    }
    // Until the guard is close.
    for _ in 0..4000 {
        let near = world.pursuits.first().and_then(|p| p.pos).is_some_and(|g| g.dist(world.person_pos(me)) < 9.0);
        if world.pursuits.is_empty() || near {
            break;
        }
        world.step(0.1);
    }
}
