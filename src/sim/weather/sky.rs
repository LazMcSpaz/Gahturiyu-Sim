//! A region's weather at a moment: the part that is the same for the whole
//! region, before a particular spot's height and hollows are taken into
//! account (`local.rs` does that).
//!
//! Weather comes in off the sea. The country shares its big weather: one
//! set of slow curves and one run of storms for the whole world, which each
//! region takes its own share of (`shared` in the climate file), and which
//! reach places further east a little later (`lag`). So a storm darkens the
//! sea, then the coast, then the hills; the plateau may miss it altogether.
//!
//! Three kinds of thing are laid over each other, all fixed by the world
//! seed, the region and the time asked about:
//!
//! - **Slow curves** (`noise::even`) for the everyday cloud, rain, wind,
//!   temperature and wind direction, shaped by the season's table.
//! - **Storms.** Each day may bring one (one keyed roll per region and day,
//!   likelier in winter and in an unsettled spell). A storm has a start, a
//!   build, a worst and a clearing; the wind answers it first, then the
//!   cloud, then the rain.
//! - **Fog mornings.** Each day may start in fog or mist (one keyed roll,
//!   weighted towards the calmest mornings and the morning after a storm).
//!   It forms in the small hours and burns off before noon, unless the day
//!   is grey, when it hangs on.
//!
//! Wet ground and the state of the sea remember the last few hours: they are
//! worked out by looking back along the same curves, not by keeping a tally.

use super::climate::{climate, seasonal, year_phase, Climate, RegionClimate};
use super::noise::{even, smooth};
use super::Region;
use crate::sim::rng::{self, Rng};
use crate::sim::world::{DAY, HOUR};

/// Marks every weather roll, so none can coincide with another system's.
pub(super) const WEATHER: u64 = 0x5745_4154_4845;
/// Stands in for a region in the keys of what the whole world shares.
const WORLD: u64 = 99;

const SPELL: u64 = 1;
const CLOUD: u64 = 2;
const RAIN: u64 = 3;
const WIND: u64 = 4;
const TEMP: u64 = 5;
const TURN: u64 = 6;
const BASE: u64 = 7;
const SQUALL: u64 = 8;
const STORM: u64 = 9;
const FOG: u64 = 10;

/// The weather over a whole region at one moment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sky {
    /// Share of the sky covered, 0..1.
    pub cloud: f32,
    /// Rain or snow falling, 0..1 (which it is depends on the height).
    pub precip: f32,
    /// Wind speed, metres a second.
    pub wind: f32,
    /// The way the wind blows towards (x east, y south), scaled by its speed.
    pub wind_x: f32,
    pub wind_y: f32,
    /// How gusty, 0..1.
    pub gust: f32,
    /// Ground fog, 0..1, where it lies.
    pub fog: f32,
    /// How thick the fog layer is, metres above the low ground.
    pub fog_depth: f32,
    /// Height of the cloud base above the sea, metres.
    pub cloud_base: f32,
    /// Temperature as if at sea level, deg C.
    pub temp0: f32,
    /// How much of a storm is on, 0..1.
    pub storm: f32,
    /// Lightning flashes a minute.
    pub lightning: f32,
    /// The sea, 0 flat calm .. 1 heavy surf.
    pub sea: f32,
    /// How wet the ground is, 0..1.
    pub wetness: f32,
    /// Snow lies on ground above this height (metres above the sea): where
    /// the last few days have been cold enough, whatever is falling now.
    /// Below zero means it lies everywhere.
    pub snow_lying: f32,
}

impl Sky {
    /// `self` plus `share` of `o` (for mixing regions at a border).
    pub(super) fn add(&mut self, o: &Sky, share: f32) {
        self.cloud += o.cloud * share;
        self.precip += o.precip * share;
        self.wind += o.wind * share;
        self.wind_x += o.wind_x * share;
        self.wind_y += o.wind_y * share;
        self.gust += o.gust * share;
        self.fog += o.fog * share;
        self.fog_depth += o.fog_depth * share;
        self.cloud_base += o.cloud_base * share;
        self.temp0 += o.temp0 * share;
        self.storm += o.storm * share;
        self.lightning += o.lightning * share;
        self.sea += o.sea * share;
        self.wetness += o.wetness * share;
        self.snow_lying += o.snow_lying * share;
    }
}

