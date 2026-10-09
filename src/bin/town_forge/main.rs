//! Town Forge: grows the demo town one step at a time, each step shown to
//! Laz as a review pack and frozen once he approves it.
//!
//!   town_forge step 0 [--seed N] [--world N] [--only A] [--fast]
//!
//! Review packs go to `town_forge_out/step_<n>/`; approved steps are saved
//! to `assets/towns/demo/`. Only the tooling for the step at hand exists.

mod canvas;
mod concepts;
mod erode;
mod field;
mod geology;
mod land;
mod noise;
mod render;

use canvas::{rgb, Canvas};
use concepts::Concept;
use gahturiyu_sim::sim::geo::V2;
use render::{Cam, MapView};
use std::path::PathBuf;

struct Args {
    step: Option<u32>,
    seed: u64,
    world: u64,
    only: Option<char>,
    /// Skip the slow 3D views (maps only).
    fast: bool,
}

fn args() -> Args {
    let v: Vec<String> = std::env::args().skip(1).collect();
    let mut a = Args { step: None, seed: 41, world: 1, only: None, fast: false };
    let mut i = 0;
    while i < v.len() {
        match v[i].as_str() {
            "step" => {
                a.step = v.get(i + 1).and_then(|s| s.parse().ok());
                i += 1;
            }
            "--seed" => {
                a.seed = v.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(a.seed);
                i += 1;
            }
            "--world" => {
                a.world = v.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(a.world);
                i += 1;
            }
            "--only" => {
                a.only = v.get(i + 1).and_then(|s| s.chars().next());
                i += 1;
            }
            "--fast" => a.fast = true,
            _ => {}
        }
        i += 1;
    }
    a
}

fn main() {
    let a = args();
    match a.step {
        Some(0) => step0(&a),
        Some(n) => eprintln!("Step {n} isn't built yet: each step's tooling is built once the step before it is approved."),
        None => eprintln!("usage: town_forge step <n> [--seed N] [--world N] [--only A|B|C] [--fast]"),
    }
}

fn out_dir(step: u32) -> PathBuf {
    let d = PathBuf::from(format!("town_forge_out/step_{step}"));
    std::fs::create_dir_all(&d).expect("make the review folder");
    d
}

