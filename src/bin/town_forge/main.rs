//! Town Forge: grows the demo town one step at a time, each step shown to
//! Laz as a review pack and frozen once he approves it.
//!
//!   town_forge step 0 [--seed N] [--world N]
//!
//! Review packs go to `town_forge_out/step_<n>/`; approved steps are saved
//! to `assets/towns/demo/`. Only the tooling for the step at hand exists.

mod canvas;
mod concepts;
mod field;
mod noise;
mod render;

use canvas::{rgb, Canvas};
use concepts::Concept;
use render::{MapView, Thing, WayKind};
use std::path::{Path, PathBuf};

struct Args {
    step: Option<u32>,
    seed: u64,
    world: u64,
}

fn args() -> Args {
    let v: Vec<String> = std::env::args().skip(1).collect();
    let mut a = Args { step: None, seed: 41, world: 1 };
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
        None => eprintln!("usage: town_forge step <n> [--seed N] [--world N]"),
    }
}

fn out_dir(step: u32) -> PathBuf {
    let d = PathBuf::from(format!("town_forge_out/step_{step}"));
    std::fs::create_dir_all(&d).expect("make the review folder");
    d
}

fn step0(a: &Args) {
    let dir = out_dir(0);
    let terrain = gahturiyu_sim::sim::terrain::Terrain::generate(a.world);
    let mut overview = Canvas::new(1920, 1110, rgb(26, 28, 30));
    let mut notes = String::new();
    notes += "# Step 0: site pick\n\n";
    notes += &format!("Seed {} (world seed {}). Three sketches of the site where the northern range meets the sea, \
        about 650 m north of today's Mouawiala. They're drawn over the real world land; the mountains behind are the world's own.\n\n", a.seed, a.world);
    notes += "Sketches only: the land is roughed in to show the idea, and Step 1 builds the chosen one properly (with its checks).\n\n";
    let builders: [fn(&gahturiyu_sim::sim::terrain::Terrain, u64) -> Concept; 3] = [concepts::ledge_town, concepts::glen_town, concepts::stack_town];
    for (k, build) in builders.iter().enumerate() {
        let c = build(&terrain, a.seed);
        eprintln!("concept {} built", c.key);
        let sea = render::sea_view(&c.field, &c.cam, &c.things);
        let map = render::top_down(&c.field, &MapView { centre: c.centre, span: 800.0, px: 1000 }, &c.things, &c.ways, &c.labels);
        let tag = c.key;
        sea.save(&dir.join(format!("concept_{tag}_sea.png"))).expect("save");
        map.save(&dir.join(format!("concept_{tag}_map.png"))).expect("save");
        sheet(&c, &sea, &map, &dir.join(format!("concept_{tag}.png")));
        // The side-by-side overview.
        let x = 10 + k * 637;
        let title = format!("{}: {}", c.key, c.name);
        overview.text(x as f32, 34.0, &title, 24.0, rgb(240, 214, 140));
        for (n, l) in overview.wrap(c.hook, 15.0, 620.0).iter().enumerate() {
            overview.text(x as f32, 60.0 + n as f32 * 19.0, l, 15.0, rgb(226, 226, 220));
        }
        overview.paste_scaled(&sea, x, 130, 627);
        overview.paste_scaled(&map, x, 130 + 316, 627);
        notes += &format!("## {}: {}\n\n**Hook:** {}\n\n", c.key, c.name, c.hook);
        for n in &c.notes {
            notes += &format!("- {n}\n");
        }
        notes += &format!("\nFiles: `concept_{tag}.png` (sheet), `concept_{tag}_sea.png`, `concept_{tag}_map.png`.\n\n");
    }
    overview.save(&dir.join("step_0_overview.png")).expect("save");
    notes += "## Choose\n\nPick A, B or C, or a mix (say which parts). The choice and its hook are saved as \
        `assets/towns/demo/step_0_site.ron`, and Step 1 builds that landform.\n";
    std::fs::write(dir.join("notes.md"), notes).expect("save notes");
    eprintln!("wrote {}", dir.display());
}

