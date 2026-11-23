//! Shared event model and the rules every job applies first:
//! R1 (dedup by event_id) and R2 (per-VIN event-time order).

use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One telemetry event, as published to the `vehicle-telemetry` topic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub vin: String,
    /// Event time (when it happened in the car), not arrival time.
    pub ts: DateTime<Utc>,
    pub event_type: EventType,
    pub soc_pct: u8,
    /// Only set on `battery_status` events.
    pub battery_temp_c: Option<f64>,
    /// Full-precision location. Never log it (R3) and never output it unrounded (R4).
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    BatteryStatus,
    ChargeStart,
    ChargeStop,
}

/// Read a JSONL file (our stand-in for replaying the Kafka topic), in arrival order.
/// Errors report the line number only, never the line content (R3).
pub fn read_events(path: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let reader = BufReader::new(File::open(path)?);
    let mut events = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let event = serde_json::from_str(&line)
            .map_err(|e| format!("invalid event on line {}: {:?}", i + 1, e.classify()))?;
        events.push(event);
    }
    Ok(events)
}

/// R1: keep the first occurrence of each event_id, drop the rest.
pub fn dedup_by_event_id(events: Vec<Event>) -> Vec<Event> {
    let mut seen = HashSet::new();
    events
        .into_iter()
        .filter(|e| seen.insert(e.event_id.clone()))
        .collect()
}

/// R2: order by VIN, then event time, tie-break by event_id. Never trust arrival order.
pub fn sort_by_vin_and_time(events: &mut [Event]) {
    events.sort_by(|a, b| (&a.vin, a.ts, &a.event_id).cmp(&(&b.vin, b.ts, &b.event_id)));
}

/// Write one JSON object per line to stdout. Returns the number of records written.
pub fn write_jsonl<T: Serialize>(records: &[T]) -> io::Result<usize> {
    let mut out = io::stdout().lock();
    for record in records {
        serde_json::to_writer(&mut out, record)?;
        out.write_all(b"\n")?;
    }
    out.flush()?;
    Ok(records.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str, vin: &str, ts: &str) -> Event {
        Event {
            event_id: id.into(),
            vin: vin.into(),
            ts: ts.parse().unwrap(),
            event_type: EventType::BatteryStatus,
            soc_pct: 50,
            battery_temp_c: Some(20.5),
            lat: 0.0,
            lon: 0.0,
        }
    }

    #[test]
    fn dedup_keeps_first_occurrence_of_each_event_id() {
        let events = vec![
            event("evt-1", "VIN_A", "2026-01-01T00:00:00Z"),
            event("evt-2", "VIN_A", "2026-01-01T00:01:00Z"),
            event("evt-1", "VIN_A", "2026-01-01T00:00:00Z"),
        ];
        let ids: Vec<_> = dedup_by_event_id(events)
            .into_iter()
            .map(|e| e.event_id)
            .collect();
        assert_eq!(ids, ["evt-1", "evt-2"]);
    }

    #[test]
    fn sort_orders_by_vin_then_ts_then_event_id() {
        let mut events = vec![
            event("evt-3", "VIN_B", "2026-01-01T00:00:00Z"),
            event("evt-2", "VIN_A", "2026-01-01T00:05:00Z"),
            event("evt-9", "VIN_A", "2026-01-01T00:01:00Z"),
            event("evt-1", "VIN_A", "2026-01-01T00:01:00Z"),
        ];
        sort_by_vin_and_time(&mut events);
        let ids: Vec<_> = events.iter().map(|e| e.event_id.as_str()).collect();
        assert_eq!(ids, ["evt-1", "evt-9", "evt-2", "evt-3"]);
    }
}
