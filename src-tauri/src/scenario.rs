//! Splaunch: Zero-K scenarios, and the start script they compile to.
//!
//! `docs/DESIGN.md` has the research this is built on. The finding this module is built
//! on: **a Zero-K scenario's most portable form is a start script, not a file
//! format.** The engine reads `script.txt`; units, teams, AIs and modoptions are
//! all expressible there against unmodified Zero-K, with no archive to build and
//! no server to publish to.
//!
//! The consequence is that "Test" is not a preview. It writes a script and
//! launches the real game into it, so there is no second renderer to build and
//! no fidelity gap to apologise for.
//!
//! The writer escapes rather than refuses. A lobby has to reject a name
//! containing `;` or `}` outright, because a server-issued name with a
//! delimiter in it would silently produce a different script than intended. A
//! scenario name is the author's own, and losing their semicolon beats refusing
//! to launch - so delimiters are removed and everything else is kept.

use serde::{Deserialize, Serialize};

use crate::customkey::{self, Table, Value};
use crate::customkey as ck;

/// One unit placed on the map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placed {
    /// Zero-K's unit name, e.g. `armcom`. Not validated here - the engine is
    /// the authority on what exists, and guessing would go stale.
    pub unit: String,
    pub team: u32,
    /// Map position in elmos.
    pub x: f32,
    pub z: f32,
}

/// A team in the scenario. Team 0 is the player unless `ai` says otherwise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: u32,
    pub ally: u32,
    /// None for the human player; otherwise the AI's short name.
    pub ai: Option<String>,
    /// "1 0 0". Left to the caller so the editor and the game agree on colours.
    pub colour: String,
}

/// A wreck, rock or other feature placed on the map.
///
/// Zero-K resurrects a feature whose name ends in `_dead` back into the unit
/// it came from, so placing `armcom_dead` leaves a reclaimable, rebuildable
/// wreck rather than scenery. The gadget wires that up on its own.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub name: String,
    pub x: f32,
    pub z: f32,
    /// 0-3. Random if absent, which is what Zero-K does for scenery.
    #[serde(default)]
    pub facing: Option<u32>,
}

/// What an author is actually trying to say, rather than the fields it takes.
///
/// Zero-K's objectives are unit-count comparisons over time windows: 24 fields
/// whose useful combinations are not guessable from their names, and one of
/// which is spelled `comparisionType`. These seven cover the goals people
/// actually write, and each compiles to a combination read out of Zero-K's own
/// annotated reference - see docs/MISSION-MODEL.md section 4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Goal {
    /// Keep at least one of these alive to the deadline.
    SurviveUntil { seconds: u32, units: Vec<String> },
    /// Produce this many, counting ones that died on the way.
    BuildBy { unit: String, count: u32, seconds: u32 },
    /// Have this many at one moment. The satisfying set is frozen so
    /// overbuilding afterwards cannot pad it.
    HaveAtOnce { unit: String, count: u32 },
    /// None of the enemy's left by the deadline.
    DestroyAllBy { unit: String, seconds: u32 },
    /// Kill this many, however long it takes.
    KillCount { unit: String, count: u32 },
    /// Win the match before the clock runs out.
    WinBefore { seconds: u32 },
}

/// One objective: what the player is told, and what the game checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Objective {
    pub description: String,
    pub goal: Goal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub name: String,
    pub map: String,
    pub game: String,
    pub teams: Vec<Team>,
    pub units: Vec<Placed>,
    /// Free-text objectives, shown in the briefing alongside the checked ones.
    /// Kept because not every intention is a unit count, and a sentence beats
    /// contorting one into a comparison.
    pub objectives: Vec<String>,
    /// Objectives the game actually evaluates.
    #[serde(default)]
    pub goals: Vec<Objective>,
    #[serde(default)]
    pub features: Vec<Feature>,
    /// Shown before the match starts, in Zero-K's briefing window.
    #[serde(default)]
    pub briefing: Option<String>,
}

/// Anything a script value cannot contain.
///
/// Unlike `launch.rs`, this escapes rather than refuses: a scenario name is the
/// author's to choose, and losing their apostrophe is better than refusing to
/// launch. Delimiters are the exception - they change the script's shape, so
/// they are removed rather than represented.
fn escape(value: &str) -> String {
    value
        .chars()
        .filter(|c| !matches!(c, ';' | '{' | '}' | '\n' | '\r'))
        .collect()
}