/// One concept's sheet: title and hook, the sea view, the map and a legend.
fn sheet(c: &Concept, sea: &Canvas, map: &Canvas, path: &Path) {
    let mut s = Canvas::new(1600, 110 + sea.h + 1000, rgb(26, 28, 30));
    s.text(20.0, 40.0, &format!("{}: {}", c.key, c.name), 30.0, rgb(240, 214, 140));
    for (n, l) in s.wrap(c.hook, 18.0, 1560.0).iter().enumerate() {
        s.text(20.0, 72.0 + n as f32 * 23.0, l, 18.0, rgb(230, 230, 224));
    }
    let top = 110;
    s.paste_scaled(sea, 0, top, 1600);
    s.text(12.0, top as f32 + 24.0, "From the sea", 16.0, rgb(250, 250, 250));
    s.paste_scaled(map, 0, top + sea.h, 1000);
    let (x, mut y) = (1030.0, (top + sea.h) as f32 + 40.0);
    s.text(x, y, "Map: 800 m across, north up", 17.0, rgb(240, 214, 140));
    y += 28.0;
    s.text(x, y, "Contours every 5 m, bold every 25 m", 14.0, rgb(210, 210, 204));
    y += 36.0;
    let legend: [(&str, Thing); 6] = [
        ("Roduro home (grown stone)", Thing::Home { at: Default::default(), r: 5.0, eldest: false }),
        ("Eldest Home", Thing::Home { at: Default::default(), r: 7.0, eldest: true }),
        ("Horaro stilt dome", Thing::Stilt { at: Default::default(), r: 4.0 }),
        ("Ṭaḍoro tent", Thing::Tent { at: Default::default() }),
        ("Qotiro stepped hall", Thing::Hall { at: Default::default(), size: 14.0, yaw: 0.0 }),
        ("Beacon", Thing::Beacon { at: Default::default() }),
    ];
    for (label, t) in legend {
        let (cx, cy) = (x + 14.0, y - 5.0);
        match t {
            Thing::Home { r, eldest, .. } => {
                s.disc(cx, cy, r * 1.25 + 1.2, rgb(48, 36, 26), 1.0);
                s.disc(cx, cy, r * 1.25, if eldest { [0.52, 0.44, 0.32] } else { render::STONE }, 1.0);
            }
            Thing::Stilt { .. } => {
                s.disc(cx, cy, 6.2, rgb(30, 30, 30), 1.0);
                s.disc(cx, cy, 5.0, [0.58, 0.44, 0.28], 1.0);
            }
            Thing::Tent { .. } => s.disc(cx, cy, 4.0, [0.66, 0.58, 0.66], 1.0),
            Thing::Hall { .. } => s.fill_rect(cx - 8.0, cy - 8.0, 16.0, 16.0, [0.72, 0.50, 0.30], 1.0),
            _ => s.disc(cx, cy, 5.0, rgb(240, 150, 50), 1.0),
        }
        s.text(x + 36.0, y, label, 15.0, rgb(226, 226, 220));
        y += 30.0;
    }
    for (label, kind) in [("Cobbled lane", WayKind::Lane), ("Stone stairs", WayKind::Stairs), ("Road in", WayKind::Road), ("Stream", WayKind::Stream)] {
        let col = kind.colour();
        if kind == WayKind::Stairs {
            s.dashed(&[(x + 2.0, y - 5.0), (x + 28.0, y - 5.0)], 4.0, 3.0, 2.0, col, 1.0);
        } else {
            s.line(x + 2.0, y - 5.0, x + 28.0, y - 5.0, 4.0, col, 1.0);
        }
        s.text(x + 36.0, y, label, 15.0, rgb(226, 226, 220));
        y += 30.0;
    }
    s.line(x + 2.0, y - 5.0, x + 28.0, y - 5.0, 5.0, [0.42, 0.31, 0.21], 1.0);
    s.text(x + 36.0, y, "Bridge", 15.0, rgb(226, 226, 220));
    y += 30.0;
    s.line(x + 2.0, y - 5.0, x + 28.0, y - 5.0, 5.0, rgb(150, 150, 146), 1.0);
    s.text(x + 36.0, y, "Quay / landing", 15.0, rgb(226, 226, 220));
    y += 50.0;
    for n in &c.notes {
        for l in s.wrap(n, 15.0, 540.0) {
            s.text(x, y, &l, 15.0, rgb(210, 210, 204));
            y += 20.0;
        }
        y += 10.0;
    }
    s.save(path).expect("save sheet");
}
