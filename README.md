# Splaunch

A scenario editor for **Zero-K**. Place units on a map, set objectives, and
press Test to launch straight into the real game.

## What it produces

A **Spring start script**, not a file format of its own. Zero-K's modern
campaign expresses a whole mission - map, teams, AIs, start units, objectives -
as options on a start script read by a gadget that already ships with the game.

The consequence is the useful part: **Test is not a preview.** There is no
second renderer to keep honest and no fidelity gap to apologise for, because
what you are testing is the game.

**`docs/SCENARIOS.md` is how to make one.** `docs/MISSION-MODEL.md` is how
Zero-K's mission engine works underneath, with the archives and files every
claim was read out of. `docs/DESIGN.md` is the older research this was built
from. `docs/HANDOFF.md` says which claims are verified and which are not.

## Running it

Needs an existing Zero-K installation for the engine - Splaunch finds it the
same way the lobby does, and does not install or manage Zero-K itself.

    npm install
    npm run tauri dev

Builds are published to the `dev` release. The zip holds a portable
`Splaunch.exe` - unpack it anywhere and run it - and the setup `.exe` beside
it installs instead, if you would rather. Shiro's app launcher can also
install and launch it for you.

## Start here

Press **Open the example**. *First Contact* ships inside the binary: a
commander and three Glaives against a small outpost, with three objectives and
some reclaimable wrecks in between. Press Test and it should start.

You need to have played its map once, or any map you pick - Zero-K downloads
maps on demand, so a fresh install has a handful and the catalogue lists 343.
Splaunch marks the ones you do not have.

## What it knows about Zero-K

Everything it needs is on the disk that already has it, so it reads it rather
than guessing:

- **The roster** comes out of `games/zk-stable.sdz`, which is a zip - 275 unit
  definitions, with both the name a player uses and the name the engine uses.
  A Glaive is `cloakraid`, and the palette shows both.
- **The Zero-K version** comes from that archive's `modinfo.lua`, because
  `GameType` has to name the archive exactly.
- **The engine version** is whichever is newest under `engine/`.
- **The AIs** are whatever is under `AI/Skirmish`.
- **The maps you actually have** are what is in `maps/`.

A vendored copy of the roster, pinned to a Zero-K commit, stands in when there
is no install yet. The install always wins.

## Known gaps

- **It does not know where the water is.** Units are placed against the map's
  minimap, so nothing yet stops you putting a land unit in the sea. Real
  geometry means reading the map's heightmap out of its archive, which is
  `.sd7` - 7-zip, where the game archives are zip.
- **No scenario has been launched by the author of this code.** The three
  reasons a launch could not previously have worked are fixed and covered by
  tests, and every field is checked against the gadget that consumes it, but
  nobody has yet sat down at a machine with Zero-K on it and pressed Test.
  That is the one remaining thing worth doing, and `docs/HANDOFF.md` says what
  to expect.
- **Objectives are unit counts over time windows**, because that is all
  Zero-K's mission engine evaluates. Seven author-facing goals compile to its
  field combinations; there is no trigger graph and the modern game has no
  mechanism for one.
- **Nothing speaks during a match.** The briefing is reachable; in-game
  dialogue needs a widget on the player's install. `docs/MISSION-MODEL.md` §6.
