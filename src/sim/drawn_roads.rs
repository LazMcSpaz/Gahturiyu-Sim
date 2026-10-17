//! Laz's roads, as he drew them (Laz, 17 Oct: "The only roads I want are the
//! ones that I had drawn"). The strokes lifted off his draft
//! (`docs/map/laz-roads-2026-10-13.png`, or `assets/world/roads.png` when that
//! layer exists) are thinned to centrelines, traced into lines and turned into
//! metres, once per process, the same in every world. These are the game's
//! only roads (`Routes::build`): nothing is drawn, painted or walked as a road
//! where he drew none. Travellers between towns with no road between them
//! walk the land.

use std::sync::OnceLock;

use super::atlas::{self, PX};
use super::geo::V2;

/// Where the road picture is looked for, first found wins (the same canvas as the map).
const SOURCES: [&str; 2] = ["assets/world/roads.png", "docs/map/laz-roads-2026-10-13.png"];
/// A pixel is road where the picture is at least this opaque.
const INK: u8 = 40;
/// Lines shorter than this many pixels are specks left from lifting the strokes.
const SPECK_PX: f32 = 4.0;
/// How far a simplified line may stray from the traced pixels, in pixels.
const SIMPLIFY_PX: f32 = 0.7;
/// A patch of ink smaller than this across, in pixels (~1.6 km), isn't a road:
/// the house icons and lettering lifted with the strokes (every such patch on
/// the 13 Oct draft is under 50 px; the road network is one patch across the
/// continent, the western island's road 113 px). A short spur joined to a road
/// is part of the road's patch and stays.
const MIN_SPAN_PX: usize = 60;
/// A dead-end stub shorter than this, in pixels (~210 m), is an icon that
/// touched a road, not a road.
const STUB_PX: f32 = 8.0;

/// The roads, each a line of points in metres. Empty when no road picture is found.
pub fn drawn_roads() -> &'static [Vec<V2>] {
    static ROADS: OnceLock<Vec<Vec<V2>>> = OnceLock::new();
    ROADS.get_or_init(|| match load() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("drawn roads: {e}");
            Vec::new()
        }
    })
}

fn load() -> Result<Vec<Vec<V2>>, String> {
    let path = SOURCES.iter().find_map(|rel| atlas::find_file(rel)).ok_or_else(|| format!("no road picture: looked for {}", SOURCES.join(", ")))?;
    let img = image::open(&path).map_err(|e| format!("{}: {e}", path.display()))?.to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut mask: Vec<bool> = img.pixels().map(|p| p.0[3] >= INK).collect();
    drop_specks(&mut mask, w, h);
    thin(&mut mask, w, h);
    let lines = prune_stubs(trace(&mask, w, h));
    Ok(lines
        .into_iter()
        .filter(|l| length(l) >= SPECK_PX)
        .map(|l| simplify(&l, SIMPLIFY_PX))
        .map(|l| l.into_iter().map(|(x, y)| V2::new((x + 0.5) * PX, (y + 0.5) * PX)).collect())
        .collect())
}

/// Drop short lines that end in nothing (an end no other line shares).
fn prune_stubs(lines: Vec<Vec<(f32, f32)>>) -> Vec<Vec<(f32, f32)>> {
    let key = |p: (f32, f32)| (p.0 as i32, p.1 as i32);
    let mut ends: std::collections::HashMap<(i32, i32), u32> = std::collections::HashMap::new();
    for l in &lines {
        *ends.entry(key(l[0])).or_default() += 1;
        *ends.entry(key(l[l.len() - 1])).or_default() += 1;
    }
    lines
        .into_iter()
        .filter(|l| {
            let dead = ends[&key(l[0])] == 1 || ends[&key(l[l.len() - 1])] == 1;
            let both = ends[&key(l[0])] == 1 && ends[&key(l[l.len() - 1])] == 1;
            // (A line with both ends free is a whole road on its own: kept.)
            both || !dead || length(l) >= STUB_PX
        })
        .collect()
}

fn length(l: &[(f32, f32)]) -> f32 {
    l.windows(2).map(|s| ((s[1].0 - s[0].0).powi(2) + (s[1].1 - s[0].1).powi(2)).sqrt()).sum()
}