fn key(out: &mut String, indent: &str, k: &str, v: impl std::fmt::Display) {
    out.push_str(&format!("{indent}{k}={v};\n"));
}

/// What is wrong with this scenario, in sentences a person can act on.
///
/// Returned rather than thrown so the editor can show a count before Test is
/// pressed: an invalid scenario should be visible while it is being made, not
/// after it fails to start.
pub fn problems(s: &Scenario) -> Vec<String> {
    let mut out = Vec::new();
    if s.map.trim().is_empty() {
        out.push("No map chosen.".into());
    }
    if s.teams.is_empty() {
        out.push("No teams.".into());
    }
    if !s.teams.iter().any(|t| t.ai.is_none()) {
        out.push("No player team - somebody has to be you.".into());
    }
    let allies: std::collections::HashSet<u32> = s.teams.iter().map(|t| t.ally).collect();
    if allies.len() < 2 && !s.teams.is_empty() {
        out.push("Every team is on the same side, so the game ends immediately.".into());
    }
    for u in &s.units {
        if !s.teams.iter().any(|t| t.id == u.team) {
            out.push(format!("A {} belongs to team {}, which does not exist.", u.unit, u.team));
            break;
        }
    }
    if s.units.is_empty() {
        out.push("Nothing placed yet.".into());
    }
    out
}

/// Zero-K's comparison constants, from `mission_galaxy_campaign_battle.lua`.
const AT_LEAST: f64 = 1.0;
const AT_MOST: f64 = 2.0;

/// A list of unit names as the Lua array Zero-K expects.
fn unit_list(names: &[String]) -> Value {
    let mut list = Table::new();
    for name in names {
        list.push(ck::s(name));
    }
    ck::t(list)
}

/// One objective, as the field combination Zero-K evaluates.
///
/// The mapping is not invented: each combination is taken from the worked
/// examples in Zero-K's own `sample_planet.lua`, which is the only place the
/// interactions between `satisfy*`, `countRemovedUnits` and `lockUnitsOnSatisfy`
/// are written down.
fn goal_fields(objective: &Objective) -> Table {
    let mut table = Table::new();
    table.set("description", ck::s(&objective.description));

    match &objective.goal {
        Goal::WinBefore { seconds } => {
            // The one objective that is not a unit count.
            table.set("victoryByTime", ck::n(*seconds));
        }
        Goal::SurviveUntil { seconds, units } => {
            table.set("satisfyUntilTime", ck::n(*seconds));
            table.set("comparisionType", ck::n(AT_LEAST));
            table.set("targetNumber", ck::n(1));
            if !units.is_empty() {
                table.set("unitTypes", unit_list(units));
            }
        }
        Goal::BuildBy { unit, count, seconds } => {
            table.set("satisfyByTime", ck::n(*seconds));
            // Units that died on the way still count, or "build 5" would mean
            // "have 5 simultaneously" and fail for an unrelated reason.
            table.set("countRemovedUnits", ck::b(true));
            table.set("comparisionType", ck::n(AT_LEAST));
            table.set("targetNumber", ck::n(*count));
            table.set("unitTypes", unit_list(std::slice::from_ref(unit)));
        }
        Goal::HaveAtOnce { unit, count } => {
            table.set("satisfyOnce", ck::b(true));
            // Freeze the satisfying set, so building more afterwards cannot be
            // used to paper over losses.
            table.set("lockUnitsOnSatisfy", ck::b(true));
            table.set("comparisionType", ck::n(AT_LEAST));
            table.set("targetNumber", ck::n(*count));
            table.set("unitTypes", unit_list(std::slice::from_ref(unit)));
        }
        Goal::DestroyAllBy { unit, seconds } => {
            table.set("satisfyByTime", ck::n(*seconds));
            table.set("comparisionType", ck::n(AT_MOST));
            table.set("targetNumber", ck::n(0));
            table.set("enemyUnitTypes", unit_list(std::slice::from_ref(unit)));
        }
        Goal::KillCount { unit, count } => {
            table.set("satisfyOnce", ck::b(true));
            // Only the dead count, which is what makes this "kill" rather than
            // "have".
            table.set("onlyCountRemovedUnits", ck::b(true));
            table.set("comparisionType", ck::n(AT_LEAST));
            table.set("targetNumber", ck::n(*count));
            table.set("enemyUnitTypes", unit_list(std::slice::from_ref(unit)));
        }
    }
    table
}

