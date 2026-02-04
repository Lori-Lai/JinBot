use std::thread;
use std::time::Duration;

use eyre::{eyre, Result};
use rustypot::servo::feetech::sts3215::Sts3215Controller;
use tracing::{info, warn};

use crate::{config::Settings, motors::MotorConfig};

pub fn open_controller(settings: &Settings) -> Result<Sts3215Controller> {
    let serial_port = serialport::new(&settings.port, settings.baudrate)
        .timeout(Duration::from_millis(settings.timeout_ms))
        .open()
        .map_err(|e| eyre!("open serial port: {e}"))?;

    Ok(Sts3215Controller::new()
        .with_protocol_v1()
        .with_serial_port(serial_port))
}

pub fn open_and_ping(settings: &Settings, motors: &[MotorConfig]) -> Option<Sts3215Controller> {
    match open_controller(settings) {
        Ok(mut controller) => {
            if let Err(err) = ping_all(&mut controller, motors) {
                warn!("[ping] {err:?}");
                None
            } else {
                info!("Ping ok for all servos");
                Some(controller)
            }
        }
        Err(err) => {
            warn!("[serial-open] {err:?}");
            None
        }
    }
}

pub fn ping_all(controller: &mut Sts3215Controller, motors: &[MotorConfig]) -> Result<()> {
    for m in motors {
        match controller.ping(m.id) {
            Ok(true) => info!("Ping ok: {} (id={})", m.joint_name, m.id),
            Ok(false) => return Err(eyre!("Ping failed for id {}", m.id)),
            Err(err) => return Err(eyre!("Ping error for id {}: {err}", m.id)),
        }
    }
    Ok(())
}

pub fn reopen_with_backoff(
    settings: &Settings,
    motors: &[MotorConfig],
) -> Option<Sts3215Controller> {
    warn!("[serial] retrying to reopen...");
    thread::sleep(Duration::from_millis(200));
    open_and_ping(settings, motors)
}

pub fn log_ping(controller: Option<Sts3215Controller>) -> Option<Sts3215Controller> {
    info!("──────── Ping Test ────────────────");
    if controller.is_some() {
        info!(" Serial ready and all servos responded to ping");
    } else {
        warn!(" Ping failed; will retry on first operation");
    }
    info!("────────────────────────────────────");
    controller
}