/// Clear every 8-connected patch of ink less than `MIN_SPAN_PX` across.
fn drop_specks(m: &mut [bool], w: usize, h: usize) {
    let mut seen = vec![false; w * h];
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !m[start] || seen[start] {
            continue;
        }
        let mut patch = Vec::new();
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        seen[start] = true;
        stack.push(start);
        while let Some(k) = stack.pop() {
            patch.push(k);
            let (i, j) = (k % w, k / w);
            x0 = x0.min(i);
            x1 = x1.max(i);
            y0 = y0.min(j);
            y1 = y1.max(j);
            for dj in -1i64..=1 {
                for di in -1i64..=1 {
                    let (ii, jj) = (i as i64 + di, j as i64 + dj);
                    if ii >= 0 && jj >= 0 && (ii as usize) < w && (jj as usize) < h {
                        let n = jj as usize * w + ii as usize;
                        if m[n] && !seen[n] {
                            seen[n] = true;
                            stack.push(n);
                        }
                    }
                }
            }
        }
        if (x1 - x0).max(y1 - y0) + 1 < MIN_SPAN_PX {
            for k in patch {
                m[k] = false;
            }
        }
    }
}

/// Zhang-Suen thinning: strokes a few pixels wide become one-pixel centrelines.
fn thin(m: &mut [bool], w: usize, h: usize) {
    let at = |m: &[bool], i: i64, j: i64| i >= 0 && j >= 0 && (i as usize) < w && (j as usize) < h && m[j as usize * w + i as usize];
    let mut gone: Vec<usize> = Vec::new();
    loop {
        let mut changed = false;
        for pass in 0..2 {
            gone.clear();
            for j in 0..h as i64 {
                for i in 0..w as i64 {
                    if !m[j as usize * w + i as usize] {
                        continue;
                    }
                    // P2..P9 clockwise from north.
                    let p = [at(m, i, j - 1), at(m, i + 1, j - 1), at(m, i + 1, j), at(m, i + 1, j + 1), at(m, i, j + 1), at(m, i - 1, j + 1), at(m, i - 1, j), at(m, i - 1, j - 1)];
                    let b = p.iter().filter(|&&x| x).count();
                    if !(2..=6).contains(&b) {
                        continue;
                    }
                    let a = (0..8).filter(|&k| !p[k] && p[(k + 1) % 8]).count();
                    if a != 1 {
                        continue;
                    }
                    let (p2, p4, p6, p8) = (p[0], p[2], p[4], p[6]);
                    let ok = if pass == 0 { !(p2 && p4 && p6) && !(p4 && p6 && p8) } else { !(p2 && p4 && p8) && !(p2 && p6 && p8) };
                    if ok {
                        gone.push(j as usize * w + i as usize);
                    }
                }
            }
            for &k in &gone {
                m[k] = false;
            }
            changed |= !gone.is_empty();
        }
        if !changed {
            break;
        }
    }
}

/// The m-adjacent neighbours of a skeleton pixel: the four sides, and a corner
/// only when neither side between holds a pixel (so a staircase isn't a junction).
fn neighbours(m: &[bool], w: usize, h: usize, k: usize) -> Vec<usize> {
    let (i, j) = ((k % w) as i64, (k / w) as i64);
    let on = |i: i64, j: i64| i >= 0 && j >= 0 && (i as usize) < w && (j as usize) < h && m[j as usize * w + i as usize];
    let idx = |i: i64, j: i64| j as usize * w + i as usize;
    let mut out = Vec::new();
    for (di, dj) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        if on(i + di, j + dj) {
            out.push(idx(i + di, j + dj));
        }
    }
    for (di, dj) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        if on(i + di, j + dj) && !on(i + di, j) && !on(i, j + dj) {
            out.push(idx(i + di, j + dj));
        }
    }
    out
}

