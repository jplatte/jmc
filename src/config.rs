use std::{fs, path::PathBuf};

use matrix_sdk::Session;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<Session>,
}

static CONFIG_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| dirs::data_dir().unwrap().join("jmc"));
static CONFIG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| CONFIG_DIR_PATH.join("config.json"));

pub fn load() -> anyhow::Result<Config> {
    match fs::read(&*CONFIG_FILE_PATH) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save(config: &Config) -> anyhow::Result<()> {
    fs::create_dir_all(&*CONFIG_DIR_PATH)?;
    let file = fs::File::create(&*CONFIG_FILE_PATH)?;
    serde_json::to_writer(file, config)?;

    Ok(())
}
