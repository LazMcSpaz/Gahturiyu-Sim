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
    /// `GAHT_TALK=1`: talk to the nearest townsperson.
    pub talk: bool,
    /// `GAHT_STARVE=1`: the squad is starving (no food, hunger 92).
    pub starve: bool,
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
    /// `GAHT_DUEL=1`: squad member 0 is judged by duel in the nearest town.
    pub duel: bool,
    /// `GAHT_SHUN=1`: the nearest coastal town's stilt village withdraws;
    /// the world runs on past the next dawn.
    pub shun: bool,
    /// `GAHT_TRADE=1`: talk to the nearest merchant at work and look at their wares.
    pub trade: bool,
    /// `GAHT_CONVO=1`: a local who was robbed last night talks about it
    /// (a conversation put together from the dialogue pieces).
    pub convo: bool,
    /// `GAHT_GUARD=1`: squad member 0 on a guard contract, standing at a
    /// merchant's stall in its hours.
    pub guard: bool,
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
            zoom: var("GAHT_ZOOM").and_then(|v| v.parse().ok()),
            speed: var("GAHT_SPEED").and_then(|v| v.parse().ok()),
            view: var("GAHT_VIEW"),
            pitch: var("GAHT_PITCH").and_then(|v| v.parse().ok()),
            yaw: var("GAHT_YAW").and_then(|v| v.parse().ok()),
            bandits: var("GAHT_BANDITS").and_then(|v| v.parse().ok()),
            camp: var("GAHT_CAMP").and_then(|v| v.parse().ok()),
            wait: var("GAHT_WAIT").and_then(|v| v.parse().ok()),
            hours: var("GAHT_HOURS").and_then(|v| v.parse().ok()),
            sneak: var("GAHT_SNEAK").is_some(),
            enter: var("GAHT_ENTER").is_some(),
            craft: var("GAHT_CRAFT").and_then(|v| v.parse().ok()),
            talk: var("GAHT_TALK").is_some(),
            starve: var("GAHT_STARVE").is_some(),
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
            town_kind: var("GAHT_TOWN").filter(|v| v != "1"),
            duel: var("GAHT_DUEL").is_some(),
            shun: var("GAHT_SHUN").is_some(),
            trade: var("GAHT_TRADE").is_some(),
            convo: var("GAHT_CONVO").is_some(),
            guard: var("GAHT_GUARD").is_some(),
            feud: var("GAHT_FEUD").is_some(),
            edit: var("GAHT_EDIT").is_some(),
            society: var("GAHT_SOCIETY"),
            nudge: pair("GAHT_NUDGE"),
        })
    }

    /// Where the camera should look instead of at the squad, if anywhere.
    pub fn focus(&self, world: &World) -> Option<V2> {
        use gahturiyu_sim::sim::routine::Doing;
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
        if let Some((dx, dy)) = self.nudge {
            world.teleport_squad(world.squad.pos.add(V2::new(dx, dy)));
            world.step(0.001);
        }
        if let Some(h) = self.hours {
            let end = world.time + h * 3600.0;
            while world.time < end {
                world.step(60.0);
            }
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
            let npc = world.residents_in_band1(town).into_iter().min_by(|&a, &b| world.person_pos(a).dist(here).total_cmp(&world.person_pos(b).dist(here)));
            if let Some(npc) = npc {
                world.order_talk(lead, npc);
                for _ in 0..240 {
                    if world.talk.is_some() {
                        break;
                    }
                    world.step(0.5);
                }
                world.ask(Topic::ThisTown);
                world.ask(Topic::Bandits);
                if world.topics().contains(&Topic::Work) {
                    world.ask(Topic::Work);
                }
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
                for slot in [Slot::MainHand, Slot::Body, Slot::Back] {
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
    }
}
