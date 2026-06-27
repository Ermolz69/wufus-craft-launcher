use std::path::PathBuf;

pub struct PathResolver {
    pub app_data: PathBuf,
    pub cache: PathBuf,
    pub temp: PathBuf,
    pub logs: PathBuf,
    pub game: PathBuf,
}

impl PathResolver {
    pub fn new(app_data_path: PathBuf, game_path: String) -> Self {
        Self {
            cache: app_data_path.join("cache"),
            temp: app_data_path.join("temp"),
            logs: app_data_path.join("logs"),
            app_data: app_data_path,
            game: PathBuf::from(game_path),
        }
    }
}
