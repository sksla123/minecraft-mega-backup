use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub minecraft: MinecraftConfig,
    pub mega: MegaConfig,
    pub shoutrrr: ShoutrrrConfig,
    pub rcon: Option<RconConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DaemonConfig {
    pub interval: String,
    pub startup_retry: Option<u32>,
    pub startup_retry_interval: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MinecraftConfig {
    pub server_dir: String,
    pub backup_temp_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MegaConfig {
    pub remote_dir: String,
    pub reserve_gb: u64,
    pub keep_versions: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ShoutrrrConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RconConfig {
    pub enable: bool,
    pub address: String,
    pub password: String,
}
