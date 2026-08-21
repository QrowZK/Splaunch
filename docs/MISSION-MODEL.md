# How Zero-K runs a mission, and what Splaunch should build on

Research notes, and the model that follows from them. Everything here was read
out of the installed game rather than from documentation or memory; file paths
are inside the archives named, from a Steam install of Zero-K on 2026-08-21.

The short version: **Splaunch does not need to ship a mission archive.** Zero-K
carries a mission engine in the base game, and it is configured entirely through
start-script modoptions - objectives, defeat conditions, features, terraform, map
markers, briefing text and placed units all travel as start script values. That is
the same file Splaunch already writes. The catch is that the transport those
values ride in has two decoder faults, and hitting either one silently destroys
the whole payload.

## 1. There are two mission systems, not one

**The legacy one** ships inside each mission archive: `mission_runner.lua`
(1,927 lines) as a gadget, with `mission_gui.lua` and `mission_messenger.lua`
as widgets, driven by a `mission.lua` data file at the archive root. Still
present in `User Interface Tutorial r22.sdz`, `Economy Tutorial r17.sdz`,
`quicktutorial.sdz`, `Unit Classes Tutorial r13.sdz` and `seeker.sdz`.

It is a genuine trigger graph: 25 condition types and 39 synced actions plus 28
unsynced ones. Triggers have `probability`, `maxOccurrences` and an `enabled`
flag; actions are scheduled onto game frames, so `WaitAction` shifts everything
after it later in the same trigger.

**The modern one** is `mission_galaxy_campaign_battle.lua` (1,616 lines,
GoogleFrog, 2017) in `zk-stable.sdz` - the base game - with
`mission_galaxy_battle_handler.lua` as its widget. This is what the current
campaign uses. It has no trigger graph at all: objectives are unit-count
comparisons over time windows.

Both gate on the same switch:

```lua
local campaignBattleID = Spring.GetModOptions().singleplayercampaignbattleid
if not campaignBattleID and not GG.load_galaxy_mission_handler then
	return
end
```

Set that one modoption to any value and the whole modern system turns on, in
stock Zero-K, with no archive. That is the single most important finding here.

The legacy system is more expressive and the modern one is more deliverable. The
recommendation below picks the modern one, and section 6 says what that costs.

## 2. The dialogue systems

Zero-K has three text surfaces, and it is worth being precise about which is
which, because they are often spoken of as one thing.

### Convo - the visual-novel one

`ConvoMessageAction` in the legacy runner reaches `WG.AddConvo`, implemented in
`luaui/widgets/mission_messagebox_zk.lua` in the base game:

```lua
local function AddConvo(text, fontsize, image, sound, time)
  convoQueue[#convoQueue+1] = {text=text, fontsize=fontsize, image=image, sound=sound, time=time}
  if #convoQueue == 1 then ShowConvoBox(convoQueue[1]) end
end
```

That is the visual-novel shape exactly: a **queue** of lines, each with a
**portrait** (drawn as a square at the left edge, `keepAspect`), a **voice clip**
(played on the `ui` channel), and a **duration** in game frames - default 150,
so five seconds at 30fps. When one expires the next plays automatically.
`ClearConvoMessageQueueAction` flushes the queue.

Two details worth keeping:

- It is **time-advanced, not click-advanced**. There is a Next button, but it is
  driven by a game rules param (`tutorial_show_next_button`) rather than by the
  convo system itself.
- **It relocates during a cutscene.** Normally the box sits centred at 20% or
  80% screen height; inside a cutscene it becomes a full-width subtitle bar at
  the top. So the same authored line reads as chatter during play and as a
  subtitle during a cutscene, with no extra authoring.

### Briefing - the pre-mission one

The modern system's equivalent, carried in the `planetmissioninformationtext`
modoption and rendered by `InitializeBriefingWindow`:

```lua
{ name = "Folsom", description = "...", tips = { {text=..., image=...}, ... } }
```

A titled window with the objective lists side by side, then a scrolling body of
text-and-image entries, dismissed with Continue. Prose with pictures rather than
a conversation - closer to a mission dossier than to dialogue.

### Persistent message box

`GuiMessagePersistentAction`, a box with back/forward arrows through message
history. Reference material during play, not narrative.

### What this means

**The modern campaign has no in-game dialogue.** `WG.AddConvo` is published as
an API by the base game, but nothing in the campaign gadget calls it - the only
callers are `mission_gui.lua` inside legacy mission archives. A start-script
scenario gets the briefing and the objective panel, and nothing that speaks
during play.

That is the real gap between what Splaunch can do today and what a
visual-novel-ish scenario wants, and section 6 is about closing it.

### On Ren'Py

There is none, anywhere: no Ren'Py in the Zero-K install, the game archive, or
the menu archive. The nearest thing is `mission_cutscene.lua`, a 425-line LuaUI
widget doing camera moves, letterboxing, fades and skip handling. The convo
queue above is what carries the words.

