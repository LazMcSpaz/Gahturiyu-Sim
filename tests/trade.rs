//! Honest trade (playtest 2 item 6, playtest 3 "one honest price"): the
//! number on the Sell all button is the coin received, the sell list shows
//! what fetches most first and hides nothing, and worn things are pointed
//! out instead of silently missing.

use gahturiyu_sim::sim::{
    dialogue::{Topic, SELL_PAGE, SELL_SHOWN, WORN_SHOWN},
    items::{self, ItemId, ITEMS},
    jobs::Job,
    person::PersonId,
    world::{DAY, HOUR},
    worldgen, World,
};

fn until_hour(w: &mut World, hour: f64) {
    let now = w.time.rem_euclid(DAY) / HOUR;
    let wait = (hour - now).rem_euclid(24.0);
    for _ in 0..(wait * 2.0).round() as usize {
        w.step(HOUR / 2.0);
    }
}

/// A world at 11 in the morning, and a merchant at their stall who buys `it`.
fn market(seed: u64, it: ItemId) -> (World, PersonId) {
    let mut w = worldgen::generate(seed);
    until_hour(&mut w, 11.0);
    let merchant = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Merchant && w.at_work(p, w.time) && w.offer(p, it, None).is_some()).expect("a merchant at work who takes it");
    (w, merchant)
}

/// Stand the squad's first member by the merchant and open their wares.
fn open_wares(w: &mut World, merchant: PersonId) -> PersonId {
    let me = w.squad.members[0];
    let at = w.person_pos(merchant);
    for k in 0..w.squad.members.len() {
        w.squad.at[k] = at;
        w.squad.goal[k] = at;
        w.squad.route[k].clear();
    }
    w.squad.pos = at;
    assert!(w.order_talk(me, merchant));
    for _ in 0..40 {
        if w.talk.is_some() {
            break;
        }
        w.step(0.25);
    }
    assert!(w.talk.is_some(), "the talk opened");
    w.ask(Topic::Trade);
    me
}

fn give(w: &mut World, who: PersonId, it: ItemId, n: u16) {
    w.people[who as usize].detail.as_mut().unwrap().gear.add(it, n);
}

#[test]
fn the_quote_for_selling_the_lot_is_the_coin_received() {
    let coin = items::id("coin");
    // A good in quantity (its price falls as they buy), a cheap good, and
    // something that isn't a good at all.
    for (key, n) in [("timber", 60), ("iron_ore", 50), ("dried_fish", 8), ("healing_draught", 5)] {
        let it = items::id(key);
        let (mut w, merchant) = market(3, it);
        let me = w.squad.members[0];
        let before = w.squad_count(it);
        give(&mut w, me, it, n);
        let (count, total) = w.sell_all_quote(merchant, it);
        assert!(count > 0, "{key}: they take some");
        let (had, purse) = (w.squad_count(coin), w.squad_count(it));
        let mut sold = 0;
        while w.sell(merchant, it) {
            sold += 1;
        }
        assert_eq!(sold, count, "{key}: as many as quoted");
        assert_eq!(w.squad_count(coin) - had, total, "{key}: for the coin quoted");
        assert_eq!(purse - w.squad_count(it), count);
        assert_eq!(purse, before + n);
    }
}

#[test]
fn a_big_sale_pays_less_each_and_the_button_says_what_it_comes_to() {
    let (timber, coin) = (items::id("timber"), items::id("coin"));
    let (mut w, merchant) = market(3, timber);
    let me = open_wares(&mut w, merchant);
    give(&mut w, me, timber, 60);
    let first = w.offer(merchant, timber, None).unwrap();
    let (count, total) = w.sell_all_quote(merchant, timber);
    // What the first one fetches is not what they all fetch: that was the
    // "(2 coin each)" that paid 54 for 50.
    assert!((total as u32) < first as u32 * count as u32 || count < w.squad_count(timber), "sixty don't fetch sixty times the first: {first} each, {total} for {count}");
    // Sixty timber is one of the bigger lots: it's on the first page.
    let topic = w.topics().into_iter().find(|t| *t == Topic::SellAll(timber)).expect("timber is on the first page");
    let label = w.topic_text(topic);
    assert!(label.ends_with(&format!("— {total} coin")), "{label}");
    assert!(!label.contains("each"), "{label}");
    if count < w.squad_count(timber) {
        assert!(label.starts_with(&format!("Sell {count} of ")), "{label}");
    } else {
        assert!(label.starts_with(&format!("Sell all {count} × ")), "{label}");
    }
    let had = w.squad_count(coin);
    w.ask(topic);
    assert_eq!(w.squad_count(coin) - had, total, "{label}");
}

