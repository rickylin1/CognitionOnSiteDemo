use charging_session::process;
use common::{Event, EventType};

const VIN: &str = "TESTVIN0000000001";

fn event(id: &str, vin: &str, ts: &str, event_type: EventType, soc_pct: u8) -> Event {
    Event {
        event_id: id.into(),
        vin: vin.into(),
        ts: ts.parse().unwrap(),
        event_type,
        soc_pct,
        battery_temp_c: None,
        lat: 37.77493,
        lon: -122.41942,
    }
}

fn start(id: &str, vin: &str, ts: &str, soc: u8) -> Event {
    event(id, vin, ts, EventType::ChargeStart, soc)
}

fn stop(id: &str, vin: &str, ts: &str, soc: u8) -> Event {
    event(id, vin, ts, EventType::ChargeStop, soc)
}

#[test]
fn duplicate_events_produce_one_session() {
    let s = start("evt-1", VIN, "2026-11-20T09:00:00Z", 60);
    let e = stop("evt-2", VIN, "2026-11-20T09:47:30Z", 88);
    let out = process(vec![s.clone(), s, e.clone(), e]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].duration_min, 47);
    assert_eq!(out[0].soc_gained_pct, 28);
}

#[test]
fn stop_arriving_before_start_still_closes_session() {
    let out = process(vec![
        stop("evt-19", VIN, "2026-11-20T13:15:59Z", 75),
        start("evt-20", VIN, "2026-11-20T12:00:00Z", 30),
    ]);
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].session_start.to_rfc3339(),
        "2026-11-20T12:00:00+00:00"
    );
    assert_eq!(out[0].duration_min, 75);
    assert_eq!(out[0].soc_gained_pct, 45);
}

#[test]
fn orphan_stop_and_open_session_are_not_emitted() {
    let out = process(vec![
        stop("evt-1", VIN, "2026-11-20T05:00:00Z", 10),
        start("evt-2", VIN, "2026-11-20T10:00:00Z", 15),
        stop("evt-3", VIN, "2026-11-20T11:05:20Z", 55),
        start("evt-4", VIN, "2026-11-20T18:00:00Z", 40),
    ]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].start_soc_pct, 15);
    assert_eq!(out[0].duration_min, 65);
}

#[test]
fn second_start_while_open_is_ignored() {
    let out = process(vec![
        start("evt-1", VIN, "2026-11-20T09:00:00Z", 20),
        start("evt-2", VIN, "2026-11-20T09:10:00Z", 25),
        stop("evt-3", VIN, "2026-11-20T10:00:00Z", 70),
    ]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].start_soc_pct, 20);
    assert_eq!(out[0].duration_min, 60);
}

#[test]
fn open_session_does_not_carry_over_to_next_vin() {
    let out = process(vec![
        start("evt-1", "TESTVIN0000000001", "2026-11-20T09:00:00Z", 20),
        stop("evt-2", "TESTVIN0000000002", "2026-11-20T10:00:00Z", 70),
    ]);
    assert!(out.is_empty());
}

#[test]
fn location_comes_from_start_rounded_to_two_decimals() {
    let mut e = stop("evt-2", VIN, "2026-11-20T09:47:30Z", 88);
    e.lat = 37.77511;
    let out = process(vec![start("evt-1", VIN, "2026-11-20T09:00:00Z", 60), e]);
    assert_eq!((out[0].lat, out[0].lon), (37.77, -122.42));
}

#[test]
fn sorted_by_vin_then_session_start() {
    let out = process(vec![
        start("evt-5", "TESTVIN0000000002", "2026-11-20T08:00:00Z", 10),
        stop("evt-6", "TESTVIN0000000002", "2026-11-20T09:00:00Z", 20),
        start("evt-3", VIN, "2026-11-20T12:00:00Z", 30),
        stop("evt-4", VIN, "2026-11-20T13:00:00Z", 40),
        start("evt-1", VIN, "2026-11-20T07:00:00Z", 10),
        stop("evt-2", VIN, "2026-11-20T08:00:00Z", 20),
    ]);
    let keys: Vec<_> = out
        .iter()
        .map(|r| (r.vin.as_str(), r.start_soc_pct))
        .collect();
    assert_eq!(keys, [(VIN, 10), (VIN, 30), ("TESTVIN0000000002", 10)]);
}
