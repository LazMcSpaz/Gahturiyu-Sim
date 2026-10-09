//! The naming system's own little tool: prints the tongues so they can be read.
//!
//!     cargo run --release --bin lang -- glossary        (every root in all four tongues; this is docs/glossary.md)
//!     cargo run --release --bin lang -- sacred          (gods, elements, peoples, sacred words; this is docs/sacred.md)
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
        _ => eprintln!("usage: lang glossary | sacred | cognates | say <first speech form> ... | pronounce <tongue> <word> ... | clashes"),
    }
}
