use std::{
    collections::HashMap,
    env,
    f64::consts::PI,
    fs,
    path::PathBuf,
    time::Duration,
};

use eyre::Result;
use dora_node_api::{DoraNode, Event};
use rustypot::servo::feetech::sts3215::Sts3215Controller;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MotorConfig {
    id: u8,
    model: String,
    torque: bool,
}

fn main() -> Result<()> {
    println!("Starting Feetech Driver...");

    // Allow overriding the motor config path via env; default to repo-level config/motor.json
    let default_config: PathBuf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../config/motor.json");
    let config_path = env::var("MOTOR_CONFIG")
        .map(PathBuf::from)
        .unwrap_or(default_config);

    let json = fs::read_to_string(&config_path)?;
    let motors: HashMap<String, MotorConfig> = serde_json::from_str(&json)?;
    println!(
        "Loaded motor config from {} ({} motors)",
        config_path.display(),
        motors.len()
    );
    for (name, cfg) in &motors {
        println!("- {name}: id={}, model={}, torque={}", cfg.id, cfg.model, cfg.torque);
    }

    let serial_port = serialport::new(
        env::var("PORT").unwrap_or_else(|_| "/dev/ttyACM0".to_string()),
        1_000_000,
    )
    .timeout(Duration::from_millis(1000))
    .open()?;

    let mut controller = Sts3215Controller::new()
        .with_protocol_v1()
        .with_serial_port(serial_port);

    // Track last commanded goal per motor (radians) and scan direction (+1 or -1) to avoid jumps.
    let mut last_goal_rad: HashMap<u8, f64> = motors.values().map(|cfg| (cfg.id, 0.0)).collect();
    let mut dir: HashMap<u8, i8> = motors.values().map(|cfg| (cfg.id, 1)).collect(); // 1=up, -1=down

    // Dora event loop
    let (_node, mut events) = DoraNode::init_from_env()?;
    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, .. } => match id.as_str() {
                // Ping-pong each motor by 1 degree (in radians) between [-PI, PI], avoiding wrap jumps.
                "write_goal_position" => {
                    const STEP_DEG: f64 = 1.0;
                    let step_rad = STEP_DEG.to_radians();
                    const MIN_RAD: f64 = -PI;
                    const MAX_RAD: f64 = PI;

                    let ids: Vec<u8> = motors.values().map(|m| m.id).collect();
                    let mut goals: Vec<f64> = Vec::with_capacity(ids.len());
                    for motor_id in &ids {
                        let current = *last_goal_rad.get(motor_id).unwrap_or(&0.0);
                        let d = *dir.get(motor_id).unwrap_or(&1) as f64;
                        let mut next = current + d * step_rad;
                        let mut new_dir = d;

                        if next >= MAX_RAD {
                            next = MAX_RAD;
                            new_dir = -1.0;
                        } else if next <= MIN_RAD {
                            next = MIN_RAD;
                            new_dir = 1.0;
                        }

                        last_goal_rad.insert(*motor_id, next);
                        dir.insert(*motor_id, new_dir as i8);
                        goals.push(next);
                    }
                    if let Err(e) = controller.sync_write_goal_position(&ids, &goals) {
                        eprintln!("⚠️ write_goal_position failed: {e}");
                    } else {
                        println!("Wrote goals (radians): {:?}", goals);
                    }
                }
                // Read present positions and print
                "read_current_position" => {
                    let ids: Vec<u8> = motors.values().map(|m| m.id).collect();
                    match controller.sync_read_present_position(&ids) {
                        Ok(positions) => {
                            for (i, pos) in ids.iter().zip(positions.iter()) {
                                let degrees = pos.to_degrees();
                                println!("Motor {i} position: {:.3} rad ({:.2}°)", pos, degrees);
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️ read_current_position failed: {e}");
                        }
                    }
                }
                other => eprintln!("ignoring unexpected input {other}"),
            },
            Event::Stop(_) => break,
            _ => {}
        }
    }

    Ok(())
}
