# How to make a Zero-K scenario

This is the practical document. `MISSION-MODEL.md` says how Zero-K's mission
engine works and where the claims come from; this says what to click.

## Before you start

You need Zero-K installed, and you need to have **played the map at least
once**, or at least downloaded it. Zero-K fetches maps on demand through its
own lobby, so a fresh install has a handful of maps and the catalogue lists 343.
Splaunch greys out the ones you do not have and marks them *Not installed* -
picking one anyway gets you an error from the engine about a missing archive.

Splaunch does not install or manage Zero-K. It finds the install the same way
Shiro does, which is the same way the official lobby does, and reuses its
engine, its games and its maps.

## The fastest way to see one work

Open Splaunch and press **Open the example**. That loads *First Contact*, which
ships inside the binary: your commander and three Glaives bottom-left, a small
enemy outpost north-east, some reclaimable wrecks in between, and three
objectives. Press **Test**.

If it starts, the loop works on your machine and everything below is editing.
If it does not, the message says which of the three things is missing - Zero-K,
an engine, or the map.

## Making one from scratch

### 1. Pick a map

The map picker is the first screen. Installed maps sort first. Choosing one sets
the scenario's size in elmos from the catalogue, which is what makes a click in
the middle of the map land in the middle of the map. If the catalogue does not
know the size, the footer field says 4096 and you can correct it - a Zero-K map
described as "12x12" is 12 × 512 = 6144 elmos on a side.

### 2. Place things

The palette on the left is the roster of the Zero-K you actually have, read out
of its own archive. Each entry shows both names: **Glaive** is what a player
calls it, `cloakraid` is what the engine calls it, and the second one is what
travels in the script.

- **Units** tab, pick a unit, pick a team colour at the bottom, click the map.
- **Wrecks** tab places features. A feature named `<unit>_dead` is reclaimable,
  and Zero-K wires it up to resurrect back into the unit it came from, which is
  free scenery with a purpose.
- **Marks** labels a place. The player sees them on the map from the start.
- Click a placed unit to edit it. Facing, stance, how far built it is,
  veterancy, invincibility, and whether Gaia owns it instead of a team.

Select a unit and you can also give it a **patrol**: click *Draw a route*, then
click the map, and it walks those points for ever. This is as close as the
modern mission system gets to scripted behaviour - there is no trigger graph to
hang orders on, so a sentry walking a line is built out of a patrol. *Patrol on
the spot* is the one-click version.

A unit can also be set to exist **only at some difficulties**, which is how one
scenario becomes three. The difficulty it is played at is on the Teams tab; the
example's second Lotus turret only appears on Hard.

Nothing here knows where the water is. Positions are placed against the minimap,
so a land unit can be dropped in the sea and nothing will stop you. Check when
you test.

### 3. Say who is playing

The **Teams** tab. One team has to be you; the rest get an AI from the ones your
install has under `AI/Skirmish`. Teams that share a **side** are allies, and a
scenario where everybody is on the same side ends the moment it starts.

An enemy team with no AI is not a mistake. Its units sit where you put them and
shoot back when you come into range, which is exactly what a scripted outpost
should do - the example works this way.

### 4. Say what losing means

Also on the **Teams** tab. Without a defeat condition a side is only beaten when
it has nothing left at all, which is a long way to lose a mission that was about
one commander. "Loses when this is gone" is the usual one.

Give both sides one. A scenario with no way to lose is a sandbox, and one with
no way to win is a diorama.

### 5. Write the objectives

The **Objectives** tab, and this is the part worth understanding.

Zero-K's mission engine does not have a trigger graph. It evaluates **unit
counts over time windows** - 24 fields whose useful combinations are not
guessable, one of which is spelled `comparisionType`. Splaunch does not show you
that table. You pick one of seven things people actually mean:

| Pick this | And the player must |
| --- | --- |
| **Survive until** | keep at least one of the named unit alive to the deadline |
| **Build n by** | finish that many before the clock, counting ones that died after |
| **Have n at once** | hold that many alive at the same moment |
| **Destroy all by** | leave none of that enemy unit standing by the deadline |
| **Kill n** | destroy that many, however long it takes |
| **Win before** | win the match before the clock runs out |

The line under each objective in the list is what the game will check, in
words. Deadlines are typed as `m:ss`.

**"Build 5" and "have 5" are different objectives.** Build counts everything you
finished, so losses are forgiven; have needs them alive together. They differ by
one flag in the payload, and picking the wrong one produces an objective that
looks reasonable and cannot be completed.

Below the objectives are **notes**, which are sentences the game shows in the
briefing but does not check. Not every intention is a unit count.

### 6. Write the briefing

The **Briefing** tab. It appears in Zero-K's own briefing window before the match
starts, under the scenario's name, with the notes beside it.

This is the only place your words reach the player. The modern mission system
has no in-game dialogue at all: `WG.AddConvo` exists in the base game with
exactly the visual-novel shape, and nothing in the campaign gadget calls it.
`MISSION-MODEL.md` §6 has the three ways that could change.

Write a question mark if you want one. The transport escapes it - unescaped, a
`?` at the wrong byte offset truncates the payload and the mission loses *every*
objective at once, which is why `customkey.rs` exists.

### 7. Test, then save

**Test** writes a start script and launches the real game into it. It is not a
preview; there is no second renderer to be wrong. The button says *Running*
until the engine exits, and the editor comes back by itself.

**Save** writes a `.splaunch` file - plain JSON, with a format version. It does
not record which Zero-K it was built against, because that is a property of the
machine and not of the scenario, so a scenario you send someone runs against
their install.

## What a scenario becomes

A Spring start script, and nothing else. No archive to build, no server to
publish to, no file format of Splaunch's own on the way to the game.

Zero-K's mission engine lives in the base game, and one modoption arms it:

```
singleplayercampaignbattleid=splaunch;
```

Everything else travels as start-script values beside it - objectives, defeat
conditions, features, the briefing - as base64'd Lua tables, and the placed
units ride on the team that owns them. `MISSION-MODEL.md` §3 has the transport
and the two faults in Zero-K's decoder that it has to write around.

## When it does not work

**"No Zero-K installation found"** - Splaunch looks in the standalone
installer's directory, your home directory, the Spring data directories and
every Steam library. If yours is somewhere else, it can be pointed at it.

**"engine ... is not installed"** - Zero-K downloads engines on demand too.
Start any game once in the official lobby.

**The map is greyed out** - you do not have it. Play it once in the lobby.

**It starts, and there are no objectives** - the payload did not survive the
decoder. This should not happen; `customkey.rs` sweeps every byte value at every
alignment through a port of Zero-K's own decoder. If it does, it is a bug worth
reporting with the scenario file attached.

**It starts, and an objective never completes** - check "build" against "have",
and check that the unit named is the one that actually spawns. The gadget
resolves unit names through `UnitDefNames` and silently ignores what it cannot
find, so a name that is one character wrong is an objective that can never be
met. The palette only offers real names; typing one in by hand is where this
goes wrong.