This is worth stating plainly because the resemblance is real - portrait, text
box, voice clip, auto-advance, a queue you can flush - and a Ren'Py-shaped
authoring model would map onto it well. It just has to be built rather than
integrated.

## 3. The transport, and its two faults

Structured data reaches the game as base64'd Lua table literals in start script
values:

```lua
UsefulTableToCustomKey(t) = Base64Encode(TableToString(t))
CustomKeyToUsefulTable(s) = loadstring("return " .. Base64Decode(s:gsub('_','=')))
```

`TableToString` writes `[i]=` for numeric keys and a bare name otherwise, a
trailing comma on every entry, and no whitespace. `Base64Encode` is the URL-safe
alphabet (`-` for 62, `_` for 63) with `=` for padding.

Both faults below were found by transcribing the Lua byte-for-byte and
round-tripping payloads through it. They are reproduced in `customkey.rs`'s
tests, which run our encoder's output through a faithful port of Zero-K's
decoder.

**Fault 1 - the underscore.** The encoder emits `_` for sextet 63. The decoder
rewrites `_` to `=` *before* decoding, and `=` is absent from its alphabet, so
it reads as end-of-data. Nothing performs the inverse substitution anywhere in
either archive - the swap is decode-side only.

For ASCII payloads a 63 sextet arises from exactly one character: `?`, and only
when it lands at an offset ≡ 2 (mod 3). So a question mark in an objective
description truncates the payload; a truncated Lua literal does not parse;
`CustomKeyToUsefulTable` returns nil; **the mission loses every objective at
once.** An author would see a question mark in one description and no objectives
at all, which is not a clue that leads anywhere.

**Fault 2 - the modulo.** The last byte of each triple is assembled as
`lor(lsh(chars[3],6) % 192, chars[4])`. When the top two bits are both set that
`% 192` zeroes them, so any byte >= 0xC0 is corrupted. Every UTF-8 lead byte is
>= 0xC0, so accented and non-Latin text is mangled independently of fault 1.

