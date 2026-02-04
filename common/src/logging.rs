use std::{
    collections::HashMap,
    env,
    time::{Duration, Instant},
};

use tracing::warn;

pub fn init_tracing() {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

#[derive(Debug)]
pub struct WarnCounter {
    count: u64,
    last_emit: Instant,
}

pub fn rate_limited_warn(counters: &mut HashMap<String, WarnCounter>, id: &str) -> bool {
    const EMIT_INTERVAL: Duration = Duration::from_millis(1000);
    let entry = counters
        .entry(id.to_string())
        .or_insert_with(|| WarnCounter {
            count: 0,
            last_emit: Instant::now(),
        });
    entry.count += 1;
    if entry.last_emit.elapsed() >= EMIT_INTERVAL {
        warn!(
            "Ignored input '{}' ({} times in last {:?})",
            id,
            entry.count,
            entry.last_emit.elapsed()
        );
        entry.count = 0;
        entry.last_emit = Instant::now();
        return true;
    }
    false
}