/// Trace the skeleton into lines: from every end and junction along each way
/// out until the next end or junction; then any closed loops.
fn trace(m: &[bool], w: usize, h: usize) -> Vec<Vec<(f32, f32)>> {
    let pts: Vec<usize> = (0..w * h).filter(|&k| m[k]).collect();
    let mut deg = vec![0u8; w * h];
    for &k in &pts {
        deg[k] = neighbours(m, w, h, k).len() as u8;
    }
    let xy = |k: usize| ((k % w) as f32, (k / w) as f32);
    let mut seen = vec![false; w * h]; // a line pixel (degree 2) already in a line
    let mut used: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new(); // a node's first step taken
    let mut lines = Vec::new();
    let is_node = |k: usize| deg[k] != 2;
    for &start in pts.iter().filter(|&&k| is_node(k)) {
        if deg[start] == 0 {
            continue;
        }
        for first in neighbours(m, w, h, start) {
            if used.contains(&(start, first)) || (!is_node(first) && seen[first]) {
                continue;
            }
            let mut line = vec![xy(start)];
            let (mut prev, mut cur) = (start, first);
            loop {
                line.push(xy(cur));
                if is_node(cur) {
                    used.insert((cur, prev));
                    break;
                }
                seen[cur] = true;
                let next = neighbours(m, w, h, cur).into_iter().find(|&n| n != prev && (is_node(n) || !seen[n]));
                match next {
                    Some(n) => {
                        prev = cur;
                        cur = n;
                    }
                    None => break,
                }
            }
            used.insert((start, first));
            lines.push(line);
        }
    }
    // Closed loops: no ends, no junctions.
    for &k in &pts {
        if deg[k] != 2 || seen[k] {
            continue;
        }
        let mut line = vec![xy(k)];
        seen[k] = true;
        let (mut prev, mut cur) = (k, neighbours(m, w, h, k)[0]);
        while !seen[cur] {
            seen[cur] = true;
            line.push(xy(cur));
            match neighbours(m, w, h, cur).into_iter().find(|&n| n != prev && !seen[n]) {
                Some(n) => {
                    prev = cur;
                    cur = n;
                }
                None => break,
            }
        }
        line.push(xy(k));
        lines.push(line);
    }
    lines
}

/// Douglas-Peucker: the fewest points that keep the line within `eps` pixels.
fn simplify(p: &[(f32, f32)], eps: f32) -> Vec<(f32, f32)> {
    if p.len() < 3 {
        return p.to_vec();
    }
    let (a, b) = (p[0], p[p.len() - 1]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    let dist = |q: (f32, f32)| if len < 1e-6 { ((q.0 - a.0).powi(2) + (q.1 - a.1).powi(2)).sqrt() } else { ((q.0 - a.0) * dy - (q.1 - a.1) * dx).abs() / len };
    let (mut far, mut at) = (0.0f32, 0usize);
    for (k, &q) in p.iter().enumerate().take(p.len() - 1).skip(1) {
        let d = dist(q);
        if d > far {
            far = d;
            at = k;
        }
    }
    if far <= eps {
        return vec![a, b];
    }
    let mut left = simplify(&p[..=at], eps);
    let right = simplify(&p[at..], eps);
    left.pop();
    left.extend(right);
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_thick_stroke_becomes_one_line() {
        let (w, h) = (40, 12);
        let mut m = vec![false; w * h];
        for j in 4..8 {
            for i in 3..37 {
                m[j * w + i] = true;
            }
        }
        thin(&mut m, w, h);
        let lines: Vec<_> = trace(&m, w, h).into_iter().filter(|l| length(l) >= SPECK_PX).collect();
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(length(&lines[0]) > 25.0);
        assert_eq!(simplify(&lines[0], SIMPLIFY_PX).len(), 2);
    }

    #[test]
    fn a_crossroads_is_four_lines_meeting() {
        let (w, h) = (31, 31);
        let mut m = vec![false; w * h];
        for k in 2..29 {
            m[15 * w + k] = true;
            m[k * w + 15] = true;
        }
        let lines: Vec<_> = trace(&m, w, h).into_iter().filter(|l| length(l) >= 1.0).collect();
        assert_eq!(lines.len(), 4, "{lines:?}");
        assert!(lines.iter().all(|l| l.contains(&(15.0, 15.0))));
    }
}
