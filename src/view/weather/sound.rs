//! The weather's sounds: which there are, and how loud each should be.
//!
//! There are no sound files yet and nothing is played. This is the list of
//! slots a file can be dropped into, and the working-out of how loud each
//! one is from the weather where the camera is, so that playing them is
//! only a matter of wiring: read `WeatherView::sound` each frame, set each
//! loop's volume, and start each one-shot in `played`.


// Nothing plays these yet: the file names and lengths are for whoever does.
#![allow(dead_code)]

use gahturiyu_sim::sim::weather::Weather;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    RainLight,
    RainHeavy,
    WindLow,
    WindHigh,
    Gale,
    SurfCalm,
    SurfHeavy,
    SnowWind,
    ThunderNear,
    ThunderFar,
    RainOnRoof,
}

/// The sounds that run on and on, each at its own changing volume.
pub const LOOPS: [Slot; 8] = [Slot::RainLight, Slot::RainHeavy, Slot::WindLow, Slot::WindHigh, Slot::Gale, Slot::SurfCalm, Slot::SurfHeavy, Slot::SnowWind];
/// The sounds played once, when something happens.
pub const ONE_SHOTS: [Slot; 3] = [Slot::ThunderNear, Slot::ThunderFar, Slot::RainOnRoof];

impl Slot {
    pub fn name(self) -> &'static str {
        match self {
            Slot::RainLight => "light rain",
            Slot::RainHeavy => "heavy rain",
            Slot::WindLow => "low wind",
            Slot::WindHigh => "high wind",
            Slot::Gale => "gale",
            Slot::SurfCalm => "calm surf",
            Slot::SurfHeavy => "heavy surf",
            Slot::SnowWind => "snow wind",
            Slot::ThunderNear => "thunder, near",
            Slot::ThunderFar => "thunder, far",
            Slot::RainOnRoof => "rain on a roof",
        }
    }

    /// Where the file for this slot is to go, under `assets/`.
    pub fn file(self) -> &'static str {
        match self {
            Slot::RainLight => "sounds/weather/rain_light.ogg",
            Slot::RainHeavy => "sounds/weather/rain_heavy.ogg",
            Slot::WindLow => "sounds/weather/wind_low.ogg",
            Slot::WindHigh => "sounds/weather/wind_high.ogg",
            Slot::Gale => "sounds/weather/gale.ogg",
            Slot::SurfCalm => "sounds/weather/surf_calm.ogg",
            Slot::SurfHeavy => "sounds/weather/surf_heavy.ogg",
            Slot::SnowWind => "sounds/weather/snow_wind.ogg",
            Slot::ThunderNear => "sounds/weather/thunder_near.ogg",
            Slot::ThunderFar => "sounds/weather/thunder_far.ogg",
            Slot::RainOnRoof => "sounds/weather/rain_on_roof.ogg",
        }
    }

    /// A sensible length for the file, seconds (loops want to be long
    /// enough not to be heard coming round).
    pub fn seconds(self) -> f32 {
        match self {
            Slot::RainLight | Slot::RainHeavy => 30.0,
            Slot::WindLow | Slot::WindHigh | Slot::Gale | Slot::SnowWind => 40.0,
            Slot::SurfCalm | Slot::SurfHeavy => 45.0,
            Slot::ThunderNear => 6.0,
            Slot::ThunderFar => 10.0,
            Slot::RainOnRoof => 20.0,
        }
    }

    pub fn looping(self) -> bool {
        LOOPS.contains(&self)
    }
}

/// The speed of sound, metres a second: thunder comes this much after its flash.
pub const SOUND_SPEED: f32 = 343.0;
/// Thunder nearer than this is the crack; further, the roll.
pub const NEAR_THUNDER: f32 = 1500.0;
/// How much of the weather's sound gets indoors.
pub const INDOORS: f32 = 0.35;

