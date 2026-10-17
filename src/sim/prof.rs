//! A section timer for finding what a step of the world spends its time on
//! (the performance pass, 17 Oct). Off unless `prof::enable(true)` (tests and
//! tools); when off a section costs one relaxed atomic load. Drawing-free and
//! changes nothing that happens.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

static ON: AtomicBool = AtomicBool::new(false);

thread_local! {
    static SPENT: RefCell<BTreeMap<&'static str, (f64, u64)>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn enable(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// Run `f`, adding its time to `name` when timing is on.
#[inline]
pub fn time<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
    if !ON.load(Ordering::Relaxed) {
        return f();
    }
    let t = Instant::now();
    let r = f();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    SPENT.with(|s| {
        let mut s = s.borrow_mut();
        let e = s.entry(name).or_insert((0.0, 0));
        e.0 += ms;
        e.1 += 1;
    });
    r
}

/// What each section has cost since the last call (milliseconds, calls),
/// biggest first; the tally starts again.
pub fn take() -> Vec<(&'static str, f64, u64)> {
    SPENT.with(|s| {
        let mut v: Vec<_> = std::mem::take(&mut *s.borrow_mut()).into_iter().map(|(k, (ms, n))| (k, ms, n)).collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v
    })
}
