//! The naming system's own little tool: prints the tongues so they can be read.
//!
//!     cargo run --release --bin lang -- glossary        (every root in all four tongues; this is docs/glossary.md)
//!     cargo run --release --bin lang -- sacred          (gods, elements, peoples, sacred words; this is docs/sacred.md)
//!     cargo run --release --bin lang -- things          (jobs, materials, items, buildings, creatures, weather; this is docs/things.md)
//!     cargo run --release --bin lang -- things-clashes  (things whose native names come out alike)
//!     cargo run --release --bin lang -- make <recipe> ..  (try a recipe from things.ron in every tongue)
//!     cargo run --release --bin lang -- names           (the hand-kept given names; this is docs/names.md)
//!     cargo run --release --bin lang -- people          (twenty sample people of each people; this is docs/people.md)
//!     cargo run --release --bin lang -- places          (thirty sample places; this is docs/places.md)
//!     cargo run --release --bin lang -- world [seed]    (the towns of a generated world, and a few of their people)
//!     cargo run --release --bin lang -- guess           (how often a name's tongue can be told from its sounds)
//!     cargo run --release --bin lang -- name-candidates (every name the naming roots could make, for choosing the lists)
//!     cargo run --release --bin lang -- cognates        (the sets shown in docs/naming.md)
//!     cargo run --release --bin lang -- pronounce <tongue> <word> ..   (how to say a word)
//!     cargo run --release --bin lang -- say <first> ..  (First Speech forms run through every tongue)
//!     cargo run --release --bin lang -- clashes         (roots that sound alike within a tongue)

