use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileCategory {
    Mod,
    Config,
    ResourcePack,
    Shader,
    Library,
    World,
    Screenshot,
    MapData,
    LauncherMeta,
    Asset,
    Other,
}
