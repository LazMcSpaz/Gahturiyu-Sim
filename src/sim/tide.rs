//! The tide clock and the season — both worked out from the time alone, so
//! anything that follows them (a stilt village's working day, a farm's
//! hours) is a lookup, never something ticked along.
//!
//! The tide is a plain wave: two highs a day, each a little later than the
//! day before, so the low water a diver works at slides through the day and
//! comes round again in about a fortnight. The season is another slow wave
//! that only moves working hours; daylight itself doesn't follow it yet.

use super::world::{DAY, HOUR};

/// Time from one high tide to the next (a half lunar day), seconds. Two
/// tides a day, each about 25 minutes later than the one before.
pub const TIDE_PERIOD: f64 = 12.42 * HOUR;
/// When the world's first high tide came in.
pub const TIDE_PHASE: f64 = 3.0 * HOUR;
/// Days in a year (a placeholder; the calendar isn't canon yet).
pub const YEAR_DAYS: f64 = 48.0;
/// How far seasonal working hours slide each way between midwinter and
/// midsummer, hours.
pub const SEASON_SWING: f32 = 1.0;

/// Water level at `t`: 1 at high tide, −1 at low.
pub fn level(t: f64) -> f32 {
    ((t - TIDE_PHASE) / TIDE_PERIOD * std::f64::consts::TAU).cos() as f32
}

/// The low tide closest to time `t`.
pub fn low_near(t: f64) -> f64 {
    let first_low = TIDE_PHASE + TIDE_PERIOD * 0.5;
    let k = ((t - first_low) / TIDE_PERIOD).round();
    first_low + k * TIDE_PERIOD
}

/// The next high tide after `t`.
pub fn next_high(t: f64) -> f64 {
    let k = ((t - TIDE_PHASE) / TIDE_PERIOD).floor() + 1.0;
    TIDE_PHASE + k * TIDE_PERIOD
}

/// The low tide nearest late morning on day `day`, as hours into that day.
/// Tide-keepers start work round this.
pub fn work_low(day: i64) -> f32 {
    let target = day as f64 * DAY + 10.5 * HOUR;
    ((low_near(target) - day as f64 * DAY) / HOUR) as f32
}

/// Where in the year we are: 1 at midsummer, −1 at midwinter.
pub fn season(t: f64) -> f32 {
    ((t / DAY / YEAR_DAYS) * std::f64::consts::TAU).sin() as f32
}

/// Hours seasonal workers start earlier (positive) or later (negative) on a day.
pub fn season_shift(day: i64) -> f32 {
    season((day as f64 + 0.5) * DAY) * SEASON_SWING
}

pub fn season_name(t: f64) -> &'static str {
    let s = season(t);
    let rising = season(t + DAY) > s;
    match (s > 0.5, s < -0.5, rising) {
        (true, _, _) => "midsummer",
        (_, true, _) => "midwinter",
        (_, _, true) => "spring",
        _ => "autumn",
    }
}

/// A word for the water.
pub fn tide_word(t: f64) -> &'static str {
    let l = level(t);
    let rising = level(t + 600.0) > l;
    if l > 0.7 {
        "high water"
    } else if l < -0.7 {
        "low water"
    } else if rising {
        "flooding"
    } else {
        "ebbing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_working_low_tide_slides_later_each_day_and_comes_round() {
        let a = work_low(1);
        let b = work_low(2);
        let slip = (b - a).rem_euclid(TIDE_PERIOD as f32 / HOUR as f32);
        assert!((slip - 0.84).abs() < 0.05, "a day later the low tide should be ~50 min later, was {slip}");
        // Always within half a tide of late morning.
        for d in 0..60 {
            let h = work_low(d);
            assert!((h - 10.5).abs() <= 6.22, "day {d}: {h}");
        }
        assert!((level(low_near(5.0 * DAY)) + 1.0).abs() < 1e-4);
    }
}
