use tracing::info;

use crate::{config::Settings, motors::MotorConfig};

pub fn log_settings(settings: &Settings) {
    info!("──────── Node Configuration ────────");
    info!(" Config file : {}", settings.config_path.display());
    info!(" Serial port : {}", settings.port);
    info!(" Baudrate    : {}", settings.baudrate);
    info!(" Timeout     : {} ms", settings.timeout_ms);
}

pub fn log_motors(motors: &[MotorConfig]) {
    info!("──────── Motor Configuration ───────");
    info!(" Loaded {} motors:", motors.len());
    for m in motors {
        info!(
            " • {:<14} | id={:<3} type={}",
            m.joint_name, m.id, m.motor_type
        );
    }
    info!("────────────────────────────────────");
}