#[test]
fn the_sell_list_is_what_fetches_most_first_and_nothing_is_cut_off() {
    let (mut w, merchant) = market(3, items::id("timber"));
    let me = open_wares(&mut w, merchant);
    // One each of plenty of things this merchant takes.
    let takes: Vec<ItemId> = (0..ITEMS.len() as ItemId).filter(|&it| w.squad_count(it) == 0 && w.offer(merchant, it, None).is_some()).take(SELL_SHOWN + SELL_PAGE + 4).collect();
    assert!(takes.len() > SELL_SHOWN, "this merchant takes {} kinds", takes.len());
    for &it in &takes {
        give(&mut w, me, it, 1);
    }
    // A cheap thing in bulk is worth more than one dear thing: it's what
    // the lot fetches that orders the list (the woodcutter's sixty timber
    // mustn't sink under a single torch).
    give(&mut w, me, items::id("timber"), 60);
    let kinds = w.sell_kinds(merchant);
    assert!(kinds.len() >= takes.len());
    assert!(kinds.windows(2).all(|p| p[0].2 >= p[1].2), "what fetches most first: {kinds:?}");
    for &(it, first, lot) in &kinds {
        assert!(first > 0);
        assert_eq!(lot, w.sell_all_quote(merchant, it).1);
    }
    let selling = |w: &World| w.topics().into_iter().filter(|t| matches!(t, Topic::Sell(..) | Topic::SellAll(_))).collect::<Vec<_>>();
    let shown = selling(&w);
    assert_eq!(shown.len(), SELL_SHOWN);
    // The dearest thing carried is the first on the list.
    let top = match shown[0] {
        Topic::Sell(it, _) | Topic::SellAll(it) => it,
        _ => unreachable!(),
    };
    assert_eq!(top, kinds[0].0);
    // The rest are a question away, a page at a time, and the question
    // says how many are left.
    let mut seen: Vec<Topic> = shown.clone();
    let mut pages = 0;
    while let Some(more) = w.topics().into_iter().find(|t| *t == Topic::SellRest) {
        assert!(w.topic_text(more).contains(&format!("({} more)", kinds.len() - seen.len())), "{}", w.topic_text(more));
        w.ask(more);
        let page = selling(&w);
        assert!(!page.is_empty() && page.len() <= SELL_PAGE);
        assert!(!w.topics().iter().any(|t| matches!(t, Topic::Buy(..) | Topic::SellWorn(..))), "a later page is the sell list alone");
        seen.extend(page);
        pages += 1;
        assert!(pages < 10);
    }
    assert!(pages >= 1);
    assert_eq!(seen.len(), kinds.len(), "every kind they'd take is on some page");
    for k in &kinds {
        assert!(seen.iter().any(|t| matches!(t, Topic::Sell(it, _) | Topic::SellAll(it) if *it == k.0)), "{k:?}");
    }
    // And back to their wares.
    assert!(w.topics().contains(&Topic::Trade));
    w.ask(Topic::Trade);
    assert_eq!(selling(&w), shown);
    assert!(w.topics().iter().any(|t| matches!(t, Topic::Buy(..))));
}

