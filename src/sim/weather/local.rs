//! The weather at one spot: the region's sky brought down to the ground
//! there. Height makes it colder (and turns rain to snow), hollows hold the
//! fog while the hills above stand clear, and high ground under a low cloud
//! base is in hill fog.

use super::climate::climate;
use super::noise::smooth;
use super::sky::Sky;
use super::Region;
use crate::sim::geo::V2;

/// What the weather is, in a word or two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Clear,
    BrokenCloud,
    Overcast,
    Drizzle,
    Rain,
    HeavyRain,
    Downpour,
    Mist,
    SeaMist,
    Fog,
    ThickFog,
    HillFog,
    Gale,
    Storm,
    Thunderstorm,
    SnowFlurries,
    Snow,
    HeavySnow,
    Blizzard,
}

impl Kind {
    pub const ALL: [Kind; 19] = [
        Kind::Clear,
        Kind::BrokenCloud,
        Kind::Overcast,
        Kind::Drizzle,
        Kind::Rain,
        Kind::HeavyRain,
        Kind::Downpour,
        Kind::Mist,
        Kind::SeaMist,
        Kind::Fog,
        Kind::ThickFog,
        Kind::HillFog,
        Kind::Gale,
        Kind::Storm,
        Kind::Thunderstorm,
        Kind::SnowFlurries,
        Kind::Snow,
        Kind::HeavySnow,
        Kind::Blizzard,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Clear => "clear",
            Kind::BrokenCloud => "broken cloud",
            Kind::Overcast => "overcast",
            Kind::Drizzle => "drizzle",
            Kind::Rain => "rain",
            Kind::HeavyRain => "heavy rain",
            Kind::Downpour => "downpour",
            Kind::Mist => "mist",
            Kind::SeaMist => "sea mist",
            Kind::Fog => "fog",
            Kind::ThickFog => "thick fog",
            Kind::HillFog => "hill fog",
            Kind::Gale => "gale",
            Kind::Storm => "storm",
            Kind::Thunderstorm => "thunderstorm",
            Kind::SnowFlurries => "snow flurries",
            Kind::Snow => "snow",
            Kind::HeavySnow => "heavy snow",
            Kind::Blizzard => "blizzard",
        }
    }

    pub fn is_storm(self) -> bool {
        matches!(self, Kind::Gale | Kind::Storm | Kind::Thunderstorm | Kind::Blizzard)
    }

    pub fn is_fog(self) -> bool {
        matches!(self, Kind::Mist | Kind::SeaMist | Kind::Fog | Kind::ThickFog | Kind::HillFog)
    }

    pub fn is_wet(self) -> bool {
        matches!(self, Kind::Drizzle | Kind::Rain | Kind::HeavyRain | Kind::Downpour)
    }

    pub fn is_snow(self) -> bool {
        matches!(self, Kind::SnowFlurries | Kind::Snow | Kind::HeavySnow | Kind::Blizzard)
    }
}

/// Wind speeds (m/s) that count as a gale and as a full storm.
pub const GALE: f32 = 17.0;
pub const STORM_WIND: f32 = 24.0;
/// Sight (m) below which fog is "thick".
pub const THICK_FOG: f32 = 100.0;

