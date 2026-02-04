use std::{env, path::PathBuf};

#[derive(Debug)]
pub struct Settings {
    pub config_path: PathBuf,
    pub port: String,
    pub baudrate: u32,
    pub timeout_ms: u64,
}

pub fn load_settings() -> Settings {
    let default_config: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../config/motor.json");

    let config_path = env::var("CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or(default_config);

    let port = env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/ttyACM0".to_string());
    let baudrate = env::var("BAUDRATE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let timeout_ms = env::var("SERIAL_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000);

    Settings {
        config_path,
        port,
        baudrate,
        timeout_ms,
    }
}
