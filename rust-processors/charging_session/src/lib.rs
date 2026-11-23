//! Port of legacy-spark/ChargingSessionJob.scala.

use chrono::{DateTime, Utc};
use common::{dedup_by_event_id, sort_by_vin_and_time, Event, EventType};
use serde::Serialize;

/// One closed charging session. Location is the charge_start's, rounded to 2 decimals (R4).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChargingSession {
    pub vin: String,
    pub session_start: DateTime<Utc>,
    pub session_end: DateTime<Utc>,
    pub duration_min: i64,
    pub start_soc_pct: i32,
    pub end_soc_pct: i32,
    pub soc_gained_pct: i32,
    pub lat: f64,
    pub lon: f64,
}

/// R4: round half away from zero to 2 decimal places.
pub fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Pure function: events in (any order, may contain duplicates),
/// closed sessions out (sorted by vin, then session_start).
pub fn process(events: Vec<Event>) -> Vec<ChargingSession> {
    // R1: dedup before anything else.
    let mut events: Vec<Event> = dedup_by_event_id(events)
        .into_iter()
        .filter(|e| matches!(e.event_type, EventType::ChargeStart | EventType::ChargeStop))
        .collect();

    // R2: per-VIN event-time order.
    sort_by_vin_and_time(&mut events);

    let mut out = Vec::new();
    let mut open: Option<Event> = None;
    for e in events {
        // New VIN: an open session from the previous VIN is never closed, so never emitted.
        if open.as_ref().is_some_and(|s| s.vin != e.vin) {
            open = None;
        }
        match e.event_type {
            // Opens a session; ignored if one is already open.
            EventType::ChargeStart if open.is_none() => open = Some(e),
            // Closes the open session; an orphan stop is ignored.
            EventType::ChargeStop => {
                if let Some(s) = open.take() {
                    out.push(ChargingSession {
                        vin: e.vin,
                        session_start: s.ts,
                        session_end: e.ts,
                        duration_min: (e.ts - s.ts).num_minutes(),
                        start_soc_pct: s.soc_pct.into(),
                        end_soc_pct: e.soc_pct.into(),
                        soc_gained_pct: i32::from(e.soc_pct) - i32::from(s.soc_pct),
                        lat: round2(s.lat),
                        lon: round2(s.lon),
                    });
                }
            }
            _ => {}
        }
    }
    // Sorted input means sessions are already ordered by (vin, session_start).
    out
}