/// What the listener's situation is.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ears {
    /// Under a roof: everything is muffled, and rain drums overhead.
    pub indoors: bool,
    /// 1 on the shore .. 0 out of earshot of the sea.
    pub by_sea: f32,
    /// 1 down among it .. 0.3 from far overhead.
    pub closeness: f32,
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How loud each loop should be, 0..1, in `LOOPS` order.
pub fn volumes(w: &Weather, ears: Ears) -> [f32; 8] {
    let rain = w.rain;
    let wind = w.wind;
    let mut v = [
        // Light rain gives way to heavy.
        smooth(0.02, 0.15, rain) * (1.0 - smooth(0.35, 0.6, rain)),
        smooth(0.3, 0.7, rain),
        // The wind steps up through three voices.
        smooth(2.0, 8.0, wind) * (1.0 - smooth(11.0, 16.0, wind)),
        smooth(8.0, 14.0, wind) * (1.0 - smooth(17.0, 22.0, wind)),
        smooth(15.0, 22.0, wind),
        ears.by_sea * (0.35 + 0.65 * smooth(0.0, 0.3, w.sea)) * (1.0 - smooth(0.35, 0.7, w.sea)),
        ears.by_sea * smooth(0.35, 0.8, w.sea),
        smooth(0.08, 0.5, w.snow) * smooth(4.0, 12.0, wind),
    ];
    let k = ears.closeness * if ears.indoors { INDOORS } else { 1.0 };
    for x in &mut v {
        *x = (*x * k).clamp(0.0, 1.0);
    }
    v
}

/// The sounds as they stand this frame.
#[derive(Clone, Debug, Default)]
pub struct Mixer {
    /// Each loop's volume, in `LOOPS` order.
    pub volumes: [f32; 8],
    /// Play everything dulled (the listener is under a roof).
    pub muffled: bool,
    /// One-shots waiting for their moment: (when on `WeatherView::clock`,
    /// which, how loud). Thunder waits here while its sound travels.
    pub due: Vec<(f64, Slot, f32)>,
    /// One-shots to start this frame.
    pub played: Vec<(Slot, f32)>,
    /// The last one-shot started, and when (for the readout).
    pub last: Option<(Slot, f64)>,
    roof_next: f64,
}

impl Mixer {
    /// Thunder from a strike `distance` metres off, heard when the sound arrives.
    pub fn thunder(&mut self, struck_at: f64, distance: f32, power: f32) {
        let slot = if distance < NEAR_THUNDER { Slot::ThunderNear } else { Slot::ThunderFar };
        let loud = (power * (1.0 - smooth(1000.0, 9000.0, distance) * 0.85)).clamp(0.0, 1.0);
        self.due.push((struck_at + (distance / SOUND_SPEED) as f64, slot, loud));
    }

    pub fn update(&mut self, w: &Weather, ears: Ears, clock: f64) {
        self.volumes = volumes(w, ears);
        self.muffled = ears.indoors;
        self.played.clear();
        let k = if ears.indoors { INDOORS } else { 1.0 };
        let mut i = 0;
        while i < self.due.len() {
            if self.due[i].0 <= clock {
                let (_, slot, loud) = self.due.swap_remove(i);
                self.played.push((slot, loud * k));
            } else {
                i += 1;
            }
        }
        if let Some(&(slot, _)) = self.played.last() {
            self.last = Some((slot, clock));
        }
        // Under a roof in the rain: the drumming overhead, started again as it runs out.
        if ears.indoors && w.rain > 0.08 {
            if clock >= self.roof_next {
                self.played.push((Slot::RainOnRoof, smooth(0.08, 0.7, w.rain)));
                self.roof_next = clock + (Slot::RainOnRoof.seconds() - 1.0) as f64;
            }
        } else {
            self.roof_next = 0.0;
        }
    }

    /// The loudest few loops, for the readout.
    pub fn loudest(&self) -> Vec<(Slot, f32)> {
        let mut v: Vec<(Slot, f32)> = LOOPS.iter().copied().zip(self.volumes).filter(|x| x.1 > 0.03).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v.truncate(4);
        v
    }
}
