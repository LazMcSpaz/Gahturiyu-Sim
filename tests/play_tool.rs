//! The text play tool, driven the way a play-tester drives it: one command a
//! call, the save loaded and written each time. These are the tool's own
//! faults from the bug hunt (odd input, wrong words), fixed.

use std::path::PathBuf;
use std::process::Command;

struct Game {
    save: PathBuf,
}

impl Game {
    fn new(name: &str, seed: u64) -> Game {
        let dir = std::env::temp_dir().join(format!("gaht-play-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let g = Game { save: dir.join("t.save") };
        let out = g.run(&["new", &seed.to_string()]);
        assert!(out.contains("A new game"), "{out}");
        g
    }

    /// Run one command; a crash fails the test with what was printed.
    fn run(&self, args: &[&str]) -> String {
        let out = Command::new(env!("CARGO_BIN_EXE_play")).arg(&self.save).args(args).output().expect("the play tool runs");
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(out.status.success() && !text.contains("panicked"), "`{}` crashed:\n{text}", args.join(" "));
        text
    }

    fn first_name(&self) -> String {
        let look = self.run(&["look"]);
        let line = look.lines().find(|l| l.contains("health")).expect("a squad line").trim_start_matches([' ', '*']);
        line.split_whitespace().next().unwrap().to_string()
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        if let Some(dir) = self.save.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// RG-5 = BL-1 = NM-9: a person number that doesn't exist is refused.
#[test]
fn a_person_who_does_not_exist_is_refused_by_every_command() {
    let g = Game::new("ids", 3);
    let name = g.first_name();
    for cmd in [vec!["talk", "p999999"], vec!["loot", "p999999"], vec!["attack", "p999999"], vec!["carry", "p999999"], vec!["go", "p999999"], vec!["talk", "p4294967295"], vec!["cast", name.as_str(), "1", "p999999"], vec!["use", name.as_str(), "0", "p999999"], vec!["dose", name.as_str(), "p999999"]] {
        let out = g.run(&cmd);
        assert!(!out.contains("A conversation opens"), "{out}");
    }
    assert!(g.run(&["talk", "p999999"]).contains("Can't talk to them."));
}

/// NM-70: an id of the wrong kind is refused, not read as another thing's number.
#[test]
fn the_letter_of_an_id_counts() {
    let g = Game::new("letters", 3);
    assert!(g.run(&["pickup", "x6"]).contains("takes a g id"));
    assert!(g.run(&["pickup", "p6"]).contains("takes a g id"));
    assert!(g.run(&["gather", "p59"]).contains("takes a n id"));
    assert!(g.run(&["work", "h2"]).contains("takes a d id"));
    assert!(g.run(&["butcher", "g1"]).contains("takes a c id"));
    assert!(g.run(&["pickup", "17"]).contains("takes a g id"));
    assert!(g.run(&["enter", "0.2"]).contains("No such building."));
    assert!(g.run(&["hunt", "p3"]).contains("Can't hunt that."));
    assert!(g.run(&["search", "0.2.0"]).contains("Can't go through that"));
    // Nobody has gone anywhere.
    assert!(g.run(&["look"]).contains("Day 1, 06:0"));
}

/// NM-75 = BL-66: `wait` wants whole minutes, a week at most.
#[test]
fn wait_takes_minutes_and_nothing_else() {
    let g = Game::new("wait", 3);
    for bad in ["abc", "", "0", "-5", "1.5", "nan", "inf"] {
        let out = if bad.is_empty() { g.run(&["wait"]) } else { g.run(&["wait", bad]) };
        assert!(out.contains("Usage: wait M"), "wait {bad}: {out}");
    }
    assert!(g.run(&["wait", "99999999"]).contains("a week"));
    assert!(g.run(&["look"]).contains("Day 1, 06:00"), "no time passed");
    assert!(g.run(&["wait", "5"]).contains("(5 minutes pass.)"));
}

/// NM-17 = BL-65: the word beside the clock follows the light.
#[test]
fn the_clock_s_word_follows_the_light() {
    let g = Game::new("light", 3);
    assert!(g.run(&["look"]).contains("06:00 (dawn)"));
    assert!(g.run(&["wait", "60"]).contains("07:00 (day)"));
    let dusk = g.run(&["wait", "690"]);
    assert!(dusk.contains("18:30 (day)"), "light until about seven: {}", dusk.lines().find(|l| l.starts_with("==")).unwrap_or(""));
    let later = g.run(&["wait", "90"]);
    assert!(later.contains("20:00 (dusk)"), "{}", later.lines().find(|l| l.starts_with("==")).unwrap_or(""));
    let night = g.run(&["wait", "60"]);
    assert!(night.contains("21:00 (night)"), "{}", night.lines().find(|l| l.starts_with("==")).unwrap_or(""));
}

/// BL-64: `putdown` and `torch` say what happened.
#[test]
fn putdown_and_torch_say_what_happened() {
    let g = Game::new("torch", 3);
    assert!(g.run(&["putdown"]).contains("Nobody selected is carrying anyone."));
    let lit = g.run(&["torch"]);
    assert!(lit.contains("lit.") || lit.contains("Nobody selected has a torch"), "{lit}");
    if lit.contains("lit.") {
        assert!(g.run(&["torch"]).contains("put out."));
    }
    // BL-61: no "-0" in the load.
    assert!(!g.run(&["look"]).contains("load -0"));
}