/// The weather at a spot at a moment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weather {
    /// The region the spot mostly belongs to.
    pub region: Region,
    pub kind: Kind,
    /// Share of the sky covered, 0..1.
    pub cloud: f32,
    /// Rain, 0..1: drizzle below 0.2, a downpour above 0.8.
    pub rain: f32,
    /// Snow, 0..1.
    pub snow: f32,
    /// Wind speed, metres a second.
    pub wind: f32,
    /// The way the wind blows towards (x east, y south); length 1, or 0 in a dead calm.
    pub wind_to: V2,
    /// How gusty, 0..1.
    pub gust: f32,
    /// How thick the fog is here, 0..1.
    pub fog: f32,
    /// How far the fog reaches above the ground here, metres (a few metres:
    /// ground fog you can see over; hundreds: a whiteout).
    pub fog_height: f32,
    /// How far you can see, metres.
    pub visibility: f32,
    /// Deg C.
    pub temperature: f32,
    /// How wet the ground is, 0..1; lingers for hours after rain.
    pub wetness: f32,
    /// How much of a storm is on, 0..1.
    pub storm: f32,
    /// Lightning flashes a minute.
    pub lightning: f32,
    /// The sea, 0 flat calm .. 1 heavy surf.
    pub sea: f32,
    /// Above this height (metres above the sea) what falls is snow.
    pub snowline: f32,
    /// Height of the cloud base, metres above the sea.
    pub cloud_base: f32,
    /// Snow lies on ground above this height (metres above the sea).
    pub snow_lies_above: f32,
    /// How much snow lies here, 0 none .. 1 a full cover. (Steep ground
    /// sheds it; that is for whoever draws it.)
    pub snow_cover: f32,
}

/// Bring a region's sky down to a spot `height` metres up, where the lowest
/// ground round about is at `floor`.
pub fn localise(sky: &Sky, region: Region, height: f32, floor: f32) -> Weather {
    let c = climate();
    let temperature = sky.temp0 - c.lapse * height.max(0.0);

    // Ground fog lies on the low ground and reaches `fog_depth` above it.
    let over = (floor + sky.fog_depth - height).max(0.0);
    let ground_fog = sky.fog * smooth(0.0, 20.0, over);
    // Hill fog: the ground is up in the cloud.
    let hill_fog = 0.8 * smooth(sky.cloud_base - 50.0, sky.cloud_base + 100.0, height) * smooth(0.55, 0.85, sky.cloud);
    let fog = 1.0 - (1.0 - ground_fog) * (1.0 - hill_fog);
    let fog_height = if hill_fog > 0.02 {
        400.0
    } else if ground_fog > 0.0 {
        over
    } else {
        0.0
    };

    let frozen = smooth(c.snow_temp + 1.0, c.snow_temp - 1.0, temperature);
    let rain = sky.precip * (1.0 - frozen);
    let snow = sky.precip * frozen;
    let snowline = ((sky.temp0 - c.snow_temp) / c.lapse).max(0.0);

    let s = &c.sight;
    let dim = 1.0 / s.clear + fog.powf(2.74) / s.fog + rain.powf(2.7) / s.rain + snow.powi(2) / s.snow;
    let visibility = 1.0 / dim;

    let speed = (sky.wind_x * sky.wind_x + sky.wind_y * sky.wind_y).sqrt();
    let wind_to = if speed > 0.05 { V2::new(sky.wind_x / speed, sky.wind_y / speed) } else { V2::new(0.0, 0.0) };

    let kind = if sky.storm > 0.45 && sky.lightning > 0.3 {
        Kind::Thunderstorm
    } else if snow > 0.03 && snow >= rain {
        if snow > 0.2 && sky.wind >= 15.0 {
            Kind::Blizzard
        } else if snow > 0.55 {
            Kind::HeavySnow
        } else if snow > 0.2 {
            Kind::Snow
        } else {
            Kind::SnowFlurries
        }
    } else if sky.wind >= STORM_WIND {
        Kind::Storm
    } else if sky.wind >= GALE {
        Kind::Gale
    } else if fog > 0.3 {
        if hill_fog > ground_fog {
            Kind::HillFog
        } else if visibility < THICK_FOG {
            Kind::ThickFog
        } else if region.maritime() {
            Kind::SeaMist
        } else if fog > 0.5 {
            Kind::Fog
        } else {
            Kind::Mist
        }
    } else if rain > 0.8 {
        Kind::Downpour
    } else if rain > 0.55 {
        Kind::HeavyRain
    } else if rain > 0.2 {
        Kind::Rain
    } else if rain > 0.03 {
        Kind::Drizzle
    } else if sky.cloud >= 0.7 {
        Kind::Overcast
    } else if sky.cloud >= 0.25 {
        Kind::BrokenCloud
    } else {
        Kind::Clear
    };

    Weather {
        region,
        kind,
        cloud: sky.cloud,
        rain,
        snow,
        wind: sky.wind,
        wind_to,
        gust: sky.gust,
        fog,
        fog_height,
        visibility,
        temperature,
        wetness: sky.wetness,
        storm: sky.storm,
        lightning: sky.lightning,
        sea: sky.sea,
        snowline,
        cloud_base: sky.cloud_base,
        snow_lies_above: sky.snow_lying,
        snow_cover: smooth(sky.snow_lying - 40.0, sky.snow_lying + 40.0, height),
    }
}