/// The three cameras every pack uses, placed round the site's centre.
fn cameras(centre: V2) -> [(&'static str, Cam); 3] {
    [
        ("sea", Cam { pos: concepts::at(-820.0, 300.0), z: 12.0, look: centre, pitch_deg: 4.0, fov_deg: 60.0, w: 1600, h: 760 }),
        ("close", Cam { pos: concepts::at(-560.0, 60.0), z: 6.0, look: centre, pitch_deg: 2.0, fov_deg: 55.0, w: 1600, h: 760 }),
        ("high", Cam { pos: concepts::at(-700.0, 520.0), z: 230.0, look: centre, pitch_deg: -16.0, fov_deg: 60.0, w: 1600, h: 900 }),
    ]
}

fn step0(a: &Args) {
    let dir = out_dir(0);
    let terrain = gahturiyu_sim::sim::terrain::Terrain::generate(a.world);
    let mut notes = String::new();
    notes += "# Step 0: site pick\n\n";
    notes += &format!(
        "Seed {} (world seed {}). Three concepts for the site where the northern range meets the sea, \
        about 650 m north of today's Mouawiala. The mountains behind are the world's own.\n\n",
        a.seed, a.world
    );
    notes += "The land is not drawn: each concept is a rock recipe (layers, their tilt, cracks, hard bodies) and a \
        starting surface of rolling hills running on under the sea, and the land is what the sea, rain and gravity \
        make of it over a run of weathering. Same seed, same land.\n\n";
    let mut overview = Canvas::new(1920, 1180, rgb(26, 28, 30));
    for (k, c) in concepts::all(a.seed).into_iter().enumerate() {
        if a.only.is_some_and(|o| o != c.key) {
            continue;
        }
        eprintln!("concept {}: weathering", c.key);
        let t0 = std::time::Instant::now();
        let land = land::make(&terrain, a.seed, &c.site, c.centre);
        eprintln!("  {:.0} s", t0.elapsed().as_secs_f32());
        let mut f = land::field(&terrain, a.seed, &land, c.centre);
        render::colour_ground(&mut f, a.seed);
        let face = land::face_colour(&land);
        let map = render::top_down(&f, &MapView { centre: c.centre, span: 900.0, px: 1000 }, &[], &[], &[]);
        map.save(&dir.join(format!("concept_{}_map.png", c.key))).expect("save");
        let mut views = Vec::new();
        if !a.fast {
            for (name, cam) in cameras(c.centre) {
                eprintln!("  view: {name}");
                let img = render::view(&f, &cam, &[], &face);
                img.save(&dir.join(format!("concept_{}_{name}.png", c.key))).expect("save");
                views.push((name, img));
            }
        }
        sheet(&c, &map, &views, &dir.join(format!("concept_{}.png", c.key)));
        let x = 10 + k * 637;
        overview.text(x as f32, 34.0, &format!("{}: {}", c.key, c.name), 24.0, rgb(240, 214, 140));
        for (n, l) in overview.wrap(c.hook, 14.0, 620.0).iter().enumerate() {
            overview.text(x as f32, 60.0 + n as f32 * 18.0, l, 14.0, rgb(226, 226, 220));
        }
        if let Some((_, high)) = views.iter().find(|(n, _)| *n == "high") {
            overview.paste_scaled(high, x, 150, 627);
        }
        overview.paste_scaled(&map, x, 150 + 353 + 10, 627);
        notes += &format!("## {}: {}\n\n**Hook:** {}\n\n", c.key, c.name, c.hook);
        for n in c.notes {
            notes += &format!("- {n}\n");
        }
        notes += &format!("- {}\n", checks(&land));
        notes += &format!("\nFiles: `concept_{0}.png` (sheet), `concept_{0}_map.png`, `concept_{0}_sea.png`, `concept_{0}_close.png`, `concept_{0}_high.png`.\n\n", c.key);
    }
    overview.save(&dir.join("step_0_overview.png")).expect("save");
    notes += "## Choose\n\nPick A, B or C, or a mix (say which parts), or ask for a re-roll (`--seed N`). \
        The choice and its hook are saved as `assets/towns/demo/step_0_site.ron`, and Step 1 builds that landform \
        with its checks.\n";
    std::fs::write(dir.join("notes.md"), notes).expect("save notes");
    eprintln!("wrote {}", dir.display());
}

/// A few numbers about the land, for the notes.
fn checks(land: &erode::Land) -> String {
    let mut highest_cliff = 0.0f32;
    let n = land.w * land.h;
    for k in 0..n {
        let (i, j) = (k % land.w, k / land.w);
        if i < 3 || j < 3 || i >= land.w - 3 || j >= land.h - 3 || land.sea[k] {
            continue;
        }
        let near_sea = [k - 3, k + 3, k - 3 * land.w, k + 3 * land.w].iter().any(|&q| land.sea[q]);
        if near_sea {
            highest_cliff = highest_cliff.max(land.z[k]);
        }
    }
    // Stacks and skerries: small blobs of land not joined to the rest.
    let mut seen = vec![false; n];
    let mut stacks = 0;
    for k in 0..n {
        if land.sea[k] || seen[k] {
            continue;
        }
        let mut todo = vec![k];
        seen[k] = true;
        let mut size = 0;
        while let Some(c) = todo.pop() {
            size += 1;
            if size > 400 {
                break;
            }
            let (i, j) = (c % land.w, c / land.w);
            for (di, dj) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let (ni, nj) = (i as i32 + di, j as i32 + dj);
                if ni < 0 || nj < 0 || ni >= land.w as i32 || nj >= land.h as i32 {
                    continue;
                }
                let q = nj as usize * land.w + ni as usize;
                if !land.sea[q] && !seen[q] {
                    seen[q] = true;
                    todo.push(q);
                }
            }
        }
        if (4..=400).contains(&size) {
            stacks += 1;
        }
    }
    format!("Highest sea cliff about {highest_cliff:.0} m; {stacks} stacks or skerries.")
}

/// One concept's sheet: title and hook, the three views, the map and notes.
fn sheet(c: &Concept, map: &Canvas, views: &[(&str, Canvas)], path: &std::path::Path) {
    let view_h: usize = views.iter().map(|(_, v)| v.h * 1600 / v.w + 8).sum();
    let mut s = Canvas::new(1600, 120 + view_h + 1010, rgb(26, 28, 30));
    s.text(20.0, 40.0, &format!("{}: {}", c.key, c.name), 30.0, rgb(240, 214, 140));
    for (n, l) in s.wrap(c.hook, 17.0, 1560.0).iter().enumerate() {
        s.text(20.0, 72.0 + n as f32 * 22.0, l, 17.0, rgb(230, 230, 224));
    }
    let mut y = 120;
    for (name, v) in views {
        s.paste_scaled(v, 0, y, 1600);
        let label = match *name {
            "sea" => "From the sea, 800 m out",
            "close" => "From the water, close in",
            _ => "High, from the south-west",
        };
        s.label(12.0, y as f32 + 24.0, label, 16.0, rgb(250, 250, 250), rgb(20, 20, 20));
        y += v.h * 1600 / v.w + 8;
    }
    s.paste_scaled(map, 0, y, 1000);
    let (x, mut ty) = (1030.0, y as f32 + 40.0);
    s.text(x, ty, "Map: 900 m across, north up", 17.0, rgb(240, 214, 140));
    ty += 26.0;
    s.text(x, ty, "Contours every 5 m, bold every 25 m", 14.0, rgb(210, 210, 204));
    ty += 40.0;
    for n in c.notes {
        for l in s.wrap(n, 15.0, 540.0) {
            s.text(x, ty, &l, 15.0, rgb(210, 210, 204));
            ty += 20.0;
        }
        ty += 10.0;
    }
    s.save(path).expect("save sheet");
}
