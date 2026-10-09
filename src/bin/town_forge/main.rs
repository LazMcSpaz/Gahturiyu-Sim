//! Town Forge: grows the demo town one step at a time, each step shown to
//! Laz as a review pack and frozen once he approves it.
//!
//!   town_forge step 0 [--seed N] [--world N] [--only A] [--fast]
//!
//! Review packs go to `town_forge_out/step_<n>/`; approved steps are saved
//! to `assets/towns/demo/`. Only the tooling for the step at hand exists.

mod bake;
mod canvas;
mod concepts;
mod erode;
mod found;
mod field;
mod geology;
mod land;
mod noise;
mod render;

use canvas::{rgb, Canvas};
use concepts::Concept;
use gahturiyu_sim::sim::geo::V2;
use render::{Cam, MapView, Thing, Way, WayKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Where approved steps live.
pub const TOWN_DIR: &str = "assets/towns/demo";

/// Step 0's record: the concept picked.
#[derive(Serialize, Deserialize, Debug)]
struct SiteChoice {
    concept: char,
    name: String,
    hook: String,
    seed: u64,
    world: u64,
}

/// Step 1's record: the landform and how it did on its checks.
#[derive(Serialize, Deserialize, Debug)]
struct LandRecord {
    concept: char,
    seed: u64,
    world: u64,
    /// The way in from the world's road (where the approach is judged from).
    approach_from: V2,
    checks: Vec<(String, bool, String)>,
    land_file: String,
}

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
        Some(1) => step1(&a),
        Some(2) => eprintln!("Step 2 (rock and ground detail) comes out of the weathering and is baked with Step 1."),
        Some(3) => step3(&a),
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

/// Where the world's road reaches the site: up the coast from the south,
/// a little inland.
fn approach_from() -> V2 {
    concepts::at(260.0, 640.0)
}

fn read_ron<T: for<'de> Deserialize<'de>>(name: &str) -> Option<T> {
    let text = std::fs::read_to_string(PathBuf::from(TOWN_DIR).join(name)).ok()?;
    ron::from_str(&text).ok()
}

fn write_ron<T: Serialize>(name: &str, v: &T) {
    std::fs::create_dir_all(TOWN_DIR).expect("make the town folder");
    let text = ron::ser::to_string_pretty(v, ron::ser::PrettyConfig::default()).expect("serialise");
    std::fs::write(PathBuf::from(TOWN_DIR).join(name), text).expect("write the step file");
}

/// The approved concept (Step 0's file, or `--only` to set it).
fn chosen(a: &Args) -> (Concept, SiteChoice) {
    let all = concepts::all(a.seed);
    let choice: Option<SiteChoice> = read_ron("step_0_site.ron");
    let key = a.only.or(choice.as_ref().map(|c| c.concept)).unwrap_or('A');
    let c = all.into_iter().find(|c| c.key == key).expect("a concept A, B or C");
    let record = SiteChoice { concept: c.key, name: c.name.to_string(), hook: c.hook.to_string(), seed: a.seed, world: a.world };
    (c, record)
}

/// Step 1: the chosen landform at full size, baked as the land editor's
/// layers for the game, with its checks.
fn step1(a: &Args) {
    let dir = out_dir(1);
    let terrain = gahturiyu_sim::sim::terrain::Terrain::generate(a.world);
    let (c, choice) = chosen(a);
    write_ron("step_0_site.ron", &choice);
    eprintln!("concept {}: weathering", c.key);
    let land = land::make(&terrain, a.seed, &c.site, c.centre);
    let edits = bake::bake(&land, &terrain, a.seed);
    std::fs::create_dir_all(TOWN_DIR).expect("make the town folder");
    edits.save_to(a.world, &PathBuf::from(TOWN_DIR).join("land.gmap")).expect("write the land");
    // Checks.
    let mut checks = Vec::new();
    let from = approach_from();
    let router = found::Router::new(&land, 2);
    let walk = router.route(from, c.centre);
    // The first view: walking in, where does the sea first show, and is
    // there a spot on the way that shows the mountains too?
    let f = land::field(&terrain, a.seed, &land, c.centre);
    let (mut reveal, mut both) = (None, None);
    if let Some((path, _)) = &walk {
        for i in (0..path.len()).step_by(4) {
            let look = path[(i + 6).min(path.len() - 1)];
            let (sea, mountains) = seen_from(&f, &land, path[i], look);
            if sea && reveal.is_none() {
                reveal = Some(path[i]);
            }
            if sea && mountains && both.is_none() {
                both = Some(path[i]);
            }
        }
    }
    checks.push(("Mountains and sea both in view on the approach".to_string(), both.is_some(), match (reveal, both) {
        (Some(r), Some(b)) => format!("the sea shows {:.0} m from the centre; both together {:.0} m out", r.dist(c.centre), b.dist(c.centre)),
        (Some(r), None) => format!("the sea shows {:.0} m from the centre, but never with the mountains", r.dist(c.centre)),
        _ => "the sea never shows on the way in".into(),
    }));
    let cliff = highest_cliff(&land);
    checks.push(("A cliff band over 25 m".to_string(), cliff > 25.0, format!("highest sea cliff {cliff:.0} m")));
    let pits = pits(&land);
    checks.push(("Water drains to the sea".to_string(), pits < 40, format!("{pits} hollows left holding water")));
    checks.push(("The approach can be walked (under 26°)".to_string(), walk.is_some(), match &walk {
        Some((_, g)) => format!("steepest grade {g:.0}°"),
        None => "no route under the limit".into(),
    }));
    let eye_at = both.or(reveal).unwrap_or(from);
    let eye_look = walk.as_ref().and_then(|(p, _)| {
        let i = p.iter().position(|q| q.dist(eye_at) < 0.1).unwrap_or(0);
        p.get((i + 6).min(p.len() - 1)).copied()
    }).unwrap_or(c.centre);
    let record = LandRecord { concept: c.key, seed: a.seed, world: a.world, approach_from: from, checks: checks.clone(), land_file: "land.gmap".into() };
    write_ron("step_1_land.ron", &record);
    // Pictures.
    let mut f = f;
    render::colour_ground(&mut f, a.seed);
    let face = land::face_colour(&land);
    let ways: Vec<Way> = walk.iter().map(|(p, _)| Way { kind: WayKind::Road, pts: p.clone() }).collect();
    let mut labels = vec![(from, "Approach".to_string()), (c.centre, "Centre".to_string())];
    if let Some(r) = reveal {
        labels.push((r, "First view".to_string()));
    }
    let map = render::top_down(&f, &MapView { centre: c.centre, span: 1000.0, px: 1000 }, &[], &ways, &labels);
    map.save(&dir.join("map.png")).expect("save");
    let mut views = Vec::new();
    if !a.fast {
        for (name, cam) in cameras(c.centre) {
            eprintln!("  view: {name}");
            let img = render::view(&f, &cam, &[], &face);
            img.save(&dir.join(format!("{name}.png"))).expect("save");
            views.push((name, img));
        }
        let eye = Cam { pos: eye_at, z: land.height(eye_at) + 1.7, look: eye_look, pitch_deg: -2.0, fov_deg: 90.0, w: 1600, h: 760 };
        let img = render::view(&f, &eye, &[], &face);
        img.save(&dir.join("approach.png")).expect("save");
        views.push(("approach", img));
    }
    sheet(&c, &map, &views, &dir.join("step_1.png"));
    let mut notes = format!("# Step 1: landform ({}: {})\n\n", c.key, c.name);
    notes += &format!("Seed {} (world {}). Baked to `{TOWN_DIR}/land.gmap`: the land editor's layers (height, ground paint, rocks) laid over the world's land, fading to it over the outer {:.0} m of a 1.4 km square.\n\n", a.seed, a.world, bake::RIM);
    notes += "Step 2's rock and ground detail (crags, scree, shingle, bare rock, wet rock at the tideline) comes out of the weathering and is in the same bake.\n\n## Checks\n\n";
    for (what, ok, how) in &checks {
        notes += &format!("- {} **{}**: {how}\n", what, if *ok { "pass" } else { "FAIL" });
    }
    notes += "\nFiles: `step_1.png` (sheet), `map.png`, `sea.png`, `close.png`, `high.png`, `approach.png` (eye height, where the road comes in).\n";
    std::fs::write(dir.join("notes.md"), notes).expect("save notes");
    eprintln!("wrote {}", dir.display());
}

/// From `from` at eye height looking toward `look`: is any sea in view, and any mountain?
fn seen_from(f: &field::Field, land: &erode::Land, from: V2, look: V2) -> (bool, bool) {
    let z0 = land.height(from) + 1.7;
    let (mut sea, mut mountain) = (false, false);
    let fwd = (look.y - from.y).atan2(look.x - from.x);
    for k in 0..40 {
        let a = fwd + (k as f32 - 19.5) * (100.0f32.to_radians() / 40.0);
        let (dx, dy) = (a.cos(), a.sin());
        let mut top = -10.0f32;
        let mut t = 3.0;
        while t < 9000.0 {
            let p = V2::new(from.x + dx * t, from.y + dy * t);
            if !f.inside(p.x, p.y) {
                break;
            }
            let h = f.height(p);
            let rise = (h.max(0.0) - z0) / t;
            if rise > top {
                top = rise;
                if h < 0.0 {
                    sea = true;
                }
                if h > 250.0 {
                    mountain = true;
                }
            }
            t += 2.0 + t * 0.01;
        }
    }
    (sea, mountain)
}

fn highest_cliff(land: &erode::Land) -> f32 {
    let mut best = 0.0f32;
    for k in 0..land.w * land.h {
        let (i, j) = (k % land.w, k / land.w);
        if i < 3 || j < 3 || i >= land.w - 3 || j >= land.h - 3 || land.sea[k] {
            continue;
        }
        if [k - 3, k + 3, k - 3 * land.w, k + 3 * land.w].iter().any(|&q| land.sea[q]) {
            best = best.max(land.z[k]);
        }
    }
    best
}

/// Hollows on land that water can't leave.
fn pits(land: &erode::Land) -> usize {
    let mut n = 0;
    for k in 0..land.w * land.h {
        let (i, j) = (k % land.w, k / land.w);
        if i < 1 || j < 1 || i >= land.w - 1 || j >= land.h - 1 || land.sea[k] {
            continue;
        }
        let z = land.z[k];
        // A real hollow, not a dimple: the lowest way out is 0.3 m up.
        if [k - 1, k + 1, k - land.w, k + land.w].iter().all(|&q| land.z[q] > z + 0.3) {
            n += 1;
        }
    }
    n
}

/// Step 3: the founders.
fn step3(a: &Args) {
    let dir = out_dir(3);
    let terrain = gahturiyu_sim::sim::terrain::Terrain::generate(a.world);
    let (c, _) = chosen(a);
    let record: LandRecord = read_ron("step_1_land.ron").expect("Step 1 must be approved first (assets/towns/demo/step_1_land.ron)");
    eprintln!("concept {}: weathering", c.key);
    let land = land::make(&terrain, record.seed, &c.site, c.centre);
    if std::env::var("FORGE_SHORE").is_ok() {
        let mut r = found::Router::new(&land, 2);
        r.max_deg = 42.0;
        let d = r.distances(record.approach_from);
        let mut shore = 0;
        let mut reach = 0;
        let mut low = 0;
        for k in 0..land.w * land.h {
            let (i, j) = (k % land.w, k / land.w);
            if i < 3 || j < 3 || i >= land.w - 3 || j >= land.h - 3 || land.sea[k] {
                continue;
            }
            if land.z[k] <= 3.0 && land.slope_deg(k) <= 14.0 {
                low += 1;
                if [k - 1, k + 1, k - land.w, k + land.w].iter().any(|&q| land.sea[q]) {
                    shore += 1;
                    if r.reachable(&d, land.pos(k)) {
                        reach += 1;
                    }
                }
            }
        }
        eprintln!("low flat cells {low}, of which shore {shore}, reachable {reach}; approach reachable from itself: {}", r.reachable(&d, record.approach_from));
        eprintln!("approach height {:.1}", land.height(record.approach_from));
    }
    let (founding, notes_found) = found::found(&land, record.seed, c.site.params.swell_from_deg, record.approach_from, c.centre, concepts::SITE_R - 60.0);
    write_ron("step_3_founding.ron", &founding);
    // Pictures: homes as marks on the land.
    let mut f = land::field(&terrain, a.seed, &land, c.centre);
    render::colour_ground(&mut f, a.seed);
    let face = land::face_colour(&land);
    let things: Vec<Thing> = founding.homes.iter().map(|h| {
        let fp = found::footprint(&h.model);
        Thing::Home { at: h.at, r: fp.wide * 0.5, eldest: h.eldest }
    }).collect();
    let mut ways: Vec<Way> = founding.paths.iter().map(|p| Way { kind: WayKind::Lane, pts: p.clone() }).collect();
    if !founding.approach.is_empty() {
        ways.push(Way { kind: WayKind::Road, pts: founding.approach.clone() });
    }
    let labels = vec![
        (founding.landing, "Landing".to_string()),
        (founding.spring, "Water".to_string()),
        (founding.eldest_site.add(V2::new(0.0, -14.0)), "Eldest Home".to_string()),
    ];
    let span = 500.0;
    let map = render::top_down(&f, &MapView { centre: founding.landing.lerp(founding.eldest_site, 0.5), span, px: 1000 }, &things, &ways, &labels);
    map.save(&dir.join("map.png")).expect("save");
    let mut views = Vec::new();
    if !a.fast {
        let mid = founding.landing.lerp(founding.eldest_site, 0.5);
        let cams = [
            ("sea", Cam { pos: concepts::at(-520.0, (mid.y - concepts::SITE_Y) + 120.0), z: 10.0, look: mid, pitch_deg: 3.0, fov_deg: 50.0, w: 1600, h: 760 }),
            ("high", Cam { pos: V2::new(mid.x - 260.0, mid.y + 200.0), z: land.height(mid).max(0.0) + 120.0, look: mid, pitch_deg: -22.0, fov_deg: 55.0, w: 1600, h: 900 }),
        ];
        for (name, cam) in cams {
            eprintln!("  view: {name}");
            let img = render::view(&f, &cam, &things, &face);
            img.save(&dir.join(format!("{name}.png"))).expect("save");
            views.push((name, img));
        }
    }
    sheet(&c, &map, &views, &dir.join("step_3.png"));
    let mut notes = format!("# Step 3: founding ({}: {})\n\n", c.key, c.name);
    for n in &notes_found {
        notes += &format!("- {n}\n");
    }
    notes += "\n## Homes\n\n";
    for (i, h) in founding.homes.iter().enumerate() {
        notes += &format!("{}. {}{} at {:.0} m up: {}.\n", i + 1, h.model, if h.eldest { " (the Eldest Home's site)" } else { "" }, land.height(h.at), h.why);
    }
    notes += &format!("\nSaved as `{TOWN_DIR}/step_3_founding.ron` (landing, water, the Eldest Home's site, homes with facing and model, footpaths, the approach). The game draws these homes on the land.\n");
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
            "approach" => "At eye height where the road comes in",
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
