//! Starting the engine on a scenario.
//!
//! Splaunch does not join anything, so this is much smaller than the lobby's
//! version it came from: find the install, find the engine, run it on a script
//! we wrote, and come back when it exits.
//!
//! One game at a time. Zero-K is single-instance, and two engines fighting over
//! the same write directory corrupts the config.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::install::{self, Install};

/// Where the engine's comings and goings are announced.
const GAME_EVENT: &str = "splaunch://game";

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GameStatus {
    Launched { pid: u32 },
    Exited { code: Option<i32> },
    Failed { reason: String },
}

pub struct SpawnPlan {
    pub exe: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
}

/// How the engine is invoked: its own config if there is one, then the script.
pub fn spawn_plan(exe: &Path, root: &Path, script: &Path) -> SpawnPlan {
    let mut args: Vec<OsString> = Vec::new();
    let config = root.join("springsettings.cfg");
    if config.is_file() {
        args.push("--config".into());
        args.push(config.into_os_string());
    }
    args.push(script.as_os_str().to_os_string());
    SpawnPlan { exe: exe.to_path_buf(), args, cwd: root.to_path_buf() }
}

/// One game at a time, and where Zero-K lives if it is somewhere unusual.
#[derive(Default)]
pub struct Game {
    running: Arc<Mutex<bool>>,
    root: Arc<Mutex<Option<String>>>,
}

/// What a launch would do, resolved but not run.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPreview {
    pub install: Install,
    pub exe: String,
}

/// Find Zero-K, optionally somewhere the user pointed us.
#[tauri::command]
pub fn sp_locate_install(game: State<'_, Game>, root: Option<String>) -> Result<Install, String> {
    {
        let mut r = game.root.lock().map_err(|_| "install state poisoned".to_string())?;
        *r = root.clone().filter(|s| !s.trim().is_empty());
    }
    install::detect_with(root.as_deref())
}

/// Answer "would this work, and with what" without starting anything.
#[tauri::command]
pub fn sp_launch_preview(game: State<'_, Game>, engine: String) -> Result<LaunchPreview, String> {
    let root = game.root.lock().ok().and_then(|r| r.clone());
    let install = install::detect_with(root.as_deref())?;
    let exe = install::find_engine(&install.root, &engine)?;
    Ok(LaunchPreview { exe: exe.display().to_string(), install })
}

/// Start the engine on a script somebody else wrote.
///
/// `running` is cleared however it ends, and the exit is announced, so the
/// editor comes back by itself when the match is over.
pub fn launch_script(
    app: AppHandle,
    game: State<'_, Game>,
    script: &Path,
    engine: &str,
) -> Result<u32, String> {
    {
        let mut running = game
            .running
            .lock()
            .map_err(|_| "game state poisoned".to_string())?;
        if *running {
            return Err("A game is already running.".into());
        }
        *running = true;
    }

    let root = game.root.lock().ok().and_then(|r| r.clone());
    let spawned = (|| {
        let install = install::detect_with(root.as_deref())?;
        let exe = install::find_engine(&install.root, engine)?;
        let plan = spawn_plan(&exe, &install.root, script);
        std::process::Command::new(&plan.exe)
            .args(&plan.args)
            .current_dir(&plan.cwd)
            .spawn()
            .map_err(|e| format!("could not start the engine: {e}"))
    })();

    let mut child = match spawned {
        Ok(child) => child,
        Err(e) => {
            if let Ok(mut r) = game.running.lock() {
                *r = false;
            }
            let _ = app.emit(GAME_EVENT, GameStatus::Failed { reason: e.clone() });
            return Err(e);
        }
    };

    let pid = child.id();
    let _ = app.emit(GAME_EVENT, GameStatus::Launched { pid });

    let running = game.running.clone();
    std::thread::spawn(move || {
        let code = child.wait().ok().and_then(|s| s.code());
        if let Ok(mut r) = running.lock() {
            *r = false;
        }
        let _ = app.emit(GAME_EVENT, GameStatus::Exited { code });
    });

    Ok(pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_gets_the_config_when_there_is_one() {
        // The engine reads springsettings.cfg from the data directory, and a
        // scenario should look the way the player's game looks.
        let dir = std::env::temp_dir().join("splaunch-test-plan");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("springsettings.cfg"), "x=1").unwrap();

        // --config, the path to it, then the script - in that order, because
        // the engine takes the script as its positional argument.
        let plan = spawn_plan(Path::new("spring.exe"), &dir, Path::new("script.txt"));
        assert_eq!(plan.args.len(), 3);
        assert_eq!(plan.args[0], OsString::from("--config"));
        assert_eq!(plan.args[2], OsString::from("script.txt"));
        assert_eq!(plan.cwd, dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn without_a_config_it_is_just_the_script() {
        let dir = std::env::temp_dir().join("splaunch-test-plan-bare");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let plan = spawn_plan(Path::new("spring.exe"), &dir, Path::new("script.txt"));
        assert_eq!(plan.args, vec![OsString::from("script.txt")]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
