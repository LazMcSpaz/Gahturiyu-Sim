//! The crowd filler (`sim/crowd.rs`, authored people stage 3): the same
//! rows always make the same people; each row makes its count in its
//! role; the people vary (names, peoples, households, traits, wealth,
//! details); the mix is followed; and a filled roster reads back from its
//! file unchanged. The town keys here are made up for the tests.

use gahturiyu_sim::sim::{
    crowd::{self, details, fill_crowd, founders_mix, usual_wealth},
    jobs::Job,
    race::{Race, ALL_RACES},
    roster::{CrowdRow, Mix, Roster, Wealth},
};

fn rows() -> Vec<CrowdRow> {
    vec![
        CrowdRow { role: Job::Farmer, count: 30, mix: None },
        CrowdRow { role: Job::Guard, count: 6, mix: None },
        CrowdRow { role: Job::Merchant, count: 4, mix: Some(Mix { roduro: 0.0, qotiro: 1.0, horaro: 0.0, tadoro: 0.0 }) },
    ]
}

#[test]
fn the_same_rows_make_the_same_people() {
    let a = fill_crowd("test_town", Race::Roduro, &rows());
    let b = fill_crowd("test_town", Race::Roduro, &rows());
    assert_eq!(a, b);
    // Another town's rows make different people, keyed by its name.
    let c = fill_crowd("other_town", Race::Roduro, &rows());
    assert_ne!(a.iter().map(|p| &p.name).collect::<Vec<_>>(), c.iter().map(|p| &p.name).collect::<Vec<_>>());
}

#[test]
fn each_row_makes_its_count_in_its_role_with_its_key() {
    let crowd = fill_crowd("test_town", Race::Roduro, &rows());
    assert_eq!(crowd.len(), 40);
    for (r, row) in rows().iter().enumerate() {
        let of_row: Vec<_> = crowd.iter().filter(|p| p.profession == row.role).collect();
        assert_eq!(of_row.len(), row.count as usize);
        for (n, p) in of_row.iter().enumerate() {
            assert_eq!(p.key, format!("test_town.{r}.{n}"));
        }
    }
}

#[test]
fn the_people_vary() {
    let crowd = fill_crowd("test_town", Race::Roduro, &rows());
    let names: std::collections::HashSet<&str> = crowd.iter().map(|p| p.name.as_str()).collect();
    assert!(names.len() >= 36, "{} distinct names of 40", names.len());
    let men = crowd.iter().filter(|p| p.sex == gahturiyu_sim::sim::roster::Sex::Man).count();
    assert!((8..=32).contains(&men), "{men} men of 40");
    let households: std::collections::HashSet<u32> = crowd.iter().map(|p| p.household).collect();
    assert!(households.len() >= 8 && households.len() < 40, "{} households", households.len());
    assert!(crowd.iter().all(|p| p.household >= 1));
    // Every one has one or two traits shifted off the people's average, inside 0..1.
    for p in &crowd {
        let t = p.traits;
        let set = [t.wanderlust, t.boldness, t.sociability, t.patience, t.honour, t.asks].iter().filter(|v| v.is_some()).count();
        assert!((1..=2).contains(&set), "{}: {set} traits set", p.key);
        for v in [t.wanderlust, t.boldness, t.sociability, t.patience, t.honour, t.asks].into_iter().flatten() {
            assert!((0.05..=0.95).contains(&v));
        }
        assert!(p.appearance < crowd::APPEARANCES);
        assert!(!p.detail.is_empty(), "{} has no detail", p.key);
    }
    // Most keep the role's usual wealth, some step away.
    let farmers: Vec<_> = crowd.iter().filter(|p| p.profession == Job::Farmer).collect();
    let usual = farmers.iter().filter(|p| p.wealth == usual_wealth(Job::Farmer)).count();
    assert!(usual >= 15 && usual < 30, "{usual} of 30 farmers at the usual wealth");
    assert_eq!(usual_wealth(Job::Merchant), Wealth::Rich);
    assert_eq!(usual_wealth(Job::Drifter), Wealth::Destitute);
}

#[test]
fn the_mix_is_followed() {
    // A row with a mix of its own: all Qotiro.
    let crowd = fill_crowd("test_town", Race::Roduro, &rows());
    assert!(crowd.iter().filter(|p| p.profession == Job::Merchant).all(|p| p.people == Race::Qotiro));
    // A row without one leans to the founders.
    let big = fill_crowd("test_town", Race::Horaro, &[CrowdRow { role: Job::Fisher, count: 200, mix: None }]);
    let horaro = big.iter().filter(|p| p.people == Race::Horaro).count();
    assert!(horaro > 70 && horaro < 130, "{horaro} Horaro of 200 (founders' share {})", founders_mix(Race::Horaro).horaro);
    for r in ALL_RACES {
        assert!(big.iter().any(|p| p.people == r), "no {} at all", r.name());
    }
    let m = founders_mix(Race::Qotiro);
    assert!((m.roduro + m.qotiro + m.horaro + m.tadoro - 1.0).abs() < 1e-5);
    assert!(m.qotiro > 2.0 * m.roduro);
}

#[test]
fn details_fit_their_people_and_trade() {
    assert!(details().len() >= 40);
    let crowd = fill_crowd("test_town", Race::Roduro, &rows());
    for p in &crowd {
        let d = details().iter().find(|d| d.text == p.detail).expect("a detail from the list");
        assert!(d.peoples.is_empty() || d.peoples.contains(&p.people), "{}: {} for a {}", p.key, d.text, p.people.name());
        assert!(d.jobs.is_empty() || d.jobs.contains(&p.profession), "{}: {} for a {}", p.key, d.text, p.profession.name());
    }
    // Tags are read.
    let parsed = crowd::parse_details("detail | people=horaro, job=guard | x\ndetail | | y\n# a note\n");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].peoples, vec![Race::Horaro]);
    assert_eq!(parsed[0].jobs, vec![Job::Guard]);
    assert!(parsed[1].peoples.is_empty() && parsed[1].jobs.is_empty());
}

#[test]
fn a_roster_fills_once_and_round_trips() {
    let mut r = Roster { town: "test_town".into(), crowd_rows: rows(), ..Default::default() };
    assert_eq!(r.fill_crowd(Race::Roduro, false), Ok(40));
    // A stored crowd isn't overwritten unasked (Laz may have edited it).
    assert!(r.fill_crowd(Race::Roduro, false).is_err());
    assert_eq!(r.fill_crowd(Race::Roduro, true), Ok(40));
    let back = Roster::from_ron(&r.to_ron()).unwrap();
    assert_eq!(back, r);
    assert_eq!(back.len(), 40);
}