/// The mission modoptions for a scenario, as `(key, encoded value)` pairs.
///
/// Zero-K's mission engine lives in the base game and is armed by a single
/// modoption, so a scenario needs no archive - see docs/MISSION-MODEL.md.
pub fn mission_modoptions(s: &Scenario) -> Vec<(String, String)> {
    let mut out = Vec::new();

    // Arms the mission engine. The value is only an identifier; the campaign
    // reports results against it, and nothing is listening for ours.
    out.push(("singleplayercampaignbattleid".into(), "splaunch".into()));

    if !s.goals.is_empty() {
        let mut list = Table::new();
        for objective in &s.goals {
            list.push(ck::t(goal_fields(objective)));
        }
        // Both keys: one drives the panel the player reads, the other the
        // evaluation. The gadget only reads the bonus list.
        let encoded = customkey::encode(&list);
        out.push(("bonusobjectiveconfig".into(), encoded.clone()));
        out.push(("objectiveconfig".into(), encoded));
    }

    if !s.features.is_empty() {
        let mut list = Table::new();
        for feature in &s.features {
            let mut entry = Table::new();
            entry.set("name", ck::s(&feature.name));
            entry.set("x", ck::n(feature.x as f64));
            entry.set("z", ck::n(feature.z as f64));
            if let Some(facing) = feature.facing {
                entry.set("facing", ck::n(facing));
            }
            list.push(ck::t(entry));
        }
        out.push(("featurestospawn".into(), customkey::encode(&list)));
    }

    // The briefing window titles itself from `name` and needs a description, so
    // it is only worth sending when there is something to read.
    if let Some(text) = s.briefing.as_deref().filter(|t| !t.trim().is_empty()) {
        let mut info = Table::new();
        info.set("name", ck::s(&s.name));
        info.set("description", ck::s(text));
        out.push(("planetmissioninformationtext".into(), customkey::encode(&info)));
    }

    out
}

/// A team's placed units, as the custom keys Zero-K reads them from.
///
/// Chunked forty to a key because that is what Zero-K's own script builder
/// does - a start script value has a length limit, and forty is the number the
/// campaign settled on.
const START_UNITS_BLOCK: usize = 40;

fn start_unit_keys(s: &Scenario, team: u32) -> Vec<(String, String)> {
    let mine: Vec<&Placed> = s.units.iter().filter(|u| u.team == team).collect();
    let mut out = Vec::new();
    for (block, chunk) in mine.chunks(START_UNITS_BLOCK).enumerate() {
        let mut list = Table::new();
        for unit in chunk {
            let mut entry = Table::new();
            entry.set("name", ck::s(&unit.unit));
            entry.set("x", ck::n(unit.x as f64));
            entry.set("z", ck::n(unit.z as f64));
            list.push(ck::t(entry));
        }
        out.push((format!("extrastartunits_{}", block + 1), customkey::encode(&list)));
    }
    out
}

