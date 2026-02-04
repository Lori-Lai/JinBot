# Feetech Driver

English follows the Chinese section.

## 简介
本项目是 Dora 节点，用于通过串口控制 Feetech STS 系列舵机，周期发布状态并响应控制指令。核心逻辑在 `src/main.rs`，功能拆分在模块：
- `config.rs`：环境变量读取与默认参数
- `motors.rs`：舵机 JSON 配置解析
- `controller.rs`：串口打开 / ping / 重连
- `handlers.rs`：控制写入、状态读取与发布
- `display.rs`：分段日志输出
公共日志工具在上级 `common` crate 中。

## 快速开始
```bash
cargo build -p feetech_driver
RUST_LOG=info cargo run -p feetech_driver
```
在 Dora 数据流中运行时，由 daemon 设置的环境变量会自动接管输入输出。

## 环境变量
- `CONFIG_PATH`：舵机配置 JSON 路径，默认 `../config/motor.json`
- `SERIAL_PORT`：串口设备，默认 `/dev/ttyACM0`
- `SERIAL_BAUD`：波特率，默认 `1_000_000`
- `SERIAL_TIMEOUT_MS`：串口超时，默认 `1000`
- `RUST_LOG`：日志等级与过滤（tracing-subscriber）

## 舵机配置 JSON
支持两种格式：
1) 数组：
```json
[
  {"joint_name": "shoulder_pan", "motor_type": "sts3215", "id": 1},
  {"joint_name": "elbow", "motor_type": "sts3215", "id": 5}
]
```
2) 映射：
```json
{
  "shoulder_pan": {"motor_type": "sts3215", "id": 1},
  "elbow": {"motor_type": "sts3215", "id": 5}
}
```
数组缺省的 `joint_name` 会按 `joint_{index}` 填充；映射缺省时用键名。

## 运行时行为
1. 初始化 tracing，打印配置与电机列表（分隔块）。
2. 打开串口并依次 ping 所有 id，失败时记录 warning，后续操作会自动重试串口。
3. 进入事件循环：
   - 定时读取位置，序列化 JSON 并通过 Dora 输出发布。
   - 处理控制输入（Vec<f64> 目标角度，单位弧度）。
   - 未识别输入按 1s 聚合限流打印。
   - 串口异常不会终止，重试重连。

## English
### Overview
`feetech_driver` is a Dora node that drives Feetech STS servos over a serial bus, publishes joint states periodically, and applies goal positions from control inputs. Main flow lives in `src/main.rs`; helper modules split configuration, motor parsing, controller lifecycle, handlers, and logging display. Shared logging utilities live in the sibling `common` crate.

### Quick Start
```bash
cargo build -p feetech_driver
RUST_LOG=info cargo run -p feetech_driver
```
When launched by Dora, daemon-provided env vars wire inputs/outputs automatically.

### Env Vars
- `CONFIG_PATH`: motor config JSON path (default `../config/motor.json`)
- `SERIAL_PORT`: device path, default `/dev/ttyACM0`
- `SERIAL_BAUD`: baud rate, default `1_000_000`
- `SERIAL_TIMEOUT_MS`: serial timeout ms, default `1000`
- `STATUS_PERIOD_MS`: state publish period ms, default `50`
- `CONTROL_INPUT_ID`: Dora input id for control, default `control`
- `STATUS_OUTPUT_ID`: Dora output id for status, default `status`
- `RUST_LOG`: tracing filter (e.g., `info`, `debug`)

### Motor Config JSON
Array or map form accepted:
```json
[
  {"joint_name": "shoulder_pan", "motor_type": "sts3215", "id": 1}
]
```
or
```json
{
  "shoulder_pan": {"motor_type": "sts3215", "id": 1}
}
```
If `joint_name` is missing: array → `joint_{index}`, map → key name.

### Runtime Flow
1) Init tracing; log settings and motors with section dividers.  
2) Open serial and ping all IDs; on failure, warn and retry later.  
3) Event loop: periodic state read → JSON → Dora output; control goals from input (Vec<f64> radians); unknown inputs are rate-limited; serial errors trigger reopen instead of exiting.
