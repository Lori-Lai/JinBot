use socketcan::{embedded_can::Id, CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};
use std::collections::HashSet;
use std::error::Error;
use std::io::ErrorKind;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    // Open CAN interface
    let ifname = std::env::var("CAN_IF").unwrap_or_else(|_| "can0".to_string());
    let can = CanSocket::open(&ifname)?;
    // Keep receive non-blocking-ish with short timeout
    can.set_read_timeout(Duration::from_millis(5))?;
    can.set_write_timeout(Duration::from_millis(5))?;

    // Configurable scan range and listen window
    let max_id: u16 = std::env::var("SCAN_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x7F); // 0..127
    let listen_ms: u64 = std::env::var("SCAN_WINDOW_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000); // listen 1s after sending all probes

    println!(
        "Scanning CAN IDs 0x000..0x{:03X} on {} (listen {} ms)",
        max_id, ifname, listen_ms
    );

    // Send empty data frames ("ping") to each ID in range
    for target_id in 0u16..=max_id {
        let std_id = StandardId::new(target_id).expect("valid 11-bit id");
        if let Err(e) = can.write_frame(&CanFrame::new(std_id, &[]).unwrap()) {
            eprintln!("⚠️ send to id 0x{:03X} failed: {e}", target_id);
        }
    }

    // Collect responses during the listen window
    let deadline = Instant::now() + Duration::from_millis(listen_ms);
    let mut seen: HashSet<u32> = HashSet::new();
    while Instant::now() < deadline {
        match can.read_frame() as Result<CanFrame, std::io::Error> {
            Ok(f) => {
                let resp_id_raw = match f.id() {
                    Id::Standard(id) => id.as_raw() as u32,
                    Id::Extended(id) => id.as_raw(),
                };
                if seen.insert(resp_id_raw) {
                    println!(
                        "Found device: id=0x{:03X}, dlc={}, data={:02X?}",
                        resp_id_raw,
                        f.dlc(),
                        f.data()
                    );
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // no frame this poll; keep waiting
            }
            Err(e) => {
                eprintln!("⚠️ read failed: {e}");
            }
        }
    }

    if seen.is_empty() {
        println!("No devices responded. Check wiring/bitrate or extend SCAN_MAX/SCAN_WINDOW_MS.");
    } else {
        let list: Vec<String> = seen.iter().map(|id| format!("0x{:03X}", id)).collect();
        println!("Scan complete. Detected IDs: {:?}", list);
    }

    Ok(())
}