/// Compile to a Spring start script.
///
/// The shape is taken from a real one: `_missionScript.txt` inside Zero-K's own
/// `User Interface Tutorial r22.sdz`, which is what the old mission editor
/// emitted and what the engine still reads.
pub fn write_script(s: &Scenario, player: &str) -> Result<String, String> {
    if let Some(first) = problems(s).first() {
        return Err(first.clone());
    }

    let mut out = String::new();
    out.push_str("[GAME]\n{\n");
    key(&mut out, "\t", "Mapname", escape(&s.map));
    key(&mut out, "\t", "GameType", escape(&s.game));
    key(&mut out, "\t", "MyPlayerName", escape(player));
    // Local, hosted by us, nobody to wait for.
    key(&mut out, "\t", "IsHost", 1);
    key(&mut out, "\t", "OnlyLocal", 1);
    key(&mut out, "\t", "StartPosType", 2);
    key(&mut out, "\t", "GameStartDelay", 0);
    key(&mut out, "\t", "NumRestrictions", 0);

    out.push_str("\t[MODOPTIONS]\n\t{\n");
    // Nothing a scenario does should count towards anybody's rating.
    key(&mut out, "\t\t", "noelo", 1);
    // The mission engine, its objectives, features and briefing. Not escaped:
    // these are base64 of our own making and contain no delimiter, and running
    // them through `escape` could only corrupt them.
    for (name, value) in mission_modoptions(s) {
        key(&mut out, "\t\t", &name, value);
    }
    out.push_str("\t}\n");

    // The human. One player, always index 0, on the first non-AI team.
    let human = s.teams.iter().find(|t| t.ai.is_none()).map(|t| t.id).unwrap_or(0);
    out.push_str("\t[PLAYER0]\n\t{\n");
    key(&mut out, "\t\t", "Name", escape(player));
    key(&mut out, "\t\t", "Team", human);
    out.push_str("\t}\n");

    for (i, t) in s.teams.iter().filter(|t| t.ai.is_some()).enumerate() {
        out.push_str(&format!("\t[AI{i}]\n\t{{\n"));
        key(&mut out, "\t\t", "Name", format!("AI {}", t.id));
        key(&mut out, "\t\t", "ShortName", escape(t.ai.as_deref().unwrap_or("NullAI")));
        key(&mut out, "\t\t", "Team", t.id);
        key(&mut out, "\t\t", "Host", 0);
        out.push_str("\t}\n");
    }

    for t in &s.teams {
        out.push_str(&format!("\t[TEAM{}]\n\t{{\n", t.id));
        key(&mut out, "\t\t", "TeamLeader", 0);
        key(&mut out, "\t\t", "AllyTeam", t.ally);
        key(&mut out, "\t\t", "RGBColor", escape(&t.colour));
        // Placed units ride on the team that owns them, which is how Zero-K
        // knows whose they are without a field saying so.
        for (name, value) in start_unit_keys(s, t.id) {
            key(&mut out, "\t\t", &name, value);
        }
        out.push_str("\t}\n");
    }

    let mut allies: Vec<u32> = s.teams.iter().map(|t| t.ally).collect();
    allies.sort_unstable();
    allies.dedup();
    for a in allies {
        out.push_str(&format!("\t[ALLYTEAM{a}]\n\t{{\n"));
        key(&mut out, "\t\t", "NumAllies", 0);
        out.push_str("\t}\n");
    }

    out.push_str("}\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Scenario {
        Scenario {
            name: "Test".into(),
            map: "Comet Catcher Redux".into(),
            game: "Zero-K v1.14.8.0".into(),
            teams: vec![
                Team { id: 0, ally: 0, ai: None, colour: "0 0 1".into() },
                Team { id: 1, ally: 1, ai: Some("NullAI".into()), colour: "1 0 0".into() },
            ],
            units: vec![Placed { unit: "armcom".into(), team: 0, x: 512.0, z: 512.0 }],
            objectives: vec!["Destroy the enemy commander".into()],
            goals: vec![],
            features: vec![],
            briefing: None,
        }
    }


    use crate::customkey::decode_as_the_game_does;

    /// The value of `key=` inside a script section, unterminated semicolon and
    /// surrounding whitespace removed.
    fn value_of(section: &str, name: &str) -> Option<String> {
        section.lines().map(str::trim).find_map(|line| {
            let (k, v) = line.split_once('=')?;
            (k.trim() == name).then(|| v.trim_end_matches(';').to_string())
        })
    }

    /// A modoption, decoded the way Zero-K will decode it.
    fn modoption_lua(script: &str, name: &str) -> String {
        let block = script
            .split("[MODOPTIONS]")
            .nth(1)
            .expect("no [MODOPTIONS] section");
        let raw = value_of(block, name).unwrap_or_else(|| panic!("no {name} modoption"));
        String::from_utf8(decode_as_the_game_does(&raw))
            .unwrap_or_else(|e| panic!("{name} did not survive Zero-K's decoder: {e}"))
    }

    fn with_goals(goals: Vec<Objective>) -> Scenario {
        let mut sc = sample();
        sc.goals = goals;
        sc
    }

    #[test]
    fn the_mission_engine_is_armed() {
        // Without this one modoption the whole objective system stays asleep,
        // and a scenario with objectives would launch looking like a skirmish.
        let script = write_script(&sample(), "Qrow").unwrap();
        let block = script.split("[MODOPTIONS]").nth(1).unwrap();
        assert_eq!(value_of(block, "singleplayercampaignbattleid").as_deref(), Some("splaunch"));
    }

    #[test]
    fn a_survival_goal_compiles_to_the_fields_zero_k_checks() {
        let sc = with_goals(vec![Objective {
            description: "Hold out for two minutes.".into(),
            goal: Goal::SurviveUntil { seconds: 120, units: vec!["armcom".into()] },
        }]);
        let lua = modoption_lua(&write_script(&sc, "Qrow").unwrap(), "bonusobjectiveconfig");
        assert!(lua.contains("satisfyUntilTime=120"), "{lua}");
        assert!(lua.contains("comparisionType=1"), "{lua}");
        assert!(lua.contains("targetNumber=1"), "{lua}");
        assert!(lua.contains("armcom"), "{lua}");
    }

    #[test]
    fn build_counts_the_dead_and_have_does_not() {
        // The difference between "build 5" and "have 5" is one flag, and
        // getting it wrong makes an objective that quietly cannot be completed.
        let build = with_goals(vec![Objective {
            description: "Build five Glaives.".into(),
            goal: Goal::BuildBy { unit: "armpw".into(), count: 5, seconds: 300 },
        }]);
        let lua = modoption_lua(&write_script(&build, "Qrow").unwrap(), "bonusobjectiveconfig");
        assert!(lua.contains("countRemovedUnits=true"), "{lua}");

        let have = with_goals(vec![Objective {
            description: "Have five Glaives.".into(),
            goal: Goal::HaveAtOnce { unit: "armpw".into(), count: 5 },
        }]);
        let lua = modoption_lua(&write_script(&have, "Qrow").unwrap(), "bonusobjectiveconfig");
        assert!(!lua.contains("countRemovedUnits"), "{lua}");
        assert!(lua.contains("lockUnitsOnSatisfy=true"), "{lua}");
    }

    #[test]
    fn killing_uses_the_enemys_units_not_ours() {
        let sc = with_goals(vec![Objective {
            description: "Kill three Reavers.".into(),
            goal: Goal::KillCount { unit: "armwar".into(), count: 3 },
        }]);
        let lua = modoption_lua(&write_script(&sc, "Qrow").unwrap(), "bonusobjectiveconfig");
        assert!(lua.contains("enemyUnitTypes"), "{lua}");
        assert!(!lua.contains("unitTypes={"), "counted our own units: {lua}");
        assert!(lua.contains("onlyCountRemovedUnits=true"), "{lua}");
    }

    #[test]
    fn a_question_mark_in_a_description_does_not_destroy_the_objectives() {
        /* Zero-K's decoder turns an unescaped '?' at the wrong offset into
           end-of-data, which loses every objective at once rather than just
           that one - see docs/MISSION-MODEL.md section 3. The author should be
           able to ask a question. */
        for text in [
            "Can you hold the ridge?",
            "Ready? Set? Go?",
            "Halte den Grat fünf Minuten",
            "Продержись 5 минут",
        ] {
            let sc = with_goals(vec![Objective {
                description: text.into(),
                goal: Goal::WinBefore { seconds: 60 },
            }]);
            let lua = modoption_lua(&write_script(&sc, "Qrow").unwrap(), "bonusobjectiveconfig");
            assert!(lua.contains("victoryByTime=60"), "lost objectives for {text:?}: {lua}");
        }
    }

    #[test]
    fn features_reach_the_script() {
        let mut sc = sample();
        sc.features = vec![Feature { name: "armcom_dead".into(), x: 100.0, z: 200.0, facing: Some(1) }];
        let lua = modoption_lua(&write_script(&sc, "Qrow").unwrap(), "featurestospawn");
        assert!(lua.contains("armcom_dead"), "{lua}");
        assert!(lua.contains("x=100") && lua.contains("z=200"), "{lua}");
    }

    #[test]
    fn a_briefing_is_only_sent_when_there_is_one() {
        let script = write_script(&sample(), "Qrow").unwrap();
        assert!(!script.contains("planetmissioninformationtext"));

        let mut sc = sample();
        sc.briefing = Some("The dam will not hold.".into());
        let lua = modoption_lua(&write_script(&sc, "Qrow").unwrap(), "planetmissioninformationtext");
        assert!(lua.contains("The dam will not hold."), "{lua}");
    }

    #[test]
    fn many_units_are_chunked_the_way_zero_k_chunks_them() {
        // Forty to a key, because that is what Zero-K's own script builder does
        // and a start script value has a length limit.
        let mut sc = sample();
        sc.units = (0..85)
            .map(|i| Placed { unit: "armpw".into(), team: 0, x: i as f32, z: 0.0 })
            .collect();
        let script = write_script(&sc, "Qrow").unwrap();
        let team0 = script.split("[TEAM0]").nth(1).unwrap().split("[TEAM1]").next().unwrap();
        assert!(value_of(team0, "extrastartunits_1").is_some());
        assert!(value_of(team0, "extrastartunits_2").is_some());
        assert!(value_of(team0, "extrastartunits_3").is_some());
        assert!(value_of(team0, "extrastartunits_4").is_none());
    }

    #[test]
    fn a_scenario_compiles_to_a_script_the_engine_shape_matches() {
        let s = write_script(&sample(), "Qrow").unwrap();
        assert!(s.starts_with("[GAME]\n{\n"));
        assert!(s.contains("Mapname=Comet Catcher Redux;"));
        assert!(s.contains("OnlyLocal=1;"));
        assert!(s.contains("[PLAYER0]"));
        assert!(s.contains("[AI0]"));
        assert!(s.contains("ShortName=NullAI;"));
        assert!(s.contains("[TEAM0]") && s.contains("[TEAM1]"));
        assert!(s.contains("[ALLYTEAM0]") && s.contains("[ALLYTEAM1]"));
        assert!(s.trim_end().ends_with('}'));
    }

    #[test]
    fn braces_are_balanced() {
        // The engine's parser is not forgiving, and an unbalanced script fails
        // with a message about the wrong line.
        let s = write_script(&sample(), "Qrow").unwrap();
        assert_eq!(s.matches('{').count(), s.matches('}').count());
    }

    #[test]
    fn a_name_that_would_break_the_script_is_escaped_not_refused() {
        /* The join path refuses these, because a server-issued name never
           contains one. A scenario author's name is their own, and losing a
           semicolon beats refusing to launch. */
        let mut sc = sample();
        sc.map = "Weird; }Map{".into();
        let s = write_script(&sc, "Qrow").unwrap();
        assert!(s.contains("Mapname=Weird Map;"));
        assert_eq!(s.matches('{').count(), s.matches('}').count());
    }

    #[test]
    fn a_scenario_with_no_player_does_not_compile() {
        let mut sc = sample();
        sc.teams[0].ai = Some("NullAI".into());
        let err = write_script(&sc, "Qrow").unwrap_err();
        assert!(err.contains("player team"), "{err}");
    }

    #[test]
    fn problems_are_sentences_rather_than_codes() {
        let empty = Scenario {
            name: "".into(), map: "".into(), game: "".into(),
            teams: vec![], units: vec![], objectives: vec![],
            goals: vec![], features: vec![], briefing: None,
        };
        let p = problems(&empty);
        assert!(p.len() >= 3);
        for line in &p {
            assert!(line.ends_with('.'), "{line:?} is not a sentence");
        }
    }

    #[test]
    fn one_sided_scenarios_are_caught_before_launch() {
        // Two teams on the same allyteam ends the moment it starts, which is a
        // confusing way to find out you made a mistake.
        let mut sc = sample();
        sc.teams[1].ally = 0;
        assert!(problems(&sc).iter().any(|p| p.contains("same side")));
    }

    #[test]
    fn a_unit_on_a_team_that_does_not_exist_is_caught() {
        let mut sc = sample();
        sc.units[0].team = 7;
        assert!(problems(&sc).iter().any(|p| p.contains("does not exist")));
    }

    /// Zero-K's own mission script, lifted out of
    /// `games/User Interface Tutorial r22.sdz`. This is what the engine is
    /// known to accept, so it is the thing to be measured against.
    const REAL: &str = include_str!("fixtures/mission-script.txt");

    /// Every `[SECTION]` name, in order.
    fn sections(script: &str) -> Vec<String> {
        script
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with('[') && l.ends_with(']'))
            .map(|l| l.trim_matches(['[', ']']).to_string())
            .collect()
    }

    /// Every `Key=` at any depth, lowercased.
    fn keys(script: &str) -> std::collections::HashSet<String> {
        script
            .lines()
            .filter_map(|l| l.trim().split_once('='))
            .map(|(k, _)| k.trim().to_ascii_lowercase())
            .collect()
    }

    #[test]
    fn our_script_has_the_sections_the_engine_expects() {
        /* The single biggest unknown in docs/SCENARIO-EDITOR.md is whether a
           script we write actually launches. Nothing here launches anything -
           that still wants doing by hand - but a script missing a section the
           engine's own one has would fail for a reason we can find now rather
           than at the whistle. */
        let ours = write_script(&sample(), "Qrow").unwrap();
        let theirs = sections(REAL);
        let mine = sections(&ours);

        for want in ["GAME", "MODOPTIONS", "PLAYER0", "AI0", "TEAM0", "TEAM1", "ALLYTEAM0"] {
            assert!(theirs.iter().any(|s| s == want), "the real script has no [{want}]");
            assert!(mine.iter().any(|s| s == want), "ours has no [{want}]");
        }
    }

    #[test]
    fn our_script_sets_the_keys_the_engine_reads() {
        let ours = write_script(&sample(), "Qrow").unwrap();
        let theirs = keys(REAL);
        let mine = keys(&ours);

        /* Not every key - the real one carries mission-specific extras we have
           no business emitting. These are the ones that decide whether a local
           game starts at all, and every one of them is in theirs too. */
        for want in ["mapname", "gametype", "myplayername", "ishost", "onlylocal",
                     "gamestartdelay", "name", "team", "shortname", "allyteam"] {
            assert!(theirs.contains(want), "the real script does not set {want}");
            assert!(mine.contains(want), "ours does not set {want}");
        }
    }

    #[test]
    fn our_script_parses_the_way_theirs_does() {
        /* Balanced braces, and every value terminated by a `;` before its
           section closes.

           Deliberately not a per-line rule: the real script puts four pairs on
           one line and closes the section on the same one -
           `StartRectTop=0;		StartRectBottom=0; ... }` - which is the engine
           telling us that newlines are not part of its grammar at all. Ours is
           formatted for a human to read, and that is free.

           A `=` inside a value is legal and has to stay legal: Zero-K's own
           custom keys are base64, and base64 pads with `=`. The engine splits
           an assignment at its first `=` and reads to the `;`, so that is what
           is checked here. This test used to treat a second `=` as a missing
           terminator, which was a stricter rule than the engine's and would
           have rejected every mission payload we now emit. */
        let ours = write_script(&sample(), "Qrow").unwrap();
        for script in [REAL, ours.as_str()] {
            assert_eq!(script.matches('{').count(), script.matches('}').count());
            let bytes = script.as_bytes();
            let mut at = 0;
            while let Some(offset) = script[at..].find('=') {
                let i = at + offset;
                let rest = &bytes[i + 1..];
                let end = rest
                    .iter()
                    .position(|c| matches!(c, b';' | b'}'))
                    .expect("a value with no terminator");
                assert_eq!(
                    rest[end], b';',
                    "unterminated value at {:?}",
                    &script[i.saturating_sub(24)..(i + 8).min(script.len())]
                );
                // Past this whole assignment, so padding inside the value is
                // not mistaken for the start of another one.
                at = i + 1 + end;
            }
        }
    }

    #[test]
    fn placed_units_travel_on_their_team() {
        // They used to be written to a side-car file that nothing read. Now
        // they are a team custom key, which is where Zero-K looks for them.
        let script = write_script(&sample(), "Qrow").unwrap();
        let team0 = script
            .split("[TEAM0]")
            .nth(1)
            .and_then(|s| s.split("[TEAM1]").next())
            .expect("no [TEAM0] section");
        let value = value_of(team0, "extrastartunits_1").expect("no units on team 0");
        let lua = String::from_utf8(decode_as_the_game_does(&value)).unwrap();
        assert!(lua.contains("armcom"), "{lua}");
    }
}

// --------------------------------------------------------------- commands ---

/// Where a scenario's script is written before launching.
fn scenario_paths() -> std::path::PathBuf {
    std::env::temp_dir().join("shiro").join("scenario_script.txt")
}

/// Compile without launching, so the editor can show the script.
#[tauri::command]
pub fn spsc_script(scenario: Scenario, player: String) -> Result<String, String> {
    write_script(&scenario, &player)
}

/// What is wrong with it, for the count in the header.
#[tauri::command]
pub fn spsc_problems(scenario: Scenario) -> Vec<String> {
    problems(&scenario)
}

/// Compile and launch the real game into it.
#[tauri::command]
pub fn spsc_test(
    app: tauri::AppHandle,
    game: tauri::State<'_, crate::launch::Game>,
    scenario: Scenario,
    player: String,
    engine: String,
) -> Result<u32, String> {
    let script = write_script(&scenario, &player)?;
    let script_path = scenario_paths();
    if let Some(dir) = script_path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    std::fs::write(&script_path, script)
        .map_err(|e| format!("could not write the script: {e}"))?;
    crate::launch::launch_script(app, game, &script_path, &engine)
}