/// One storm: when it starts, how it builds and clears, how bad it gets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Storm {
    pub start: f64,
    /// Hours.
    pub build: f32,
    pub hold: f32,
    pub clear: f32,
    pub strength: f32,
    pub thunder: bool,
}

impl Storm {
    /// How much of the storm is on at `t`, 0..strength.
    pub fn at(&self, t: f64) -> f32 {
        let x = ((t - self.start) / HOUR) as f32;
        let end = self.build + self.hold + self.clear;
        if x <= 0.0 || x >= end {
            return 0.0;
        }
        self.strength * smooth(0.0, self.build, x) * smooth(end, self.build + self.hold, x)
    }

    pub fn end(&self) -> f64 {
        self.start + (self.build + self.hold + self.clear) as f64 * HOUR
    }
}

/// A morning's fog: when it forms and goes, how thick, how deep.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FogBank {
    pub start: f64,
    /// Hours to thicken.
    pub rise: f32,
    /// When it starts to go.
    pub end: f64,
    /// Hours to fade.
    pub fade: f32,
    pub peak: f32,
    pub depth: f32,
}

impl FogBank {
    pub fn at(&self, t: f64) -> f32 {
        let up = ((t - self.start) / HOUR) as f32;
        let down = ((t - self.end) / HOUR) as f32;
        if up <= 0.0 || down >= self.fade {
            return 0.0;
        }
        self.peak * smooth(0.0, self.rise, up) * smooth(self.fade, 0.0, down)
    }
}

/// The share of the time `0.6 a + 0.4 b` is below `z`, for two even rolls
/// `a` and `b`, turned round: the `z` it is below for a `share` of the time.
fn blend_cut(share: f32) -> f32 {
    let (a, b) = (0.6f32, 0.4f32);
    let share = share.clamp(0.0, 1.0);
    let low = b / (2.0 * a);
    if share <= low {
        (share * 2.0 * a * b).sqrt()
    } else if share <= 1.0 - low {
        share * a + b / 2.0
    } else {
        1.0 - ((1.0 - share) * 2.0 * a * b).sqrt()
    }
}

/// Two even rolls mixed, `a` of the first and the rest of the second, are
/// no longer even (they bunch in the middle). This is the share of the time
/// such a mix is below `z`: putting the mix through it makes it even again.
fn even_again(a: f32, z: f32) -> f32 {
    let (hi, lo) = (a.max(1.0 - a), a.min(1.0 - a));
    let z = z.clamp(0.0, 1.0);
    if lo < 1e-4 {
        z
    } else if z < lo {
        z * z / (2.0 * hi * lo)
    } else if z <= hi {
        (z - lo / 2.0) / hi
    } else {
        1.0 - (1.0 - z) * (1.0 - z) / (2.0 * hi * lo)
    }
}

/// A storm coming in over the whole country on a day: its timing and its
/// power, before any region has caught it (or not).
struct Front {
    start: f64,
    build: f32,
    hold: f32,
    clear: f32,
    /// 0 the weakest .. 1 the strongest.
    power: f32,
    /// Its place among storms for thunder: low means even regions where
    /// thunder is rare will hear it.
    thunder: f32,
    /// How far it reaches: low means it covers the whole country, high
    /// that only the stormiest regions get it.
    reach: f32,
    /// Storm days a month the stormiest region has at that time of year.
    days: f32,
}