/// A word for a wind speed (m/s).
pub fn wind_word(speed: f32) -> &'static str {
    match speed {
        s if s < 1.0 => "dead calm",
        s if s < 3.5 => "light air",
        s if s < 8.0 => "a breeze",
        s if s < 11.0 => "a fresh breeze",
        s if s < GALE => "a strong wind",
        s if s < STORM_WIND => "a gale",
        _ => "a storm wind",
    }
}

/// The quarter a wind comes from, given the way it blows towards.
pub fn quarter(to: V2) -> &'static str {
    if to.len() < 0.5 {
        return "nowhere";
    }
    // Compass bearing it comes from: 0 = north, 90 = east.
    let from = (-to.x).atan2(to.y).to_degrees().rem_euclid(360.0);
    const NAMES: [&str; 8] = ["north", "north-east", "east", "south-east", "south", "south-west", "west", "north-west"];
    NAMES[((from + 22.5) / 45.0) as usize % 8]
}

/// A distance you can see, in plain words.
pub fn sight_words(metres: f32) -> String {
    if metres < 1000.0 {
        format!("{} m", ((metres / 10.0).round() * 10.0).max(10.0) as i32)
    } else if metres < 10_000.0 {
        format!("{:.1} km", metres / 1000.0)
    } else {
        format!("{:.0} km", metres / 1000.0)
    }
}

impl Weather {
    /// The weather in a word or two: "sea mist", "drizzle", "gale".
    pub fn label(&self) -> &'static str {
        self.kind.label()
    }

    /// A line for the interface.
    pub fn describe(&self) -> String {
        let sky = match self.kind {
            Kind::Clear => "A clear sky",
            Kind::BrokenCloud => "Broken cloud, some sun",
            Kind::Overcast => "A flat grey sky",
            Kind::Drizzle => "Fine drizzle under a grey sky",
            Kind::Rain => "Steady rain",
            Kind::HeavyRain => "Heavy rain",
            Kind::Downpour => "Rain coming down in sheets",
            Kind::Mist => "Mist lying on the low ground",
            Kind::SeaMist => "Sea mist drifting in off the water",
            Kind::Fog => "Fog",
            Kind::ThickFog => "Thick fog: shapes at a few paces, nothing beyond",
            Kind::HillFog => "Up in the cloud",
            Kind::Gale => "A gale blowing",
            Kind::Storm => "A full storm",
            Kind::Thunderstorm => "A thunderstorm, dark as dusk",
            Kind::SnowFlurries => "Flurries of snow",
            Kind::Snow => "Snow falling",
            Kind::HeavySnow => "Heavy snow",
            Kind::Blizzard => "A blizzard",
        };
        let wind = if self.wind < 1.0 {
            "dead calm".to_string()
        } else if matches!(self.kind, Kind::Gale | Kind::Storm) {
            format!("out of the {}", quarter(self.wind_to))
        } else {
            format!("{} from the {}", wind_word(self.wind), quarter(self.wind_to))
        };
        format!("{sky}; {wind}; {:.0}°C; you can see {}.", self.temperature, sight_words(self.visibility))
    }
}