#[test]
fn worn_things_are_pointed_out_not_left_off() {
    let (mut w, merchant) = market(3, items::id("timber"));
    let me = open_wares(&mut w, merchant);
    let worn = w.worn_sellable(merchant);
    assert!(!worn.is_empty(), "the squad starts dressed, and some of it would sell");
    assert!(worn.windows(2).all(|p| p[0].2 >= p[1].2), "dearest first");
    let topics: Vec<Topic> = w.topics().into_iter().filter(|t| matches!(t, Topic::SellWorn(..))).collect();
    // One line per kind of thing, however many are wearing one.
    let mut kinds: Vec<ItemId> = worn.iter().map(|x| x.0).collect();
    kinds.dedup();
    let mut distinct: Vec<ItemId> = Vec::new();
    for k in kinds {
        if !distinct.contains(&k) {
            distinct.push(k);
        }
    }
    assert_eq!(topics.len(), distinct.len().min(WORN_SHOWN));
    let Topic::SellWorn(it, who, price) = topics[0] else { unreachable!() };
    assert_eq!((it, who, price), worn[0]);
    let label = w.topic_text(topics[0]);
    assert!(label.contains("worn: take it off to sell") && label.contains(&format!("~{price} coin")), "{label}");
    // Labels stay short enough for the window's topic column.
    for t in w.topics() {
        assert!(w.topic_text(t).chars().count() <= 60, "{}", w.topic_text(t));
    }
    // Asking changes nothing: it's still on, and no coin has moved.
    let coin = w.squad_count(items::id("coin"));
    let wearing: Vec<ItemId> = w.people[who as usize].detail.as_ref().unwrap().gear.equipped().collect();
    w.ask(topics[0]);
    assert_eq!(w.squad_count(items::id("coin")), coin);
    assert_eq!(w.people[who as usize].detail.as_ref().unwrap().gear.equipped().collect::<Vec<_>>(), wearing);
    let _ = me;
}

#[test]
fn what_something_sells_for_here_is_what_a_merchant_here_pays() {
    let draught = items::id("healing_draught");
    let (mut w, merchant) = market(3, draught);
    open_wares(&mut w, merchant);
    let (here, town) = w.sells_for(draught, None).expect("standing in a town");
    assert_eq!(Some(town), w.people[merchant as usize].home);
    assert_eq!(Some(here), w.offer(merchant, draught, None), "the same number the merchant gives");
    // Out in the wilds there's no price to quote.
    let far = gahturiyu_sim::sim::geo::V2::new(w.squad.pos.x + 40_000.0, w.squad.pos.y + 40_000.0);
    w.squad.pos = far;
    if w.settlements.iter().all(|s| s.pos.dist(far) > s.radius() + 60.0) {
        assert_eq!(w.sells_for(draught, None), None);
    }
    // NM-18: what a member's own things fetch goes by where that member
    // stands. One of them still at the stall is quoted the town's price
    // though the squad's middle is far out in the wilds.
    let me = w.squad.members[0];
    let at = w.person_pos(merchant);
    let k = w.squad.index(me).unwrap();
    w.squad.at[k] = at;
    w.squad.goal[k] = at;
    w.squad.pos = far;
    assert_eq!(w.sells_for_held(me, draught, None), Some((here, town)));
}

/// NM-24, N4 (Laz: B): a merchant won't buy what their own town knows was
/// stolen in it; a merchant elsewhere will.
#[test]
fn stolen_goods_have_no_buyer_in_the_town_they_were_taken_in() {
    use gahturiyu_sim::sim::inventory::Entry;
    let helm = items::id("iron_helm");
    let (mut w, merchant) = market(1, helm);
    let town = w.people[merchant as usize].home.unwrap();
    assert!(w.offer(merchant, helm, None).is_some(), "bought, before anything is known");
    w.mark_hot(town, &[Entry(helm, 1, None)], None);
    assert_eq!(w.offer(merchant, helm, None), None, "not once the town knows one was stolen");
    let me = w.squad.members[0];
    give(&mut w, me, helm, 1);
    assert!(!w.sellable(merchant).iter().any(|x| x.0 == helm), "and it isn't on the list of what they'd take");
    // A merchant of another town knows nothing of it.
    let elsewhere = w.settlements.iter().flat_map(|s| s.residents.iter().copied()).find(|&p| w.life(p).job == Job::Merchant && w.people[p as usize].home != Some(town) && w.at_work(p, w.time) && w.offer(p, helm, None).is_some());
    assert!(elsewhere.is_some(), "it still sells somewhere else");
}

