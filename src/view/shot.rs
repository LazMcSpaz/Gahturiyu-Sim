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
    /// `GAHT_TOWN=1`: open the town panel for the nearest town.
    pub town: bool,
    /// `GAHT_TRADE=1`: talk to the nearest merchant at work and look at their wares.
    pub trade: bool,
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
            trade: var("GAHT_TRADE").is_some(),
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