/// The storm that comes in on a day, if one does. `spell` is how unsettled
/// the weather is (0..1).
fn front(seed: u64, day: i64, spell: f32) -> Option<Front> {
    let c = climate();
    let s = &c.storms;
    let phase = year_phase((day as f64 + 0.5) * DAY);
    let days = c.regions.iter().map(|r| seasonal(&r.storm_days, phase)).fold(0.0, f32::max);
    // A storm is at its worst for about this long, and so darkens this
    // many calendar days on average.
    let worst = (s.hold.0 + s.hold.1) / 2.0 + (s.build.0 + s.build.1 + s.clear.0 + s.clear.1) / 4.0;
    let p = days / 30.0 * (1.0 + c.unsettled * (2.0 * spell - 1.0)) / (1.0 + worst / 24.0);
    let mut r = Rng::from_keys(&[seed, WEATHER, WORLD, STORM, day as u64]);
    if !r.chance(p.min(0.95)) {
        return None;
    }
    let start = day as f64 * DAY + r.f64() * DAY;
    let (build, hold, clear) = (lerp(s.build, r.f32()), lerp(s.hold, r.f32()), lerp(s.clear, r.f32()));
    Some(Front { start, build, hold, clear, power: r.f32().powf(1.5), thunder: r.f32(), reach: r.f32(), days })
}

/// Whether any storm is over the country at some point between two moments
/// (with a few hours to spare either side). A quick way to know there can
/// be no lightning.
pub(super) fn stormy_between(seed: u64, from: f64, to: f64) -> bool {
    let c = climate();
    let spare = (c.crossing_hours + c.storms.wind_leads + 1.0) as f64 * HOUR;
    let longest = (c.storms.build.1 + c.storms.hold.1 + c.storms.clear.1) as f64 * HOUR;
    let first = ((from - spare - longest) / DAY).floor() as i64;
    let last = ((to + spare) / DAY).floor() as i64;
    (first..=last).any(|day| {
        let spell = even(rng::key(&[seed, WEATHER, WORLD, SPELL]), (day as f64 + 0.5) * DAY, c.spell_days as f64 * DAY);
        front(seed, day, spell).map(|f| f.start - spare < to && f.start + longest + spare > from).unwrap_or(false)
    })
}

fn lerp(r: (f32, f32), x: f32) -> f32 {
    r.0 + (r.1 - r.0) * x
}

/// Everything needed to work out one region's weather.
pub struct Maker {
    c: &'static Climate,
    rc: &'static RegionClimate,
    seed: u64,
    region: Region,
    /// How far behind the sea's edge the weather runs here, seconds.
    lag: f64,
}

impl Maker {
    /// For a region's weather as it comes in off the sea.
    pub fn new(seed: u64, region: Region) -> Maker {
        Maker::lagging(seed, region, 0.0)
    }

    /// For a region's weather somewhere it arrives `lag` seconds later.
    pub fn lagging(seed: u64, region: Region, lag: f64) -> Maker {
        let c = climate();
        Maker { c, rc: c.of(region), seed, region, lag }
    }

    /// A slow curve, 0..1, even over the year: part the whole world's, part
    /// this region's own, read as it has drifted in to here.
    fn curve(&self, what: u64, t: f64, period: f64, shared: f32) -> f32 {
        let t = t - self.lag;
        let own = if shared < 1.0 { even(self.key(what), t, period) } else { 0.0 };
        if shared <= 0.0 {
            return own;
        }
        let world = even(rng::key(&[self.seed, WEATHER, WORLD, what]), t, period);
        even_again(shared, shared * world + (1.0 - shared) * own)
    }

    /// How much storm is on here at `t`.
    fn storming(&self, storms: &[Option<Storm>], t: f64) -> f32 {
        level(storms, t - self.lag)
    }

    fn key(&self, what: u64) -> u64 {
        rng::key(&[self.seed, WEATHER, self.region as u64, what])
    }

    fn roll(&self, what: u64, day: i64) -> Rng {
        Rng::from_keys(&[self.seed, WEATHER, self.region as u64, what, day as u64])
    }

    /// 0 in a settled spell, 1 in an unsettled one. The whole country is in
    /// the same spell (unless the regions share nothing).
    fn spell(&self, t: f64) -> f32 {
        self.curve(SPELL, t, self.c.spell_days as f64 * DAY, if self.c.shared > 0.0 { 1.0 } else { 0.0 })
    }

