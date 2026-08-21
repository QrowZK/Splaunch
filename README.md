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

`docs/DESIGN.md` has the research this is built on, including how a Zero-K
mission is actually put together and what the older archive format did.
`docs/MISSION-MODEL.md` is the later and more precise account: where the
mission engine actually lives, the objective vocabulary that exists, and two
faults in Zero-K's own decoder that this has to write around.

**If you are picking this up, read `docs/HANDOFF.md` first.** It says which
claims here are verified and which are not.

## Running it

Needs an existing Zero-K installation for the engine - Splaunch finds it the
same way the lobby does, and does not install or manage Zero-K itself.

    npm install
    npm run tauri dev

Builds are published to the `dev` release. The zip holds a portable
`Splaunch.exe` - unpack it anywhere and run it - and the setup `.exe` beside
it installs instead, if you would rather. Shiro's app launcher can also
install and launch it for you.

## Known gaps

- **It does not know where the water is.** Units are placed against the map's
  minimap, so nothing yet stops you putting a land unit in the sea. Real
  geometry means reading the map's heightmap out of its archive.
- **No scenario has ever been launched from it.** The script it writes matches
  the shape of Zero-K's own mission scripts - sections, keys, and the way values
  terminate, all asserted against a real one - and the objective payloads are
  proven to survive Zero-K's decoder. Matching a shape and starting a game are
  still different claims. This is the next thing to do.
- **Objectives are unit counts over time windows**, because that is all
  Zero-K's mission engine evaluates. Seven author-facing goals compile to its
  field combinations; there is no trigger graph and the modern game has no
  mechanism for one. `docs/MISSION-MODEL.md` has the vocabulary that exists.
- **Nothing speaks during a match.** The briefing is reachable; in-game
  dialogue needs a widget on the player's install. Same document, section 6.
