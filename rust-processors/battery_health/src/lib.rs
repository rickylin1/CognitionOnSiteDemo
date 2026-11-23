//! Port of legacy-spark/BatteryHealthJob.scala.
//! This crate is the reference pattern for every future port.

use chrono::{DateTime, Utc};
use common::{dedup_by_event_id, sort_by_vin_and_time, Event, EventType};
use serde::Serialize;

/// Latest battery reading per vehicle. No location fields at all (R4).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BatteryHealth {
    pub vin: String,
    pub ts: DateTime<Utc>,
    pub soc_pct: u8,
    pub battery_temp_c: Option<f64>,
}

/// Pure function: events in (any order, may contain duplicates), records out (sorted by vin).
pub fn process(events: Vec<Event>) -> Vec<BatteryHealth> {
    // R1: dedup before anything else.
    let mut events: Vec<Event> = dedup_by_event_id(events)
        .into_iter()
        .filter(|e| e.event_type == EventType::BatteryStatus)
        .collect();

    // R2: per-VIN event-time order.
    sort_by_vin_and_time(&mut events);

    // Events are sorted by (vin, ts), so the last event of each VIN run is its latest.
    let mut out: Vec<BatteryHealth> = Vec::new();
    for e in events {
        let record = BatteryHealth {
            vin: e.vin,
            ts: e.ts,
            soc_pct: e.soc_pct,
            battery_temp_c: e.battery_temp_c,
        };
        match out.last_mut() {
            Some(prev) if prev.vin == record.vin => *prev = record,
            _ => out.push(record),
        }
    }
    out
}