    /// The storm that reaches this region from the front that comes in on a
    /// day, if one comes and the region catches it. (Days here are counted
    /// at the sea's edge; the storm arrives `lag` later.)
    pub fn storm_on(&self, day: i64) -> Option<Storm> {
        let mid = (day as f64 + 0.5) * DAY;
        let spell = even(rng::key(&[self.seed, WEATHER, WORLD, SPELL]), mid, self.c.spell_days as f64 * DAY);
        let f = front(self.seed, day, spell)?;
        let phase = year_phase(mid);
        // The stormiest region catches every front; the rest catch their
        // share, the big ones more surely than the small. Mostly it is the
        // front's own reach that settles it, so a storm that gets as far as
        // the sheltered country has crossed the exposed country on its way.
        let share = seasonal(&self.rc.storm_days, phase).max(0.0) / f.days.max(1e-3);
        let mut r = self.roll(STORM, day);
        let reach = even_again(0.85, 0.85 * f.reach + 0.15 * r.f32());
        if reach >= (share * (0.6 + 0.8 * f.power)).min(1.0) {
            return None;
        }
        let strength = lerp(self.c.storms.strength, f.power) * (0.85 + 0.15 * r.f32());
        let thunder = f.thunder < seasonal(&self.rc.thunder, phase) * (0.6 + 0.8 * f.power);
        Some(Storm { start: f.start, build: f.build, hold: f.hold, clear: f.clear, strength, thunder })
    }

    /// The storms that could matter at any time this module looks at when
    /// asked about `t` (it looks back most of a day).
    fn storms_round(&self, t: f64) -> [Option<Storm>; 5] {
        let day = ((t - self.lag) / DAY).floor() as i64;
        [self.storm_on(day - 3), self.storm_on(day - 2), self.storm_on(day - 1), self.storm_on(day), self.storm_on(day + 1)]
    }

    /// The everyday cloud, before any storm.
    fn cloud_plain(&self, t: f64) -> f32 {
        let phase = year_phase(t);
        let clear = seasonal(&self.rc.clear, phase).clamp(0.01, 0.9);
        let grey = seasonal(&self.rc.overcast, phase).clamp(0.01, 0.98 - clear);
        let u = self.curve(CLOUD, t, self.c.cloud_hours as f64 * HOUR, self.c.shared);
        if u < clear {
            0.25 * u / clear
        } else if u < 1.0 - grey {
            0.25 + 0.45 * (u - clear) / (1.0 - grey - clear)
        } else {
            0.7 + 0.3 * (u - (1.0 - grey)) / grey
        }
    }

    /// The everyday wind speed, before storms and fog.
    fn wind_plain(&self, t: f64) -> f32 {
        seasonal(&self.rc.wind, year_phase(t)).max(0.0) * (0.4 + 1.2 * self.curve(WIND, t, self.c.wind_hours as f64 * HOUR, self.c.shared))
    }

