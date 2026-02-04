use std::sync::Arc;

use crate::motors::MotorConfig;
use arrow::array::{ArrayRef, Float64Array, StringArray, StructArray};
use arrow::datatypes::{DataType, Field};
use dora_arrow_convert::ArrowData;
use dora_core::config::DataId;
use dora_message::metadata::MetadataParameters;
use dora_node_api::DoraNode;
use eyre::{eyre, Result};
use rustypot::servo::feetech::sts3215::Sts3215Controller;

pub fn handle_control(
    motors: &[MotorConfig],
    controller: &mut Option<Sts3215Controller>,
    data: &ArrowData,
) -> Result<()> {
    let goals: Vec<f64> = data
        .try_into()
        .map_err(|_| eyre!("control data not Vec<f64>"))?;

    if goals.len() != motors.len() {
        return Err(eyre!(
            "control len {} does not match motors {}",
            goals.len(),
            motors.len()
        ));
    }

    let ids: Vec<u8> = motors.iter().map(|m| m.id).collect();
    let controller = controller
        .as_mut()
        .ok_or_else(|| eyre!("controller not ready"))?;

    controller
        .sync_write_goal_position(&ids, &goals)
        .map_err(|e| eyre!("sync_write_goal_position: {e}"))?;

    Ok(())
}

pub fn publish_status(
    motors: &[MotorConfig],
    controller: &mut Option<Sts3215Controller>,
    node: &mut DoraNode,
    status_output: &DataId,
) -> Result<()> {
    let controller = controller
        .as_mut()
        .ok_or_else(|| eyre!("controller not ready"))?;

    let ids: Vec<u8> = motors.iter().map(|m| m.id).collect();
    let positions = controller
        .sync_read_present_position(&ids)
        .map_err(|e| eyre!("sync_read_present_position: {e}"))?;

    let positions: Vec<f64> = positions
        .into_iter()
        .map(|p| (p * 100.0).round() / 100.0)
        .collect();

    let joint_names: Vec<String> = motors.iter().map(|m| m.joint_name.clone()).collect();

    let fields_and_arrays: Vec<(Arc<Field>, ArrayRef)> = vec![
        (
            Arc::new(Field::new("joint_name", DataType::Utf8, false)),
            Arc::new(StringArray::from(joint_names)) as ArrayRef,
        ),
        (
            Arc::new(Field::new("value", DataType::Float64, false)),
            Arc::new(Float64Array::from(positions)) as ArrayRef,
        ),
    ];

    let struct_array = StructArray::from(fields_and_arrays);

    node.send_output(
        status_output.clone(),
        MetadataParameters::default(),
        struct_array,
    )
    .map_err(|e| eyre!("send status: {e}"))?;

    Ok(())
}
