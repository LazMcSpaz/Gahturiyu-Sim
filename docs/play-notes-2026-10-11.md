# Laz's play notes, 11 Oct 2026

What Laz found playing `main` on his PC (11 Oct), his words lightly tidied. Each item has an id
so branches, commits and reports can name it. Laz has said to work on them; the **Owner** column
says which branch takes each. Items marked *decision* wait for Laz's answer before building.

| Id | Area | Note | Owner |
|---|---|---|---|
| P1 | Clicks | In dialogue the reply options flicker rapidly while the mouse is over them; so do items in the inventory. | desktop (`fix/play-ui`) |
| P2 | Clicks | Clicking a door on a Roduro home is fiddly: you have to find the exact sweet spot. | desktop |
| P3 | Clicks | Selected things get a glowing outline, not a circle underneath. Many buildings' selection boxes spill into places they shouldn't. | desktop |
| P4 | Clicks | A bandit camp is selected as one thing with a circle round the whole camp. The camp isn't one object: tents, people, containers are selected and used one by one. | desktop |
| P5 | Clicks | The hover tooltip waits until the mouse has been still on something for 1.5 s. | desktop |
| P6 | Clicks | Shift-clicking squad members works up to the fourth, then they all get deselected. | desktop |
| P7 | Camera | With a squad member selected, the camera can lock on to them (squad only). | desktop |
| P8 | Camera | Double-clicking a portrait centres the camera on that member. | desktop |
| P9 | Camera | Swap the camera keys: A pans left, D pans right (Laz confirmed). | desktop |
| P10 | Menus | Opening one menu closes any other. | desktop |
| P11 | Menus | Escape closes whatever menu is open. Space only ever controls time. | desktop |
| P12 | Menus | There's no stats screen. *decision* | desktop |
| P13 | Menus | There's no menu for settings or for starting a new game. *decision* | desktop |
| P14 | Menus | The speed settings are too severe: from the third (1 min/s) up, things happen almost instantly. *decision* | desktop |
| P15 | Menus | The town panel shows town information even out in the wilds. | desktop |
| P16 | Magic | No feedback when casting beyond the energy going down. Casting is silent. Needs a visual cue and a sound. | cloud A (`fix/play-magic`) |
| P17 | Magic | The spells window stayed the same after switching to another member. It should follow the selection. | cloud A |
| P18 | Magic | Douse wouldn't let him pick any torch or fire. | cloud A |
| P19 | Magic | Spells can target objects; doors have health and can be broken. *decision* | cloud A |
| P20 | Magic | Anyone under a magic effect (squad or not) shows it, e.g. an icon. | cloud A |
| P21 | Walking | He walked through a fence and through the hall. | cloud B (`fix/play-walking`) |
| P22 | Walking | Clicking to walk up to a trader's counter left him stuck inside it. | cloud B |
| P23 | Fights | Enemies shouldn't always chase so far. Past a point some let you run and go back to help their group. | cloud C (`fix/play-fights-talk`) |
| P24 | Fights | Bandits are just called "bandit". Personal names like everyone else's are confusing. | cloud C |
| P25 | Body | Healing happens too fast. | cloud C |
| P26 | Talk | NPCs always offer the same dialogue options, even out in the wilds. | cloud C |

## Decisions asked of Laz (11 Oct)

- P12: what the stats screen shows.
- P13: the game menu (Escape with nothing open?) and what's on it.
- P14: the new speed steps.
- P19: doors with health is a new system during the freeze: build it now or later?
- P25: how slow healing should be.

Answers go here as they come.
