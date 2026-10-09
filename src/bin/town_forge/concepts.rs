//! Step 0: three site concepts. Each is a rock recipe and a hook sentence;
//! the land itself is made by weathering (`land.rs`, `erode.rs`), never drawn.
//!
//! Positions use a coast frame: `u` = metres inland from the world's own
//! shoreline (negative = out to sea), `v` = metres south of the site's
//! middle line.

use crate::land::Site;
use gahturiyu_sim::sim::geo::{self, V2};

/// Where the demo town goes: the north coast, where the northern range
/// meets the sea (around today's Mouawiala).
pub const SITE_Y: f32 = 4050.0;
/// The site's half-length along the coast, and the fade beyond it.
pub const SITE_R: f32 = 400.0;
pub const MARGIN: f32 = 150.0;

pub struct Concept {
    pub key: char,
    pub name: &'static str,
    pub hook: &'static str,
    pub notes: &'static [&'static str],
    pub centre: V2,
    pub site: Site,
}

pub fn at(u: f32, v: f32) -> V2 {
    let y = SITE_Y + v;
    V2::new(geo::coast_x(y) + u, y)
}

pub fn all(seed: u64) -> Vec<Concept> {
    vec![
        Concept {
            key: 'A',
            name: "Ledge town",
            hook: "A long crack in the rock runs inland from the sea; the sea has cut a narrow inlet along it, and the layers dip so one wall benches into ledges. Homes would stack on the ledges, joined by stairs, with the harbour at the inlet's head.",
            notes: &[
                "Rock: layers about 9 m thick dipping 6° to the south; one fault zone 22 m wide running east-north-east from the shore.",
                "The stream from the hills is small; the drama is the inlet and its walls.",
            ],
            centre: at(0.0, 0.0),
            site: crate::land::ledge_site(seed),
        },
        Concept {
            key: 'B',
            name: "Glen-mouth town",
            hook: "A stream from the mountains has cut a glen down through the coastal hills to a bay; the town would climb both sides of the glen and bridge it high up.",
            notes: &[
                "Rock: near-flat layers about 7 m thick, so both glen walls bench; no big faults.",
                "A sizeable stream comes in from the north-east and does the cutting.",
            ],
            centre: at(0.0, -40.0),
            site: crate::land::glen_site(seed),
        },
        Concept {
            key: 'C',
            name: "Stack town",
            hook: "A dyke of hard rock runs out to sea as a headland; cracks across it let the sea cut it into stacks, with a softer cove in its lee. The oldest homes would stand on the stacks, joined by bridges, with the stilt village in the cove.",
            notes: &[
                "Rock: a hard band 70 m wide running west out to sea, crossed by three cracks; a soft fault zone makes the cove south of it.",
                "Which stacks survive depends on the seed: re-roll to get a different chain.",
            ],
            centre: at(-120.0, 20.0),
            site: crate::land::stack_site(seed),
        },
    ]
}
