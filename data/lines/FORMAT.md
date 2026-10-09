# Dialogue pieces: format guide

Everything people say when you talk to them is put together from small
pieces of text in these files. Edit them freely; no code changes needed
(rebuild the game to pick changes up).

## Files

- `greetings.txt`, `farewells.txt` — said whatever the talk is about.
- One file per **topic** someone can have on their mind: `theft.txt`,
  `grudge.txt`, `money.txt`, `hunger.txt`, `safety.txt`, `work.txt`,
  `ring.txt`, `kindness.txt`, `job.txt`, `news.txt`, `ambition.txt`.

## A line

```
slot | conditions | text | flags
```

- **slot** — where the piece goes: `greeting`, `topic`, `feeling`, `hook`,
  `farewell`, or `bark` (a one-line remark as you walk past).
- **conditions** — comma-separated tags that must all be true. Empty means
  "always fits". `!tag` means the tag must be false. When several pieces
  fit, the one with the **most** conditions wins (the most specific); ties
  are broken by a roll that's fixed for that person; a piece this person
  said to this squad member recently is skipped if anything else fits.
- **text** — what's said. `{slot}` names are filled in (below). An empty
  text (as in `hook | |`) means "say nothing here".
- **flags** — optional. `refuse` ends the conversation after it.

Lines starting with `#` are notes.

A talk opens with: greeting + topic + feeling + hook. More topics come up
as you ask. The farewell is said on goodbye.

## Tags

Who's talking:
- `voice=roduro`, `voice=qotiro`, `voice=horaro`, `voice=tadoro` — their people's way of speaking.
- `job=<job>` (e.g. `job=merchant`), `low_honour`, `high_honour`.
- Temper: `hot` (quick to anger and bold), `patient`, `gentle` (patient and
  sociable), `cold` (patient and unsociable).
- Mood: `worried`, `hungry`, `afraid`, `content`.
- `has_coin` / `poor` — whether their household could pay.

Who they're talking to:
- `you=roduro` etc. — the squad member's people.
- `stranger` (no standing in this town), `known` (some standing).
- `distrusts` (won't talk to them), `warm` (likes them).
- `suspects_squad` — trouble started about when the squad came to town.

When and where: `morning`, `day`, `evening`, `night`, `rain` (no weather
yet, so never true).

The topic: `topic=theft` etc., plus the topic's own tags:
- theft: `victim`, `repeated`, `heard`, `thief_known`, `thief_unknown`,
  `blames_watch`, `blames_ring`, `wants_guard`, `wants_item`, `wants_find`,
  `revenge`, `wants_help`.
- grudge: `feud`, `revenge`, `wants_help`.
- money: `in_debt`, `wants_debt`.  safety: `camp`, `wants_camp`.
- work: `lost_post`.  ring: `extorted`.  job: `has_offer`, `unlawful`.
- news: `hidden`.

## Slots

`{name}` (speaker), `{town}`, `{item}`, `{when}` ("last night", "two days
ago"), `{workplace}`, `{place}`, `{thief}`, `{victim}`, `{actor}`,
`{deed}`, `{count}`, `{span}` ("week"), `{pay}`, `{days}`, `{reward}`,
`{target}` (the other person or household), `{relation}` ("my neighbour",
"my household"), `{debt}`, `{job}`, `{why}`, `{offer}` (the job, in a line).
A slot with nothing to fill it says "someone" (names) or nothing.
