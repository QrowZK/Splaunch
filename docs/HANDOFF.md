# Where Splaunch actually is

Written at handover, 2026-08-21. The point of this document is to be honest
about the difference between what has been *verified* and what merely
*compiles*, because the gap is larger than the commit history suggests.

## The one thing to read first

**No game has ever been launched from a Splaunch scenario.** Not once. The start
script's shape is taken from a real one, every field name is read from the
Zero-K code that consumes it, and the objective payloads are proven to survive
Zero-K's own decoder — but the loop has never been closed. Nothing here is
evidence that a generated scenario starts.

Closing that loop is the highest-value next task, and it wants a person at a
keyboard: build a scenario with one objective and one feature, press Test, and
watch what the engine does. Expect to find something. If the first attempt works
outright, be suspicious and check the objective actually evaluated rather than
merely appearing in the panel.

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
  which ships as `src-tauri/src/fixtures/mission-script.txt`. Tests assert our
  script carries every section and key the real one does.
- **The mission model.** Read out of the installed game — see
  `docs/MISSION-MODEL.md`, which names the archives and files for everything it
  claims. The headline: Zero-K's mission engine lives in the base game, not in
  mission archives, and one modoption (`singleplayercampaignbattleid`) arms it.
  Objectives, features, terraform, briefing text and placed units all travel as
  start-script values.
- **The custom-key transport.** Zero-K's decoder has two faults, both found by
  transcribing its Lua byte-for-byte and round-tripping payloads through the
  port. `customkey.rs` documents them and escapes around both. A test sweeps
  every byte value at every alignment through a faithful copy of the broken
  decoder.
- **Objective compilation.** Each of the seven author-facing goals compiles to a
  field combination taken from Zero-K's own annotated `sample_planet.lua`, and
  the tests decode the emitted modoption the way the game will and assert on the
  Lua that comes out.

## Not verified

- Whether a scenario launches at all. See above.
- Whether an objective, once launched, is evaluated the way the mapping assumes.
  "Build 5" and "have 5" differ by one flag (`countRemovedUnits`), and getting
  it wrong produces an objective that silently cannot be completed. The tests
  prove we emit the flag; nothing proves the flag means what the reading of
  `sample_planet.lua` says it means.
- Whether `startpostype = 2` is sufficient on every map. Zero-K's own script
  builder carries a comment saying it is required to stop maps crashing on
  undefined start positions, which is why it is set, but only a handful of maps
  have been tried on paper and none in the engine.
- The features path end to end. `PlaceFeature` resurrects anything named
  `*_dead` back into its unit, which is a nice property and completely untested
  here.

## Known gaps, roughly in order of value

1. **Launch something.** As above.
2. **In-game dialogue.** The modern mission system has none — `WG.AddConvo`
   exists in the base game with exactly the visual-novel shape (portrait, text,
   voice clip, duration, a queue, and it relocates to a subtitle bar during
   cutscenes) but nothing in the campaign gadget calls it. Only legacy mission
   archives do. `docs/MISSION-MODEL.md` §6 lays out the three options and
   recommends shipping the briefing first, which is nearly free once the
   transport exists.
3. **The `.sdd` importer.** Old SpringBoard projects are Spring *directory*
   archives containing `project.lua`, `model.lua` in `savetable.lua` format,
   `heightmap.data`, `metal.data` and `script.txt`. The format is recorded in
   the docs; nothing is built, because there has never been a sample project to
   test against. Do not build this speculatively — get a real `.sdd` first.
4. **The editor UI.** It is one screen and it shows. Objectives are a data model
   with no authoring surface worth the name.

## Standards this repo is held to

Not house style for its own sake — each of these caught something real.

- **Comments say *why*, not *what*.** The ones that earn their place record a
  decision or a trap: why the transport escapes, why the minimum window size is
  smaller than the default. If a comment restates the line below it, delete it.
- **Measure, do not assume.** Every claim above has a method attached. Two
  Zero-K bugs were found by porting its Lua and running payloads through it
  rather than reasoning about the code. When something can be checked, check it.
- **Say what is not known.** The "Not verified" section above is the format. A
  README that overstates its own project costs the next person a day.
- **Tests assert against the real thing** where one exists — the real mission
  script, the real decoder — rather than against our own idea of it.
- **CI lints with `-D warnings`.** Dead code fails the build. That is deliberate:
  four unused engine-launching functions rode along from Shiro into Sprofiler
  and were only noticed when CI existed.
- **Never commit credentials.** A password was committed to a related repo once
  and had to be changed. The e2e suite in the sibling Shiro repo reads one from
  the environment for this reason.
- **Do not connect to `zero-k.info:8200` for testing.** `LoginChecker` bans
  repeat failures by IP.

## Build environment

`cargo test` needs a working MSVC toolchain, and one machine involved here did
not have one: `VC\Tools\MSVC\<ver>` was missing its `include` directory and
`vcvarsall.bat` entirely, so `cl.exe` could not find `stdint.h` and the linker
could not find `msvcrt.lib`. That breaks any crate with a C dependency —
`aws-lc-sys`, via `rustls`, via `reqwest`. Repairing "Desktop development with
C++" in the Visual Studio Installer fixes it. CI is unaffected.

`customkey.rs` has no dependencies at all, so it can be tested on its own with
`rustc --test src-tauri/src/customkey.rs` when the full crate will not build.

## Releases

`.github/workflows/release.yml` publishes to a rolling `dev` tag on every push
to main, as `Splaunch_<version>_x64.zip` containing a portable `Splaunch.exe`,
plus the NSIS installer as a separate asset. Assets are never deleted: Shiro's
launcher pins an exact URL and hash, so removing one would 404 for every copy of
Shiro already shipped.

Shiro's catalogue pins a specific version, so a new release here does not reach
users until Shiro itself ships again. That is a consequence of compiling the
catalogue into the binary, which is what stops anyone adding an entry by serving
a file.
