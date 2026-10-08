//! Saving and loading.
//!
//! A save is the world's seed plus everything that has changed since the
//! world was made from it. The land and the roads are rebuilt from the seed
//! on load (they never change and are most of the size); everything else —
//! people, groups and their schedules, fights in progress, the squad, what
//! lies on the ground, bounties, news — is written out as it is.
//!
//! Because the world's randomness is keyed by what is decided (rule 2), a
//! loaded world carries on exactly as the saved one would have.

use std::io::{Read, Write};
use std::sync::Mutex;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::body::Part;
use super::items::WeaponDef;
use super::routes::Routes;
use super::terrain::Terrain;
use super::world::World;

/// The first bytes of every save file.
const MAGIC: &[u8; 4] = b"GAHT";
/// Bumped whenever what's saved changes shape; older saves are refused
/// rather than misread.
pub const FORMAT: u32 = 10;

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    NotASave,
    /// Made by a different version of the game.
    OldFormat(u32),
    Corrupt(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "couldn't read the save: {e}"),
            LoadError::NotASave => write!(f, "that file isn't a save"),
            LoadError::OldFormat(v) => write!(f, "that save is from another version of the game (format {v}, this is {FORMAT})"),
            LoadError::Corrupt(e) => write!(f, "the save is damaged: {e}"),
        }
    }
}

impl World {
    /// The world as bytes.
    pub fn save_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT.to_le_bytes());
        bincode::serialize_into(&mut out, self).expect("the world always serialises");
        out
    }

    /// A world back from bytes made by `save_bytes`.
    pub fn load_bytes(bytes: &[u8]) -> Result<World, LoadError> {
        if bytes.len() < 8 || &bytes[..4] != MAGIC {
            return Err(LoadError::NotASave);
        }
        let v = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if v != FORMAT {
            return Err(LoadError::OldFormat(v));
        }
        let mut w: World = bincode::deserialize(&bytes[8..]).map_err(|e| LoadError::Corrupt(e.to_string()))?;
        let (terrain, routes) = super::worldgen::land(Terrain::generate(w.seed), &w.settlements);
        w.terrain = terrain;
        w.routes = routes;
        w.reindex();
        Ok(w)
    }

    pub fn save_to(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Write beside it first, so a crash mid-save never spoils the old one.
        let tmp = path.with_extension("tmp");
        std::fs::File::create(&tmp)?.write_all(&self.save_bytes())?;
        std::fs::rename(tmp, path)
    }

    pub fn load_from(path: &std::path::Path) -> Result<World, LoadError> {
        let mut bytes = Vec::new();
        std::fs::File::open(path).and_then(|mut f| f.read_to_end(&mut bytes)).map_err(LoadError::Io)?;
        World::load_bytes(&bytes)
    }
}

// --- Fixed names and tables ------------------------------------------------
//
// Some of what fighters carry refers to the game's fixed tables by name
// (a weapon's name, what armour covers). On load these come back as the same
// fixed values; each distinct one is kept once, however often you load.

/// A fixed name from the game's tables. (Written as an alias so the save
/// code reads it as an owned name, not one borrowed from the file.)
pub type Name = &'static str;

static NAMES: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
static COVERS: Mutex<Vec<&'static [Part]>> = Mutex::new(Vec::new());

fn keep_name(s: String) -> &'static str {
    let mut names = NAMES.lock().unwrap();
    if let Some(&n) = names.iter().find(|&&n| n == s) {
        return n;
    }
    let n: &'static str = Box::leak(s.into_boxed_str());
    names.push(n);
    n
}

fn keep_covers(v: Vec<Part>) -> &'static [Part] {
    let mut all = COVERS.lock().unwrap();
    if let Some(&c) = all.iter().find(|&&c| c == v.as_slice()) {
        return c;
    }
    let c: &'static [Part] = Box::leak(v.into_boxed_slice());
    all.push(c);
    c
}

pub(crate) mod name {
    use super::*;
    pub fn serialize<S: Serializer>(v: &&'static str, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(v)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<&'static str, D::Error> {
        String::deserialize(d).map(keep_name)
    }
}

pub(crate) mod opt_name {
    use super::*;
    pub fn serialize<S: Serializer>(v: &Option<&'static str>, s: S) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<&'static str>, D::Error> {
        Option::<String>::deserialize(d).map(|o| o.map(keep_name))
    }
}

pub(crate) mod covers {
    use super::*;
    pub fn serialize<S: Serializer>(v: &&'static [Part], s: S) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<&'static [Part], D::Error> {
        Vec::<Part>::deserialize(d).map(keep_covers)
    }
}

pub(crate) mod named_weapon {
    use super::*;
    pub fn serialize<S: Serializer>(v: &Option<(WeaponDef, &'static str)>, s: S) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<(WeaponDef, &'static str)>, D::Error> {
        Option::<(WeaponDef, String)>::deserialize(d).map(|o| o.map(|(w, n)| (w, keep_name(n))))
    }
}

/// Stand-ins while loading; replaced from the seed straight after.
pub(crate) fn no_terrain() -> Terrain {
    Terrain::empty()
}
pub(crate) fn no_routes() -> Routes {
    Routes::default()
}
