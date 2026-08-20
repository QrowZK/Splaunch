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

## Running it

Needs an existing Zero-K installation for the engine - Splaunch finds it the
same way the lobby does, and does not install or manage Zero-K itself.

    npm install
    npm run tauri dev

Builds are published to the `dev` release: download the zip, unpack it, run the
installer inside. Shiro's app launcher can also install and launch it for you.

## Known gaps

- **It does not know where the water is.** Units are placed against the map's
  minimap, so nothing yet stops you putting a land unit in the sea. Real
  geometry means reading the map's heightmap out of its archive.
- **No scenario has been launched from it yet.** The script it writes matches
  the shape of Zero-K's own mission scripts - sections, keys, and the way values
  terminate, all asserted against a real one - but matching a shape and starting
  a game are different claims.
- **Objectives are sentences, not triggers.** That is deliberate for now; see
  the design document before making it a node graph.
