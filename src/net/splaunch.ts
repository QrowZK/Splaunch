/**
 * Splaunch's backend, from the front.
 *
 * A scenario compiles to a Spring start script in Rust and launches the real
 * game, so there is no preview to keep in sync - see
 * `src-tauri/src/scenario.rs` and `docs/DESIGN.md`.
 */
import { invoke } from "@tauri-apps/api/core";
import { inTauri } from "./connection.ts";

export interface Placed { unit: string; team: number; x: number; z: number }
export interface ScenarioTeam { id: number; ally: number; ai: string | null; colour: string }
export interface Scenario {
  name: string;
  map: string;
  game: string;
  teams: ScenarioTeam[];
  units: Placed[];
  objectives: string[];
}

export interface CatalogueMap { name: string; resourceId: number }

/** Every map Zero-K's content service knows about. */
export async function maps(): Promise<CatalogueMap[]> {
  if (!inTauri()) return [];
  return invoke<CatalogueMap[]>("sp_maps");
}

export interface Preview { install: { root: string; source: string }; exe: string; engine?: string }

/** Where Zero-K is, and which engine a scenario would run on. */
export async function launchPreview(engine = ""): Promise<Preview> {
  if (!inTauri()) throw new Error("Splaunch needs the desktop app.");
  return invoke<Preview>("sp_launch_preview", { engine });
}

/** The script this scenario would produce, without launching it. */
export async function scenarioScript(scenario: Scenario, player: string): Promise<string> {
  if (!inTauri()) return "";
  return invoke<string>("spsc_script", { scenario, player });
}

/** What is wrong with it, in sentences. */
export async function scenarioProblems(scenario: Scenario): Promise<string[]> {
  if (!inTauri()) return [];
  return invoke<string[]>("spsc_problems", { scenario });
}

/** Compile and launch the game into it. */
export async function testScenario(
  scenario: Scenario, player: string, engine: string,
): Promise<number> {
  if (!inTauri()) throw new Error("Testing a scenario needs the desktop app.");
  return invoke<number>("spsc_test", { scenario, player, engine });
}