/// RG-1 = RG-20 = NM-55: only the one talking and those standing with them
/// trade and pay; a squadmate a long way off keeps their pack and purse.
#[test]
fn only_those_standing_with_the_talker_trade_and_pay() {
    use gahturiyu_sim::sim::geo::V2;
    let (coin, timber) = (items::id("coin"), items::id("timber"));
    let (mut w, merchant) = market(1, timber);
    let me = open_wares(&mut w, merchant);
    for m in w.squad.members.clone() {
        let d = w.people[m as usize].detail.as_mut().unwrap();
        while d.gear.take(coin) {}
        while d.gear.take(timber) {}
    }
    let near = w.squad.members.iter().copied().find(|&m| m != me).unwrap();
    let far = w.squad.members.iter().copied().find(|&m| m != me && m != near).unwrap();
    give(&mut w, me, coin, 5);
    give(&mut w, near, coin, 20);
    give(&mut w, near, timber, 2);
    give(&mut w, far, coin, 500);
    give(&mut w, far, timber, 30);
    let k = w.squad.index(far).unwrap();
    w.squad.at[k] = w.squad.at[k].add(V2::new(300.0, 0.0));
    w.squad.goal[k] = w.squad.at[k];
    let has = |w: &World, m: PersonId, it: ItemId| w.people[m as usize].detail.as_ref().unwrap().gear.bag.iter().filter(|e| e.0 == it).map(|e| e.1).sum::<u16>();
    assert!(w.talk.is_some());
    assert_eq!(w.squad_count(coin), 25, "the purses at hand: the talker's and the one beside them");
    assert_eq!(w.sell_all_quote(merchant, timber).0, 2, "two timber to sell, not thirty-two");
    w.ask(Topic::SellAll(timber));
    assert_eq!(has(&w, far, timber), 30, "the far pack wasn't emptied");
    assert_eq!(has(&w, near, timber), 0, "the near one's was sold");
    assert_eq!(has(&w, far, coin), 500);
    // Nothing dearer than 25 coin can be bought, whatever the far purse holds.
    let dear = w.for_sale(merchant).into_iter().find(|x| x.2 > w.squad_count(coin));
    if let Some((it, _, price)) = dear {
        let before = w.squad_count(it);
        w.ask(Topic::Buy(it, price));
        assert_eq!(w.squad_count(it), before, "not bought with coin that isn't here");
        assert_eq!(has(&w, far, coin), 500);
    }
    // With the talk over, the squad's coin is counted whole again.
    w.end_talk();
    assert!(w.squad_count(coin) >= 525);
}

/// NM-59: all of a merchant's wares can be reached, a page at a time, and
/// food comes before everything else.
#[test]
fn all_of_a_merchants_wares_can_be_reached_food_first() {
    use gahturiyu_sim::sim::dialogue::BUY_SHOWN;
    use gahturiyu_sim::sim::jobs::good_of;
    let mut w = worldgen::generate(1);
    until_hour(&mut w, 11.0);
    let merchant = w
        .settlements
        .iter()
        .flat_map(|s| s.residents.iter().copied())
        .filter(|&p| w.life(p).job == Job::Merchant && w.at_work(p, w.time))
        .max_by_key(|&p| w.for_sale(p).len())
        .expect("a merchant at work");
    let wares = w.for_sale(merchant);
    assert!(wares.len() > BUY_SHOWN, "more wares than one page shows: {}", wares.len());
    let is_food = |it: ItemId| good_of(items::item(it).key).is_some_and(|g| g.is_food());
    let first_other = wares.iter().position(|x| !is_food(x.0)).unwrap_or(wares.len());
    assert!(wares[first_other..].iter().all(|x| !is_food(x.0)), "food comes first: {:?}", wares.iter().map(|x| items::item(x.0).name).collect::<Vec<_>>());
    open_wares(&mut w, merchant);
    let buys = |w: &World| -> Vec<(ItemId, u16)> { w.topics().iter().filter_map(|t| if let Topic::Buy(it, p) = t { Some((*it, *p)) } else { None }).collect() };
    assert_eq!(buys(&w).len(), BUY_SHOWN);
    // Page through: every ware turns up, once, in order.
    let mut seen = Vec::new();
    for _ in 0..12 {
        seen.extend(buys(&w));
        if !w.topics().contains(&Topic::BuyRest) {
            break;
        }
        w.ask(Topic::BuyRest);
    }
    assert_eq!(seen, wares.iter().map(|x| (x.0, x.2)).collect::<Vec<_>>());
    // Back to the first page.
    w.ask(Topic::Trade);
    assert_eq!(buys(&w).len(), BUY_SHOWN);
}
