//! Where the window's time goes, for finding what makes it slow
//! (`GAHT_PROF=<file>`): named spans of code add their time to a bucket, and
//! every `REPORT_FRAMES` frames the buckets are written out as milliseconds
//! a frame (average and worst) with the frame time itself. Off, a span is a
//! check of one flag. Drawing only.
//!
//! `GAHT_STROKE=<seconds>` (with `GAHT_PROF`): the land editor opens and a
//! Raise stroke is held for that long, the brush circling the middle of the
//! screen, then the window closes; the same stroke every run, so a change
//! can be measured before and after.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const REPORT_FRAMES: u32 = 30;

static ON: AtomicBool = AtomicBool::new(false);
/// What's going on, for the report (`phase`).
static PHASE: Mutex<&'static str> = Mutex::new("play");

/// Name what's going on now (play, editor, stroke), for the report.
pub fn phase(p: &'static str) {
    if on() {
        *PHASE.lock().unwrap() = p;
    }
}

#[derive(Default)]
struct Buckets {
    /// Name → (this report's total, worst single frame, this frame's).
    spans: BTreeMap<&'static str, (Duration, Duration, Duration)>,
    frames: u32,
    frame_total: Duration,
    frame_worst: Duration,
    last: Option<Instant>,
    path: Option<std::path::PathBuf>,
    started: Option<Instant>,
}

static B: Mutex<Option<Buckets>> = Mutex::new(None);

/// Turn the profiler on if `GAHT_PROF` is set (once, at start).
pub fn init() {
    if let Ok(p) = std::env::var("GAHT_PROF") {
        let path = std::path::PathBuf::from(p);
        let _ = std::fs::write(&path, "");
        *B.lock().unwrap() = Some(Buckets { path: Some(path), started: Some(Instant::now()), ..Default::default() });
        ON.store(true, Ordering::Relaxed);
    }
}

pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Time a span of code: its time goes to `name`'s bucket when the guard drops.
pub fn span(name: &'static str) -> Option<Span> {
    on().then(|| Span { name, at: Instant::now() })
}

pub struct Span {
    name: &'static str,
    at: Instant,
}

impl Drop for Span {
    fn drop(&mut self) {
        let d = self.at.elapsed();
        if let Some(b) = B.lock().unwrap().as_mut() {
            let e = b.spans.entry(self.name).or_default();
            e.0 += d;
            e.2 += d;
        }
    }
}

/// Count one of something this frame (shown as a span of 1 ms each: the
/// report's "avg" is then how many a frame).
pub fn count(name: &'static str) {
    if on() {
        if let Some(b) = B.lock().unwrap().as_mut() {
            let e = b.spans.entry(name).or_default();
            e.0 += Duration::from_millis(1);
            e.2 += Duration::from_millis(1);
        }
    }
}

/// Once a frame (the last system): close the frame, and write a report
/// every `REPORT_FRAMES`.
pub fn frame() {
    if !on() {
        return;
    }
    let mut g = B.lock().unwrap();
    let Some(b) = g.as_mut() else { return };
    let now = Instant::now();
    if let Some(last) = b.last {
        let d = now - last;
        b.frame_total += d;
        b.frame_worst = b.frame_worst.max(d);
        b.frames += 1;
    }
    b.last = Some(now);
    for e in b.spans.values_mut() {
        e.1 = e.1.max(e.2);
        e.2 = Duration::ZERO;
    }
    if b.frames >= REPORT_FRAMES {
        let n = b.frames as f64;
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let mut out = format!(
            "t={:.1}s [{}] frames={} frame avg {:.2} ms worst {:.2} ms\n",
            b.started.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0),
            *PHASE.lock().unwrap(),
            b.frames,
            ms(b.frame_total) / n,
            ms(b.frame_worst)
        );
        for (name, (total, worst, _)) in &b.spans {
            out += &format!("  {name:<24} avg {:7.2} ms  worst {:7.2} ms\n", ms(*total) / n, ms(*worst));
        }
        if let Some(p) = &b.path {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(p) {
                let _ = f.write_all(out.as_bytes());
            }
        }
        b.frames = 0;
        b.frame_total = Duration::ZERO;
        b.frame_worst = Duration::ZERO;
        for e in b.spans.values_mut() {
            *e = (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }
    }
}

/// The scripted stroke (`GAHT_STROKE`): how long to hold it, seconds.
pub fn stroke_secs() -> Option<f32> {
    std::env::var("GAHT_STROKE").ok().and_then(|v| v.parse().ok())
}
