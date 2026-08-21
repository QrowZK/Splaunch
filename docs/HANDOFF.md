# Where Splaunch actually is

Written at handover, 2026-08-21, and revised the same day after a working
session. The point of this document is to be honest about the difference
between what has been *verified* and what merely *compiles*.

## The one thing to read first

**No game has been launched from a Splaunch scenario by anyone who has written
its code.** That has not changed. What has changed is that there were three
specific reasons it could not have worked, and they are fixed:

1. **The engine was never told where Zero-K is.** `spawn_plan` passed no
   environment at all. Shiro's version - the one that demonstrably launches
   games - sets `SPRING_DATADIR` and `SPRING_WRITEDIR`, and its comment says
   why: without them the engine writes into the user's Documents folder and
   finds none of the installed games or maps. Splaunch's copy had dropped both,
   and also ran the engine from the data root rather than from its own folder.
2. **`GameType` was always empty.** Nothing ever discovered which Zero-K to
   run. The frontend hardcoded `game=""`, and `problems()` did not check it, so
   a scenario compiled to a perfectly well-formed script that starts nothing.
3. **The engine version was always empty.** `App.jsx` read `preview.engine`
   from the first commit; `LaunchPreview` had no such field and never had one.
   TypeScript declared it optional, so it did not even error. Every launch
   asked for engine `""`.

All three came from the same place: inside the lobby, the server supplied the
engine and game versions. When this was taken out of the lobby nothing replaced
them. `game.rs` now reads both off the disk.

So the loop is still unclosed, but it is no longer unclosed for reasons nobody
had noticed. Closing it still wants a person at a keyboard with Zero-K
installed: press **Open the example**, press **Test**, watch what the engine
does. Expect to find something. If it works outright, check that the objective
panel populates and that an objective actually evaluates rather than merely
appearing.

## What the thing is

One screen. Place units on a map, set objectives, press Test, and the real game
starts. It is deliberately not a lobby: no account, no server connection,
nothing to log in to. The only network call it makes is to Zero-K's public
content service for the map catalogue (343 maps).

It began as a screen inside the Shiro lobby and was taken out, which is why some
comments still argue with a design decision that was made for a lobby.

## Verified, and how

- **The map catalogue.** Live call to `GetPublicCommunityInfo`; 343 maps came
  back. `maps.rs`.
- **The start script shape.** Compared field-by-field against
  `_missionScript.txt` from Zero-K's own `User Interface Tutorial r22.sdz`,
  which ships as `src-tauri/src/fixtures/mission-script.txt`.
- **The mission model.** Read out of the installed game - see
  `docs/MISSION-MODEL.md`.
- **The custom-key transport.** Zero-K's decoder has two faults, both found by
  transcribing its Lua byte-for-byte and round-tripping payloads through the
  port. A test sweeps every byte value at every alignment.
- **The objective vocabulary, against the gadget that consumes it.** Every
  field `MISSION-MODEL.md` §4 claims is read by
  `mission_galaxy_campaign_battle.lua` was checked by reading it. Three results
  worth recording:
  - `COMPARE.AT_LEAST = 1`, `AT_MOST = 2`. Matches.
  - `removedUnits` increments **only** when `countRemovedUnits` or
    `onlyCountRemovedUnits` is set, and the count is `live + removed`. So
    "build 5" does mean cumulative production. This was listed as unverified
    before; it is verified now, by reading rather than by launching.
  - **`objectiveconfig` is not read by the gadget at all.** The string does not
    occur in it. Splaunch used to send the whole objective payload twice, to
    that key and to `bonusobjectiveconfig`, in a start-script value that has a
    length limit. Only the bonus key is real.
- **The roster.** 275 unit definitions, read from Zero-K's own `units/`. Two
  details cost a measurement each: a unit's internal name is its **table key**
  (`return { cloakraid = ... }`), not its filename - they agree for 274 of 275
  and `damagesinkrock.lua` defines `rocksink` - and grouping units by what
  builds them needs factories to outrank `athena`, which builds a 22-unit
  cross-section of six factories.

## The palette was not Zero-K's

Worth its own heading, because it is the sharpest evidence that this had drifted
away from the game it is for. The unit palette was 23 hand-written names -
`armpw`, `corhlt`, `armmex`, `armcom` - and **Zero-K defines none of them.**
They are Balanced Annihilation's. Checked against the roster: 23 of 23 missing.