    /// The fog or mist a day starts in, if any.
    pub fn fog_on(&self, day: i64, storms: &[Option<Storm>]) -> Option<FogBank> {
        let f = &self.c.fog;
        let dawn = day as f64 * DAY + 5.0 * HOUR;
        let phase = year_phase(dawn);
        let at = |t: f64| self.storming(storms, t);
        // No fog in a storm; likelier on the morning after one.
        if at(dawn + self.c.storms.wind_leads as f64 * HOUR) > 0.25 {
            return None;
        }
        let after = at(dawn - 4.0 * HOUR).max(at(dawn - 8.0 * HOUR)).max(at(dawn - 12.0 * HOUR));
        let thick = seasonal(&self.rc.fog_mornings, phase).max(0.0) / 30.0;
        let mist = seasonal(&self.rc.mist_mornings, phase).max(0.0) / 30.0;
        // How calm the morning is, as its place among all mornings (0 = the
        // calmest of the year), mixed with plain luck: fog comes on the
        // calmest mornings, as often as the table says.
        let calm = self.curve(WIND, dawn, self.c.wind_hours as f64 * HOUR, self.c.shared);
        let mut r = self.roll(FOG, day);
        // The luck is partly the whole country's too: a foggy morning on the
        // coast is likely a misty one inland.
        let ours = Rng::from_keys(&[self.seed, WEATHER, WORLD, FOG, day as u64]).f32();
        let luck = even_again(self.c.shared, self.c.shared * ours + (1.0 - self.c.shared) * r.f32());
        let z = 0.6 * calm + 0.4 * luck - 0.15 * f.after_storm * after;
        let peak = if z < blend_cut(thick) {
            0.78 + 0.22 * r.f32()
        } else if z < blend_cut(thick + mist) {
            0.3 + 0.3 * r.f32()
        } else {
            return None;
        };
        let start = day as f64 * DAY + lerp(f.forms, r.f32()) as f64 * HOUR;
        let rise = lerp(f.thickens_over, r.f32());
        let grey = self.cloud_plain(day as f64 * DAY + 12.0 * HOUR) > 0.7;
        let (burn, linger) = (lerp(f.burns_off, r.f32()), lerp(f.lingers_until, r.f32()));
        let end = day as f64 * DAY + if grey { linger } else { burn } as f64 * HOUR;
        let fade = lerp(f.fades_over, r.f32());
        let depth = self.rc.fog_depth * (0.6 + 0.9 * r.f32()) * if r.chance(self.rc.deep_fog) { 10.0 } else { 1.0 };
        Some(FogBank { start, rise, end, fade, peak, depth })
    }

    /// Cloud with the storms' share.
    fn cloud(&self, t: f64, storms: &[Option<Storm>]) -> f32 {
        let plain = self.cloud_plain(t);
        plain + (1.0 - plain) * self.storming(storms, t + self.c.storms.cloud_leads as f64 * HOUR)
    }

    /// Rain or snow falling at `t`.
    fn precip(&self, t: f64, storms: &[Option<Storm>]) -> f32 {
        let gate = smooth(0.62, 0.8, self.cloud_plain(t));
        let share = (seasonal(&self.rc.rain, year_phase(t)) * (1.0 + 0.6 * self.c.unsettled * (2.0 * self.spell(t) - 1.0))).clamp(0.02, 0.95);
        let u = self.curve(RAIN, t, self.c.rain_hours as f64 * HOUR, self.c.shared * 0.8);
        let ramp = ((u - (1.0 - share)) / share).max(0.0);
        let plain = gate * 0.55 * ramp.powf(2.2);
        let squall = 0.8 + 0.2 * self.curve(SQUALL, t, 0.7 * HOUR, 0.0);
        plain + (1.0 - plain) * self.storming(storms, t).powf(1.2) * squall
    }

    /// Wind speed with the storms' share (but not yet stilled by fog).
    fn wind(&self, t: f64, storms: &[Option<Storm>]) -> f32 {
        let plain = self.wind_plain(t);
        let s = self.storming(storms, t + self.c.storms.wind_leads as f64 * HOUR);
        plain + s * (self.c.storms.wind.max(plain) - plain)
    }

