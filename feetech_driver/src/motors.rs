use eyre::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct MotorConfig {
    pub joint_name: String,
    pub motor_type: String,
    pub id: u8,
}

#[derive(Debug, Deserialize)]
struct MotorConfigPartial {
    joint_name: Option<String>,
    motor_type: String,
    id: u8,
}

pub fn load_motors(path: &PathBuf) -> Result<Vec<MotorConfig>> {
    let json = std::fs::read_to_string(path)
        .wrap_err_with(|| format!("failed to read motor config at {}", path.display()))?;

    let motors: Vec<MotorConfig> = if json.trim_start().starts_with('[') {
        let vec: Vec<MotorConfigPartial> = serde_json::from_str(&json)
            .wrap_err_with(|| format!("failed to parse motor config at {}", path.display()))?;
        vec.into_iter()
            .enumerate()
            .map(|(idx, m)| MotorConfig {
                joint_name: m.joint_name.unwrap_or_else(|| format!("joint_{idx}")),
                motor_type: m.motor_type,
                id: m.id,
            })
            .collect()
    } else {
        let map: HashMap<String, MotorConfigPartial> = serde_json::from_str(&json)
            .wrap_err_with(|| format!("failed to parse motor config at {}", path.display()))?;
        map.into_iter()
            .map(|(key, m)| MotorConfig {
                joint_name: m.joint_name.unwrap_or(key),
                motor_type: m.motor_type,
                id: m.id,
            })
            .collect()
    };

    Ok(motors)
}