**The fix.** Both faults key on the *bytes on the wire*, so put nothing on the
wire that can trigger them: write every risky byte as a Lua decimal escape
(`\195`), which is plain ASCII in transport and becomes the original byte again
when Lua parses it. `customkey.rs` escapes `?`, `"`, `\`, DEL, control bytes and
everything >= 0x80. A test sweeps every byte value at every alignment.

We do not patch Zero-K's decoder. It is the thing reading our output and it is
not ours; we write what it reads correctly. Worth reporting upstream, though -
it is a live bug for anyone authoring campaign content in a language other than
English.

## 4. The vocabulary that actually exists

From `sample_planet.lua` in the menu archive, which is Zero-K's own annotated
reference, cross-checked against the gadget that consumes it.

### Objectives

`victoryByTime` is a special case. Everything else is a unit-count comparison
with a time window:

| Field | Meaning |
| --- | --- |
| `comparisionType` | 1 = at least, 2 = at most (spelling is theirs) |
| `targetNumber` | the number compared against |
| `unitTypes` | your unit names that count |
| `enemyUnitTypes` | enemy unit names that count |
| `satisfyAtTime` / `satisfyByTime` / `satisfyUntilTime` / `satisfyAfterTime` | the time window, in seconds |
| `satisfyOnce` | true the moment it is ever satisfied |
| `satisfyForever` | must hold to the end |
| `satisfyForeverAfterFirstSatisfied` | becomes `satisfyForever` once first met |
| `countRemovedUnits` | dead units still count - this is how "build 5 Glaives" works |
| `onlyCountRemovedUnits` | only dead ones count - this is how "kill 5" works |
| `failOnUnitLoss` | losing a counted unit fails it outright |
| `lockUnitsOnSatisfy` | freeze the satisfying set, so overbuilding cannot pad it |
| `capturedUnitsSatisfy` / `alliedUnitsSatisfy` | whether captured or allied units count |
| `completeAllBonusObjectives` | the meta-objective |
| `description`, `image`, `imageOverlay` | what the player sees |

That is enough for a real spread of goals - hold a position by holding units
alive, rush a build order, kill a specific thing before a deadline, survive a
timer - without a trigger graph.

### Defeat conditions

`vitalUnitTypes`, `defeatIfUnitDestroyed`, `loseAfterSeconds`,
`ignoreUnitLossDefeat`, indexed by allyTeam.

### Placed units

`name`, `x`, `z`, `facing`, plus `buildProgress`, `experience`, `commands`,
`patrolRoute`, `movestate`, `stunTime`, `invincible`, `noControl`,
`notAutoAttacked`, `terraformHeight`, `spawnRadius`, `orbitalDrop`, `mapMarker`,
`difficultyAtLeast` / `difficultyAtMost`, and `bonusObjectiveID` - which is how
"keep *this particular* Reaver alive" is expressed: the unit points at the
objective rather than the objective describing the unit.

They travel as **team custom keys**, chunked 40 to a key:
`extrastartunits_1`, `extrastartunits_2`, ... and `neutralstartunits_N` on the
modoptions table for Gaia-owned units.

### Features - the map dressing

`featurestospawn` (built from `gameConfig.initialWrecks`): `name`, `x`, `z`,
`facing`, `difficultyAtLeast`, `difficultyAtMost`. Facing is random if omitted.

A nice detail worth surfacing in the editor: a feature whose name contains
`_dead` is resurrectable, and the gadget wires that up automatically -
`SetFeatureResurrect` back to the unit whose name is the feature's minus
`_dead`, with the position sanitised to the building grid.

### The rest

`initalterraform` (spelling theirs), `planetmissionmapmarkers`,
`planetmissiondifficulty` (1-3, gating the `difficultyAt*` fields),
`commandertypes`, `campaignunlocks`, `typevictorylocation`.

One non-obvious requirement from the menu's own script builder:

```lua
startpostype = 2, -- Choose is required to make maps not crash due to undefined start positions.
```

Splaunch already writes `StartPosType=2`, for a different reason. Keep it.

## 5. The model for Splaunch

Compile to a start script, as now, and add:

1. `singleplayercampaignbattleid` - any stable value, to arm the mission engine.
2. `objectiveconfig` and `bonusobjectiveconfig` - the real objectives.
3. `defeatconditionconfig` - what losing means.
4. `featurestospawn` - placed wrecks and rocks.
5. `planetmissioninformationtext` - the briefing.
6. `extrastartunits_N` per team, replacing the side-car JSON.

The editor's objective UI should not expose the field table above directly.
Those fields combine into a handful of things authors actually mean, and the
combinations are not guessable:

| What the author picks | What it compiles to |
| --- | --- |
| Survive until *t* | `satisfyUntilTime`, at least 1 of the vital units |
| Keep *this unit* alive until *t* | same, with `bonusObjectiveID` on the unit |
| Build *n* × *unit* by *t* | `satisfyByTime`, `countRemovedUnits`, at least *n* |
| Have *n* × *unit* at once | `satisfyOnce`, at least *n*, `lockUnitsOnSatisfy` |
| Destroy all *enemy unit* by *t* | `satisfyByTime`, `enemyUnitTypes`, at most 0 |
| Kill *n* × *enemy unit* | `onlyCountRemovedUnits`, at least *n* |
| Win before *t* | `victoryByTime` |

Seven author-facing goals, each a verified field combination, is a better
starting point than exposing 24 fields and a spelling mistake.

`customkey.rs` implements the transport. The rest is a straightforward mapping,
and the side-car `write_units` JSON goes away - its doc comment says it exists
because "inventing a modoption name that Zero-K does not define would be a guess
dressed as an integration", and that is no longer the situation.

## 6. Dialogue: what to build

The modern system has no in-game speech, so there are three honest options.

**Ship the briefing only.** Free, works today, entirely within the start script.
Gets prose and images in front of the player before the match. Do this first
regardless - it is a few lines once the transport exists.

**Ship a widget alongside the scenario.** `WG.AddConvo` exists in the base game
and takes exactly the visual-novel shape. A small widget could read a convo
script from a modoption and drive the queue off game time and simple events. The
cost is that a widget has to reach the player's install, which means writing to
their Zero-K directory - a real imposition, and one that should be opt-in and
reversible.

**Ship a mission archive.** The legacy runner already does all of this, and more
besides: cutscenes, camera control, fades, music, and 25 condition types to fire
lines off. The cost is building `.sdz` archives, and that a scenario stops being
one file you can send someone.

The recommendation is the first now and the second next, with the third left
open. The second is where a Ren'Py-shaped authoring model would live: a script
of speaker/portrait/line/duration, compiled to a queue, with the cutscene
relocation behaviour coming free from the base game.

## 7. What is verified, and what is not

Verified by reading the shipped code and by test:

- The activation guard, and that it is a modoption rather than an archive.
- The full field vocabulary in section 4, from the gadget that consumes it.
- The transport encoding, and both decoder faults, by byte-for-byte port.
- That our encoder's output survives Zero-K's decoder for every byte value at
  every alignment.

**Not verified: no game has ever been launched from a Splaunch scenario.** The
script shape is taken from a real one and the field names are read from the code
that consumes them, but the loop has never been closed. Nothing in this document
should be taken as proof that a generated scenario starts, until one has.

That is the next thing to do, and it wants a person at a keyboard: launch a
scenario with one objective and one feature, and see whether the briefing appears
and the objective panel populates.
