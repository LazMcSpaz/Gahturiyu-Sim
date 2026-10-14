# Writing dialogue that doesn't sound like a machine

How the two Claude sessions write lines for Gahturiyu: how we research,
how we come to understand a speaker, how we write, and how we check.
Laz's ask (13 Oct 2026): "figure out how to write better dialogue; a process
that reflects combinations of traits in a way that feels real; take on
personas of great writers if you have to." This is a working document: either
session edits it, and Laz corrects it.

The lines live in `data/lines/` (format in `data/lines/FORMAT.md`), chosen by
conditions over facts the sim gathers (`talk.rs`). The engine decides *who*
speaks, *when*, and *what they know*; these pages are about *how they say it*.

## 1. What goes wrong (the audit)

Read our own lines aloud and the failures are the same every time:

- **Explaining.** The line tells the player how the system works ("I'm a
  merchant, so I buy and sell goods"). A person never explains their own
  life; they assume you know it.
- **Labelling.** "I am honourable." Nobody says what they are. Honour is a
  refusal, patience is a silence, boldness is an interruption.
- **Symmetry.** Every line is one tidy sentence of the same length with a
  subject, a verb and a full stop. Real speech is lopsided: a long one, then
  two words.
- **No want.** The speaker wants nothing from the listener, so the line goes
  nowhere. Every line is an act: to get, to keep, to test, to push away.
- **Anyone could say it.** Swap the speaker for another and the line still
  fits. Then it isn't theirs.
- **Generic nouns.** "Goods", "things", "the town", "trouble". Their world is
  made of nets, lime, the north gate, the tax man's dog.
- **The blanket voice.** One faction's greeting outranks everything, so a
  whole town says the same thing (`talk.rs`, `FACTION_WEIGHT`). A faction is
  an attitude, not a script.

Before writing anything new, each of us rewrites twenty existing lines
against this list (section 6) and the other reviews them. That is the
training set.

## 2. Research: who we read, and what we take

We can't play Kenshi or read a novel between turns; we work from what we
know and write the principle down with an example from our own lines, so the
knowledge becomes checkable. The sources, and the one thing each gives us:

| Source | The one thing | How it shows in a line |
|---|---|---|
| Hemingway (the iceberg) | Leave out what both speakers know. The weight is in the omission. | A wife's death is never named; the line is about the boat he hasn't sold. |
| David Mamet (*Three Uses of the Knife*; the plays) | People don't say what they mean; they say what gets them what they want. Interruptions, repetitions, deflections. | "You'll pay me." "I said I'd —" "You'll pay me." |
| Ursula K. Le Guin (*Steering the Craft*; Earthsea) | Rhythm is meaning. Short declaratives for weight; the taboo word left unsaid. Names matter. | Qotiro priestesses never say a god's name in anger; the gap says it. |
| Terry Pratchett | Humour comes from a character seeing clearly, never from a joke dropped on them. The watchman's tired exactness. | A guard counting what you'd owe before he's decided to arrest you. |
| Cormac McCarthy (*Blood Meridian*, *The Road*) | Landscape in the mouth. No quotation marks, no explaining; the land is the idiom. | A Roduro speaks in stone and weight; a Horaro in tides and lines. |
| Robert McKee (*Dialogue*) | Subtext: the said, the unsaid, the unsayable. Three layers in every exchange. | "Fine day for it." (said) — "You're not welcome." (unsaid) — "I'm afraid of you." (unsayable) |
| Lajos Egri (*The Art of Dramatic Writing*) | Character is three-dimensional: body, social station, psychology; dialogue comes from the collision of the three. | A rich man with a poor man's fear of hunger talks about bread at a feast. |
| Kenshi (game) | Terse, lived-in, unimpressed. The world is hard and nobody dramatises it. | "Another one. Try not to die in my street." |
| Disco Elysium (game) | Voices are specific to the point of obsession; the mundane is loaded. | A Ṭaḍoro teacher corrects your grammar while being robbed. |
| Pentiment, Sunless Sea (games) | Period voice without pastiche: idiom from the speaker's trade and faith, not "thee" and "thou". | Oaths by the local god; measures in the local units. |

We add to this table when one of us finds a principle that fixed a line.

## 3. Understand the speaker before writing a word

The sim hands the writer facts (`talk.rs` `facts`): people, sex, trade,
wealth, faction, traits (wanderlust, boldness, sociability, patience; more
with the rosters: honour, temper), mood and needs, what's on their mind
(`memory.rs`), their relation to the listener (stranger, trusted, robbed by,
one of ours), the hour, the weather, and for authored people their
background, importance and personal detail. Facts aren't a character. Before
a line, the writer answers five questions on a **character card** (in the
head, or in a comment at the top of the lines file):

1. **What do they want from this listener, right now?** (Coin, to be left
   alone, to be admired, to find out who you are, to get you to leave before
   the watch comes.)
2. **What are they afraid of?** (Hunger, the lord, being laughed at, the
   green coming closer.) Fear picks the subject they circle.
3. **Where do they stand, relative to you?** Above, below, or level — and do
   they think the same? Status decides who asks the questions and who
   answers. A bold poor man talks up; a patient rich one lets you talk.
4. **What would they never say?** Each people and each trade has its unsaid.
   A Horaro doesn't boast of a catch before it's landed. A Warden doesn't
   name what's in the Overgrowth. An official doesn't admit the till is empty.
5. **How do they sound?** Sentence length, favourite measure, oath, what they
   count in. (The voice bible below.) Their trade gives the nouns; their
   people gives the music; their traits give the delivery.

If a question has no answer, the line will be generic. Answer it or don't
write the line.

## 4. The voice bible (a start; Laz's lore outranks it)

A people's voice is a tendency, never a script (CLAUDE.md rule 15). It's the
music under the words; the trade and the traits supply the words.

- **Roduro** — stone, weight, patience of walls. Long sentences that end
  heavily. Measures in days and loads. Oaths by what stands. They don't
  hurry a sentence and they don't repeat one. Understatement.
- **Qotiro** — steps, rank, sun. Formal address, titles, the right word in
  the right order; a sentence is a small ceremony. Offence taken quietly and
  kept. Precision as pride; a Qotiro who swears has lost.
- **Horaro** — tide, line, weather. Short, present-tense, watching the sky
  while talking. Deflecting questions with questions. Luck is never named
  before it's in the boat. Jokes at their own expense, never at the sea's.
- **Ṭaḍoro** — road, wind, the teacher who taught them. Questions, lists,
  corrections; a lodger's courtesy with a traveller's detachment. They
  remember what you said last time and quote it back.

Traits are *delivery*, applied on top:

| Trait high | Low |
|---|---|
| **Boldness**: interrupts, states, closes distance, names the price first. | Hedges, asks permission to ask, talks around the thing, lets you name it. |
| **Patience**: lets silence sit; finishes your sentence a beat late; repeats nothing. | Repeats, hurries, stacks three things in one breath, ends with "well?" |
| **Sociability**: asks about you, uses your name, offers a story for a story. | Answers only what was asked; one clause; looks for the door. |
| **Wanderlust**: talks of elsewhere, compares here to there, restless nouns. | Talks of here, the house, the plot, what was always so. |
| **Honour**: keeps a bargain in the words ("I said"); refuses plainly. | Agrees too fast; "we'll see"; the price moves. |
| **Temper**: heat comes early and in the verbs. | Heat never comes; the cold word instead. |

**Combinations are where it gets real.** Two traits don't average; one leads
and the other *leaks*. The writer picks the leading trait (the highest or
the one the concern touches) and lets the second show as a crack:

- Bold *and* poor: states the price first, then undercuts it himself.
- Patient *and* afraid: the silence is too long; he fills it with weather.
- Sociable *and* dishonest: your name in every sentence, the number in none.
- Honourable *and* hot-tempered: refuses, apologises for the refusal's heat, refuses again.
- Low boldness *and* high wanderlust: talks of leaving, never of going.

The rule: **the second trait changes the delivery, not the content.** The
content comes from the want and the concern.

## 5. Emulate: writing under a lens

A line is written three times under three lenses, then one is chosen or
blended. The lenses are the writers above worn as masks, picked by the
scene:

- **The Mamet lens** for any exchange with stakes (a deal, a threat, a lie):
  nobody says the thing; the thing is in the repetition and the cut-off.
- **The Pratchett lens** for the watch, officials, anyone with a job and a
  tired exactness: the humour is the character seeing clearly.
- **The Le Guin lens** for grief, faith, the Overgrowth, anything with a
  taboo in it: short, weighted, the word left out.
- **The McCarthy / Kenshi lens** for the road, strangers, bandits, the wild:
  the land in the mouth; nothing dramatised.
- **The Egri lens** when a trait combination is the point: let the three
  dimensions collide in one sentence.

Writing under a lens means asking "how would this writer *refuse* to say
it?" more than "how would they say it". Then we take the mask off and check
it is still the speaker's voice (section 4), not the writer's.

## 6. The line test (every line, every time)

A line goes in only if it passes all of these, read aloud:

1. **It's an act.** You can name what the speaker wants from it.
2. **Only they could say it.** Swap the speaker for another person in the
   room and it breaks.
3. **One specific noun** from their world (a net, the north gate, lime, the
   tax man's dog), never "goods", "things", "trouble".
4. **No explaining, no labelling.** Nothing about how the game works; no
   "I am a …" unless it's a lie or a boast.
5. **Lopsided.** Not the same shape as the line before it. A long one, then
   two words; or a question answered with a question.
6. **Subtext.** There's something they're not saying, and you can feel it.
7. **One breath.** Readable aloud without a second breath, or it's two lines.
8. **It ends with a door.** A hook, a question, or a silence the player has
   to step into — not a full stop that closes the scene.
9. **Traits show as behaviour**, never as adjectives.
10. **It survives the second read.** Lines the engine will show a hundred
    times must not have a joke that dies on the second hearing.

## 7. The review loop (between the two sessions)

- A lines file is never merged by its writer alone. The other session reads
  it aloud (literally: it writes down how twenty lines sound as a
  conversation, `headless talk <seed> <town>` or the test's dump) and marks
  each against the ten tests. Three failures and the file goes back.
- **The table read**: generate thirty greetings in one town (the rg6 test
  does this) and read them as a crowd. If you can hear the same person three
  times, the pool is too thin or the conditions too loose.
- **The crowd is quiet** (Laz): a crowd member's pool is a greeting, one
  remark about their work or their personal detail, an occasional rumour, a
  farewell. Fewer, better lines beat many.
- The faction greeting problem: fix `pick` so a person's own facts (trade,
  personal detail, what's on their mind) can outrank a blanket faction line;
  the faction sets the *attitude* of the line that wins, not the line.

## 8. The working rhythm

1. Audit twenty old lines each against section 1 (this week).
2. Agree the voice bible with Laz's lore (he corrects section 4).
3. Rewrite the greetings and the island (Diʻub, Ḍeshu, the watch) under the
   lenses; review each other's.
4. Then the authored characters' own files (`data/lines/people/<key>.txt`),
   one character at a time, from their character card.
5. Every new line through the test; every file through the loop.

## 9. Scars: what went wrong in the second's own lines, and what fixed it

The second session wrote the faction voices (`factions.txt`), the leaders'
jobs (`leaders.txt`), Diʻub and the island (`diub.txt`, `salt_isle.txt`).
Read back a day later, these are the faults, each with what fixed it where
something did.

- **The slot wrote the line.** Every leader's offer has the same skeleton:
  the situation, the order, then `{coin} coin, <a flourish>`. Thirteen
  offers, thirteen lines that end on a price and a pat phrase ("from my own
  purse", "between friends", "if you live"). The slot `{coin}` sat at the
  end because that's where it was easiest to put. *Fix:* the price goes
  where the speaker would put it. A trader names it first; a proud man
  last, or not at all ("you'll be paid"); someone embarrassed by how little
  it is mumbles it in the middle. The slot is a fact, not a full stop.
- **Thanks that thank.** Every `thanks` line says thank you and the coin.
  People who've been helped rarely say "thank you" first: they check the
  thing, they say what it means to them, they change the subject. *Fix:*
  thanks lines are about the thing done, never about gratitude; the word
  "thanks" is allowed once in the file.
- **Flattery labelled as flattery.** Feyesh's "My friend, you're exactly
  who I hoped for ... between friends" says *I am a flatterer* in three
  places. A good flatterer flatters by noticing something true about you.
  *Fix (the Egri lens):* the flattery is an accurate observation used as a
  lever: "You came in by the back street. Most people don't know it's
  there."
- **Manifestos as greetings.** Faction greetings were the faction's
  slogan in a stranger's mouth ("Every traveller is a thread in something
  bigger"). A faction is an attitude a person *has*, not a line they
  *recite*. *Fix:* the faction changes how the ordinary thing is said: a
  Throne loyalist asks your business the way a customs clerk does; an Open
  Hand idealist asks it as if they'd like the answer.
- **Invented history.** Niralea's "My grandmother used to say..." gives a
  canon leader a grandmother Laz never wrote. `FORMAT.md` already says
  don't give a line facts the game doesn't know; for Laz's named people the
  rule is stricter: only what `docs/leaders.md` and the Project's lore say,
  or a detail so small it can't contradict anything (a habit, not a
  relative).
- **What worked, and why.** Ḍeshu's judgment: "Then you're mine. Put it to
  work, and see it's fed." It works because it's an act (a claim), the
  status is in the grammar (*it*, not *them*), it's lopsided (four words,
  then an order to someone else), and it leaves the player a door (what
  now?). Diʻub's guide lines worked when they were what he'd *do* ("Fire
  first. Then we argue about the rest"), and failed when they told the
  player what the lesson was.

## 10. Lines are assembled: write for neighbours you can't see

A talk is put together from pieces: greeting + topic + feeling + hook
(`talk.rs` `assemble_talk`), each picked separately. So:

- **Each piece must stand next to any other.** A greeting can't end with a
  question the topic then ignores; a feeling can't repeat the greeting's
  oath. The primary's audit found the greetings all ended in "What is it?",
  and then the topic answered a question nobody had asked.
- **Each piece owns one job.** The greeting places the speaker (who they
  are to you); the topic says what's on their mind; the feeling colours it;
  the hook leaves the door. If a greeting already leaves a door, that
  person's hook should usually be empty (`hook | |`).
- **Read pieces in the order they meet.** The review reads assembled talks
  (the rg6 dump, `headless talk`), never a file top to bottom: the file
  order hides the joins.
- **Kill the new tic, not just the old one.** Removing the closing question
  tends to put an imperative in its place ("Say it", "Speak"). Count how
  many lines in one slot end the same way (test 12).

## 11. What the engine's facts can and can't carry (today)

A line can only be as specific as its conditions. What a line can be
matched on now (`talk.rs` `facts`), and what it can't:

| Can | Can't (yet) |
|---|---|
| their people (`voice=`), trade (`job=`), honour band (`low_honour`, `mid_honour`, `high_honour`) | their **sex** or the listener's (no tag); their age |
| temper as combinations only: `hot` (bold and quick), `patient`, `gentle` (patient and sociable), `cold` (patient and unsociable) | boldness, sociability or wanderlust **alone**; so "bold *and* poor" can be written only as `hot, poor` |
| mood: `worried`, `hungry`, `afraid`, `content` | tiredness, cold, being wet, drink, grief |
| `has_coin` / `poor` | the five wealth steps (rosters have them; stage 4) |
| the listener: `stranger`, `known`, `warm`, `distrusts`, `trusts_you`, `confides`, `member`, `you=<people>` | the listener's trade, sex, what they're wearing, whether they're armed or hurt |
| when and where: `morning` to `night`, `place=`, `in_town`, `out_here`, `home_here`, `travelling` | the weather (`rain` exists but is never set; the weather now exists, so it could be) |
| faction and sub-faction, a leader's `task=`, a topic's own tags | what they said to *this* person last time (only "don't repeat a piece"); the personal detail and relations (rosters, stage 4) |

So **the trait table in section 4 is ahead of the engine.** Until the tags
exist, write the combinations through what's there (`hot, poor`, `cold,
worried`), and put the tags a line wanted but couldn't have in a comment
above it (`# wants: bold, sex=woman`). That list is the case for each new
tag.

## 12. The faction-greeting fix (`talk.rs` `pick`)

Today a piece's score is how many conditions it has, plus `FACTION_WEIGHT`
(2) if any condition is `faction=` or `task=`. A two-condition faction
greeting scores 4 and beats a three-condition line about the speaker's own
trade and mood, so a held town greets alike. The fix, for the dialogue
stage (no code yet: craft first):

1. **Score what's about the person above what's about the group.** A
   condition on the speaker's own facts (`job=`, mood, temper, honour, a
   personal detail, `leader=`) counts 2; a faction or place condition counts
   1; `task=` stays the strongest (it's a specific job).
2. **A faction line is a fallback in its slot, not an override:** it wins
   only when nothing about the person fits as well.
3. **The attitude travels without the words:** a faction can add a
   *feeling* piece ("...and the Throne's man in me says watch your step")
   instead of replacing the greeting. Then a guard who is also a Throne
   loyalist greets as a guard, and the loyalty leaks in the second piece:
   section 4's "one leads, the other leaks", done by the assembler.

## 13. Additions to the lenses and the test

- **The Ishiguro lens** (*The Remains of the Day*), for anyone whose status
  depends on not saying something: officials, servants, the bonded, the
  Binders' overseer. They talk round it with perfect manners; the omission
  is the whole character.
- **The Leonard lens** (Elmore Leonard), for crooks, the Sharp Hands,
  bandits: they talk shop sideways, they're funny without trying, nobody
  explains the crime.
- **Test 11: read it in the worst context.** The engine will show this line
  in the place and hour that fits its tags least (a "mind the step" on an
  open road; "come in out of the rain" at noon in a drought). If it's wrong
  there, add the tag that keeps it out, or change the line.
- **Test 12: count the slot's endings.** In one slot of one file, no more
  than one line in five may end the same way (a question, "Say it", a name,
  a blessing).
