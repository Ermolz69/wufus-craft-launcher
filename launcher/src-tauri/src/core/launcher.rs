use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::core::settings::LauncherSettings;

/// The result of a pre-launch readiness check.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchReadiness {
    Ready,
    NeedsUpdate,
    NeedsRepair,
}

/// Payload returned by `prepare_launch` to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct BuildReadiness {
    pub status: LaunchReadiness,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
    pub reason: Option<String>,
}

/// Per-build launch configuration stored at `{game_dir}/.wufus/launch.json`.
///
/// Managed by the updater (downloaded alongside mods/configs). The manifest
/// server generates the correct values for each build.
///
/// Template variables substituted in `jvm_args` and `game_args`:
/// - `${game_dir}`      — absolute path to the game directory
/// - `${assets_dir}`   — `{game_dir}/assets`
/// - `${natives_dir}`  — `{game_dir}/natives`
/// - `${username}`     — offline player name ("WufusCraft_Player")
/// - `${version}`      — `version_name` from this config
/// - `${access_token}` — offline token literal ("offline_token")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Fully-qualified main class, e.g. "net.fabricmc.loader.impl.launch.knot.KnotClient".
    pub main_class: String,
    /// Classpath entries as paths relative to `game_dir`, e.g. "bin/client.jar".
    pub classpath: Vec<String>,
    /// Extra JVM arguments inserted before `-cp`. Template variables are substituted.
    pub jvm_args: Vec<String>,
    /// Arguments passed to the main class. Template variables are substituted.
    pub game_args: Vec<String>,
    /// Version label passed to `--version`, e.g. "WufusCraft-1.21.1".
    pub version_name: String,
}

impl LaunchConfig {
    pub const FILENAME: &'static str = "launch.json";

    pub fn load(game_dir: &Path) -> Result<Self, String> {
        let path = game_dir.join(".wufus").join(Self::FILENAME);
        let json = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        serde_json::from_str(&json).map_err(|e| format!("Cannot parse launch.json: {e}"))
    }
}

/// Fully resolved parameters ready to be turned into a `Command`.
#[derive(Debug)]
pub struct GameLaunchParams {
    pub java_exe: String,
    pub game_dir: PathBuf,
    pub main_class: String,
    /// Max heap in MB (= `ram_gb × 1024`).
    pub ram_mb: u64,
    pub classpath: Vec<PathBuf>,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}

impl GameLaunchParams {
    pub fn build(
        java_exe: &str,
        game_dir: &Path,
        config: &LaunchConfig,
        settings: &LauncherSettings,
    ) -> Self {
        let ram_mb = u64::from(settings.ram_gb) * 1024;

        let game_dir_str = game_dir.to_string_lossy().into_owned();
        let assets_dir = game_dir.join("assets").to_string_lossy().into_owned();
        let natives_dir = game_dir.join("natives").to_string_lossy().into_owned();

        let sub = |s: &str| -> String {
            s.replace("${game_dir}", &game_dir_str)
                .replace("${assets_dir}", &assets_dir)
                .replace("${natives_dir}", &natives_dir)
                .replace("${username}", "WufusCraft_Player")
                .replace("${version}", &config.version_name)
                .replace("${access_token}", "offline_token")
        };

        Self {
            java_exe: java_exe.to_string(),
            game_dir: game_dir.to_owned(),
            main_class: config.main_class.clone(),
            ram_mb,
            classpath: config.classpath.iter().map(|p| game_dir.join(p)).collect(),
            jvm_args: config.jvm_args.iter().map(|s| sub(s)).collect(),
            game_args: config.game_args.iter().map(|s| sub(s)).collect(),
        }
    }

    /// Assemble the OS `Command` (not yet spawned).
    pub fn into_command(self) -> Command {
        let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
        let cp = self
            .classpath
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(sep);

        let mut cmd = Command::new(&self.java_exe);
        cmd.current_dir(&self.game_dir);
        cmd.arg(format!("-Xmx{}M", self.ram_mb));
        // Xms capped at 512 MB so startup is not slow on large heap configs.
        cmd.arg(format!("-Xms{}M", self.ram_mb.min(512)));
        cmd.args(&self.jvm_args);
        cmd.args(["-cp", &cp]);
        cmd.arg(&self.main_class);
        cmd.args(&self.game_args);
        cmd
    }
}