use gahturiyu_sim::names::{self, Tongue};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("glossary") => print!("{}", names::glossary()),
        Some("sacred") => print!("{}", names::sacred::tables()),
        Some("things") => print!("{}", names::things::tables()),
        Some("things-clashes") => {
            for t in Tongue::SPOKEN {
                let c = names::things::clashes(t);
                println!("{}: {} alike{}", t.name(), c.len(), c.iter().map(|(w, v)| format!("\n  {w} = {}", v.join(" / "))).collect::<String>());
            }
            // Longest names, to see what runs on.
            for t in Tongue::SPOKEN {
                let mut long: Vec<(usize, String, &str)> = names::things()
                    .iter()
                    .filter(|x| x.belongs.is_none() || x.belongs == Some(t))
                    .map(|x| {
                        let n = x.in_tongue(t);
                        (names::syllables(&n, t), n, x.english.as_str())
                    })
                    .collect();
                long.sort_by(|a, b| b.0.cmp(&a.0));
                println!("{} longest: {}", t.name(), long.iter().take(12).map(|(n, w, e)| format!("{w} ({n}, {e})")).collect::<Vec<_>>().join("; "));
            }
        }
        Some("pronounce") => {
            // lang pronounce <tongue> <word> ...
            let t = match args.get(2).map(|s| s.to_lowercase()).as_deref() {
                Some("roduro") => Tongue::Roduro,
                Some("qotiro") => Tongue::Qotiro,
                Some("horaro") => Tongue::Horaro,
                Some("tadoro") => Tongue::Tadoro,
                _ => {
                    eprintln!("usage: lang pronounce roduro|qotiro|horaro|tadoro <word> ...");
                    return;
                }
            };
            for w in &args[3..] {
                println!("{w}: {}  (plain letters: {})", names::pronounce(w, t), names::ascii(w));
            }
        }
        Some("cognates") => {
            println!("| Meaning | First Speech | Roduro | Qotiro | Horaro | Ṭaḍoro |");
            println!("|---|---|---|---|---|---|");
            for id in names::COGNATE_SETS {
                let r = names::root(id).unwrap_or_else(|| panic!("no root `{id}`"));
                let w: Vec<String> = Tongue::SPOKEN.iter().map(|t| names::word(id, *t).unwrap()).collect();
                println!("| {} | {} | {} | {} | {} | {} |", r.gloss, r.first, w[0], w[1], w[2], w[3]);
            }
        }
        Some("names") => print!("{}", names::people::tables()),
        Some("people") => print!("{}", names::people::samples()),
        Some("places") => print!("{}", names::places::tables()),
        Some("name-candidates") => {
            // Every name each people's roots could make: name, gender, recipe, ending, beats.
            for t in Tongue::SPOKEN {
                for c in names::people::candidates(t) {
                    println!("{}\t{}\t{}\t{}\t{}\t{}", t.name(), c.name, c.gender.word(), c.made, c.end, names::syllables(&c.name, t));
                }
            }
        }
        Some("world") => {
            // lang world [seed]: the towns of a generated world and a few of their people.
            use gahturiyu_sim::sim::{names as game, worldgen};
            let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
            let w = worldgen::generate(seed);
            println!("| Town | Founders | The land there | Its own name | Roduro | Qotiro | Horaro | Ṭaḍoro | Means |");
            println!("|---|---|---|---|---|---|---|---|---|");
            for s in &w.settlements {
                let f = game::site_features(&w.terrain, s.pos, s.coastal);
                match game::town(&w, s.id) {
                    Some(p) => println!("| **{}** | {} | {:?} | {} ({}) | {} | {} | {} | {} | {} |", s.name, s.founders.name(), f, p.name(), p.say, p.native[0], p.native[1], p.native[2], p.native[3], p.meaning),
                    None => println!("| **{}** | {} | {:?} | (not a name this system gave) | | | | | |", s.name, s.founders.name(), f),
                }
            }
            println!();
            for s in w.settlements.iter().take(3) {
                println!("{}:", s.name);
                for id in s.residents.iter().take(8) {
                    let p = game::who(&w, *id);
                    println!("  {}  [{}]  {}", p.full(), p.full_native(), p.story);
                }
            }
        }
        Some("guess") => {
            // lang guess: how often the tongue of a name can be told from its sounds.
            use gahturiyu_sim::names::{people, places, Gender};
            for t in Tongue::SPOKEN {
                let mut wrong: Vec<String> = Vec::new();
                let mut n = 0;
                for l in people::listed(t) {
                    n += 1;
                    if names::guess_tongue(&l.name) != t {
                        wrong.push(l.name.clone());
                    }
                }
                let listed = (n, wrong.len());
                for seed in 0..1500u64 {
                    let g = names::given_name(t, Gender::ALL[(seed % 3) as usize], seed);
                    n += 1;
                    if names::guess_tongue(&g.name) != t {
                        wrong.push(g.name.clone());
                    }
                }
                let mut pn = 0;
                let mut pw: Vec<String> = Vec::new();
                for (k, f) in places::FEATURES.iter().enumerate() {
                    for seed in 0..12u64 {
                        let p = names::generate_place(&[*f], t, seed * 31 + k as u64);
                        pn += 1;
                        if names::guess_tongue(p.name()) != t {
                            pw.push(p.name().to_string());
                        }
                    }
                }
                let missed = wrong.len();
                wrong.sort();
                wrong.dedup();
                println!("{}: lists {}/{} wrong; all people {:.1}% right; places {:.1}% right ({} of {})", t.name(), listed.1, listed.0, 100.0 - 100.0 * missed as f32 / n as f32, 100.0 - 100.0 * pw.len() as f32 / pn as f32, pw.len(), pn);
                println!("   people: {}", wrong.iter().take(40).cloned().collect::<Vec<_>>().join(" "));
                println!("   places: {}", pw.iter().take(20).cloned().collect::<Vec<_>>().join(" "));
            }
        }
        Some("make") => {
            // lang make <recipe> ... : try a recipe out in every tongue.
            for made in &args[2..] {
                let t = names::Thing { english: made.clone(), group: names::Group::World, belongs: None, made: made.clone(), also: vec![] };
                match t.check() {
                    Ok(()) => println!("{made}: {}", Tongue::SPOKEN.iter().map(|x| format!("{} {}", x.name(), t.in_tongue(*x))).collect::<Vec<_>>().join(" · ")),
                    Err(e) => println!("{made}: {e}"),
                }
            }
        }
        Some("say") => {
            for first in &args[2..] {
                let w: Vec<String> = Tongue::SPOKEN.iter().map(|t| format!("{} {}", t.name(), names::derive(first, *t))).collect();
                println!("{first}: {}", w.join(" · "));
            }
        }
        Some("clashes") => {
            for t in [Tongue::First, Tongue::Roduro, Tongue::Qotiro, Tongue::Horaro, Tongue::Tadoro] {
                let mut seen: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
                for r in names::roots() {
                    seen.entry(names::word(&r.id, t).unwrap_or_else(|| r.first.clone())).or_default().push(&r.id);
                }
                let alike: Vec<String> = seen.iter().filter(|(_, v)| v.len() > 1).map(|(w, v)| format!("{w} = {}", v.join(" / "))).collect();
                println!("{}: {} sound-alike groups{}{}", t.name(), alike.len(), if alike.is_empty() { "" } else { ": " }, alike.join("; "));
            }
        }
        _ => eprintln!("usage: lang glossary | sacred | things | things-clashes | make <recipe> ... | names | people | places | world [seed] | guess | name-candidates | cognates | say <first speech form> ... | pronounce <tongue> <word> ... | clashes"),
    }
}
