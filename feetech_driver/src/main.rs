mod config;
mod controller;
mod display;
mod handlers;
mod motors;

use std::collections::HashMap;

use common::logging::{init_tracing, rate_limited_warn, WarnCounter};
use dora_core::config::DataId;
use dora_node_api::{DoraNode, Event};
use eyre::Result;
use tracing::{error, warn};

use controller::{log_ping, open_and_ping};
use display::{log_motors, log_settings};
use handlers::{handle_control, publish_status};
fn main() -> Result<()> {
    // ────────────── 引导阶段：初始化日志 & 读取配置 ──────────────
    init_tracing();
    let settings = config::load_settings();
    log_settings(&settings);

    // ────────────── 配置舵机列表 ──────────────
    let motors = motors::load_motors(&settings.config_path)?;
    log_motors(&motors);

    // ────────────── 硬件准备：打开串口并 ping ──────────────
    let mut controller = log_ping(open_and_ping(&settings, &motors));

    // ────────────── 事件循环：处理控制 & 定时发布状态 ──────────────
    let status_output = DataId::from("present_position".to_owned());
    let (mut node, mut events) = DoraNode::init_from_env()?;
    let mut ignored: HashMap<String, WarnCounter> = HashMap::new();

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } => match id.as_str() {
                "goal_position" => {
                    if let Err(e) = handle_control(&motors, &mut controller, &data) {
                        let emitted = rate_limited_warn(&mut ignored, "goal_position_error");
                        if emitted {
                            error!("Failed to handle control data: {e:?}");
                        }
                        controller = controller::reopen_with_backoff(&settings, &motors);
                    }
                }
                "pull_present_position" => {
                    if let Err(e) =
                        publish_status(&motors, &mut controller, &mut node, &status_output)
                    {
                        let emitted = rate_limited_warn(&mut ignored, "status_error");
                        if emitted {
                            error!("Failed to publish status: {e:?}");
                        }
                        controller = controller::reopen_with_backoff(&settings, &motors);
                    }
                }
                other => {
                    warn!("Received data on unknown input ID: {}", other);
                }
            },
            Event::Stop(_) => break,
            other => {
                warn!("received unexpected event {other:?}");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dora_message::metadata::MetadataParameters;

    // 简单单元测试：构造一个 StructArray，字段 joint_name/value，确保 send_output 接口可接受。
    #[test]
    fn build_status_payload_shape() {
        let joints = vec!["j1".to_string(), "j2".to_string()];
        let values = vec![1.23_f64, 4.56_f64];

        let fields_and_arrays: Vec<(Arc<Field>, ArrayRef)> = vec![
            (
                Arc::new(Field::new("joint_name", DataType::Utf8, false)),
                Arc::new(StringArray::from(joints)) as ArrayRef,
            ),
            (
                Arc::new(Field::new("value", DataType::Float64, false)),
                Arc::new(Float64Array::from(values)) as ArrayRef,
            ),
        ];

        let struct_array = StructArray::from(fields_and_arrays);
        // Verify schema
        let fields = struct_array.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "joint_name");
        assert_eq!(fields[1].name(), "value");
        // Check lengths match
        assert_eq!(struct_array.len(), 2);

        // Dummy send_output signature check (type only)
        fn send_like(node: &mut DoraNode, id: DataId, arr: StructArray) {
            let _ = node.send_output(id, MetadataParameters::default(), arr);
        }

        // we can't call without real DoraNode; compile-time type check only
        let _ = ();
    }
}