    /// The region's weather at `t`.
    pub fn sky(&self, t: f64) -> Sky {
        let c = self.c;
        let storms = self.storms_round(t);
        let day = (t / DAY).floor() as i64;
        let hour = (t.rem_euclid(DAY) / HOUR) as f32;

        let s_rain = self.storming(&storms, t);
        let s_cloud = self.storming(&storms, t + c.storms.cloud_leads as f64 * HOUR);
        let s_wind = self.storming(&storms, t + c.storms.wind_leads as f64 * HOUR);
        let storm = 0.5 * s_rain + 0.3 * s_cloud + 0.2 * s_wind;

        let cloud = self.cloud(t, &storms);
        let precip = self.precip(t, &storms);
        let blow = self.wind(t, &storms);

        // Fog: this morning's bank (or the last of yesterday's), thinned if
        // the wind gets up.
        let (mut fog, mut fog_depth) = (0.0f32, self.rc.fog_depth);
        for bank in [self.fog_on(day - 1, &storms), self.fog_on(day, &storms)].iter().flatten() {
            let here = bank.at(t) * smooth(c.fog.breezy + 3.0, c.fog.calm + 2.0, blow);
            if here > fog {
                fog = here;
                fog_depth = bank.depth;
            }
        }
        let wind = blow * (1.0 - c.fog.stills_wind * fog);

        // Which way it blows: from its usual quarter, swinging slowly, and
        // veering in a storm. Compass degrees, 0 = from the north.
        let from = self.rc.wind_from + c.wind_swing * (2.0 * self.curve(TURN, t, 30.0 * HOUR, c.shared) - 1.0) + 25.0 * s_wind;
        let to = (from + 180.0).to_radians();
        let (wind_x, wind_y) = (to.sin() * wind, -to.cos() * wind);

        let base = lerp(self.rc.cloud_base, self.curve(BASE, t, 11.0 * HOUR, c.shared * 0.7));
        let cloud_base = base - 0.5 * (base - self.rc.cloud_base.0) * s_cloud;

        let phase = year_phase(t);
        let wander = c.temp_wander * (2.0 * self.curve(TEMP, t, 3.0 * DAY, c.shared) - 1.0);
        let swing = seasonal(&self.rc.swing, phase) / 2.0 * (1.0 - 0.6 * cloud) * ((hour - 15.0) / 24.0 * std::f32::consts::TAU).cos();
        let usual = seasonal(&self.rc.temp, phase) + wander;
        let temp0 = usual + swing - c.storms.chill * s_cloud;
        // Snow lies where the days have been cold enough, hour by hour aside.
        // (Below zero in a hard cold spell: it lies right down to the shore.)
        let snow_lying = (usual - c.snow_temp) / c.lapse;

        let mut thunder = 0.0f32;
        for s in storms.iter().flatten() {
            if s.thunder {
                thunder = thunder.max(s.at(t - self.lag));
            }
        }
        let lightning = c.storms.flashes * thunder * thunder;

        // The sea answers the last few hours' wind, not just this minute's.
        let mut swell = 0.0;
        for (back, share) in [(0.0, 0.35), (2.0, 0.3), (4.0, 0.2), (7.0, 0.15)] {
            swell += share * self.wind(t - back * HOUR, &storms);
        }
        let sea = (smooth(c.sea_calm, c.sea_rough, swell) * self.rc.surf).min(1.0);

        // Wet ground: the rain of the last half day, the older the fainter.
        const BACK: [f32; 10] = [0.0, 0.5, 1.0, 2.0, 3.0, 4.5, 6.0, 8.0, 10.0, 13.0];
        let mut soaked = 0.0;
        let mut prev = (0.0f32, precip);
        for &b in &BACK[1..] {
            let now = (b, self.precip(t - b as f64 * HOUR, &storms) * (-b / c.dry_hours).exp());
            soaked += (now.0 - prev.0) * (now.1 + prev.1) / 2.0;
            prev = now;
        }
        let wetness = 1.0 - (-c.wetting * soaked).exp();

        Sky { cloud, precip, wind, wind_x, wind_y, gust: (0.15 + 0.55 * s_wind + 0.1 * (blow / 15.0).min(1.0)).min(1.0), fog, fog_depth, cloud_base, temp0, storm, lightning, sea, wetness, snow_lying }
    }
}

/// How much storm is on at `t`, all storms together, 0..1.
fn level(storms: &[Option<Storm>], t: f64) -> f32 {
    let mut calm = 1.0;
    for s in storms.iter().flatten() {
        calm *= 1.0 - s.at(t);
    }
    1.0 - calm
}

/// A region's weather at a moment, as it comes in off the sea.
pub fn sky(seed: u64, region: Region, t: f64) -> Sky {
    Maker::new(seed, region).sky(t)
}

/// A region's weather at a moment, somewhere it arrives `lag` seconds later.
pub fn sky_at(seed: u64, region: Region, t: f64, lag: f64) -> Sky {
    Maker::lagging(seed, region, lag).sky(t)
}