The consequence was total and silent. The gadget resolves placed units and
objective unit types through `UnitDefNames` and ignores what it cannot find, so
every scenario built with that palette would have placed nothing and carried
objectives that could never be met. A Glaive is `cloakraid`; the palette now
shows both names and comes from the installed archive.

## Not verified

- Whether a scenario launches at all. See above.
- Whether an objective, once launched, is evaluated the way the mapping assumes.
  The field semantics are now read from the gadget rather than inferred, which
  is a much shorter distance to cover, but the engine has still never run one.
- Whether `startpostype = 2` is sufficient on every map. Zero-K's own script
  builder carries a comment saying it is required to stop maps crashing on
  undefined start positions, which is why it is set.
- The features path end to end. `PlaceFeature` resurrects anything named
  `*_dead` back into its unit, which is a nice property and still untested here.
- Whether the map dimensions in the content service's `MapItem` mean what this
  assumes. `Width` and `Height` are read as map units of 512 elmos, which
  matches the one sample in the tests (`Height 16`) and Zero-K's own way of
  describing map sizes. It could not be re-checked live from the machine this
  was written on, so the editor shows the number and lets an author correct it.

## Known gaps, roughly in order of value

1. **Launch something.** As above.
2. **Terrain.** Units are placed against the minimap, so nothing knows where the
   water is. The map's heightmap lives in a `.sd7`, which is 7-zip; the game
   archives are zip and already readable.
3. **In-game dialogue.** The modern mission system has none. `WG.AddConvo`
   exists in the base game with exactly the visual-novel shape and nothing in
   the campaign gadget calls it. `docs/MISSION-MODEL.md` §6 has the three
   options; the briefing is shipped, which was the free one.
4. **The `.sdd` importer.** Old SpringBoard projects are Spring *directory*
   archives. The format is recorded in the docs; nothing is built, because there
   has never been a sample project to test against. Do not build this
   speculatively - get a real `.sdd` first.
5. **Unit orders.** The gadget reads `commands` and `patrolRoute` off a placed
   unit, so a scripted patrol is expressible and is not yet exposed.

## Standards this repo is held to

Not house style for its own sake - each of these caught something real.

- **Comments say *why*, not *what*.** The ones that earn their place record a
  decision or a trap.
- **Measure, do not assume.** Every claim above has a method attached. The two
  Zero-K decoder bugs were found by porting its Lua and running payloads
  through it; the three launch blockers were found by diffing against Shiro's
  working launcher and by reading what the frontend actually asked for.
- **Say what is not known.** The "Not verified" section is the format.
- **Tests assert against the real thing** where one exists - the real mission
  script, the real decoder, the real roster. The shipped example is compiled by
  a test, and every unit name in it is checked against the roster, because an
  example that places nothing is worse than no example.
- **CI lints with `-D warnings`.** Dead code fails the build.
- **Never commit credentials.**
- **Do not connect to `zero-k.info:8200` for testing.** `LoginChecker` bans
  repeat failures by IP. The relay that used to point at it has been deleted.

## Build environment

`cargo test` needs a working MSVC toolchain on Windows, and one machine involved
here did not have one. Repairing "Desktop development with C++" in the Visual
Studio Installer fixes it.

**On Linux the whole suite runs** once the GTK stack is present:

    sudo apt-get update
    sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
        libayatana-appindicator3-dev librsvg2-dev

The `apt-get update` is not optional - without it the pinned package versions
404. This is a better fallback than the one below.

`customkey.rs` has no dependencies at all, so it can still be tested on its own
when the full crate will not build - but pass the edition, or you get two
misleading warnings:

    rustc --edition 2021 --test src-tauri/src/customkey.rs

Without `--edition 2021` it compiles as edition 2015, where `assert!(cond,
"{x}")` does not interpolate, and rustc warns about unused formatting
placeholders that are perfectly fine in the real crate. It only covers that one
file; `scenario.rs` needs the full build.

## Releases

`.github/workflows/release.yml` publishes to a rolling `dev` tag on every push
to main. Assets are never deleted: Shiro's launcher pins an exact URL and hash,
so removing one would 404 for every copy of Shiro already shipped.

Shiro's catalogue pins a specific version, so a new release here does not reach
users until Shiro itself ships again.
