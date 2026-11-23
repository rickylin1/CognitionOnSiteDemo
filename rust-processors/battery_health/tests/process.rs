use battery_health::process;
use common::{Event, EventType};

fn status(id: &str, vin: &str, ts: &str, soc_pct: u8) -> Event {
    Event {
        event_id: id.into(),
        vin: vin.into(),
        ts: ts.parse().unwrap(),
        event_type: EventType::BatteryStatus,
        soc_pct,
        battery_temp_c: Some(25.5),
        lat: 12.34567,
        lon: -76.54321,
    }
}

#[test]
fn duplicate_events_produce_one_record() {
    let e = status("evt-1", "TESTVIN0000000001", "2026-11-20T10:00:00Z", 80);
    let out = process(vec![e.clone(), e]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].soc_pct, 80);
}

#[test]
fn latest_by_event_time_wins_regardless_of_arrival_order() {
    // The newer reading arrives first; the older one arrives last.
    let out = process(vec![
        status("evt-2", "TESTVIN0000000001", "2026-11-20T12:00:00Z", 54),
        status("evt-1", "TESTVIN0000000001", "2026-11-20T09:00:00Z", 16),
    ]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].soc_pct, 54);
    assert_eq!(out[0].ts.to_rfc3339(), "2026-11-20T12:00:00+00:00");
}

#[test]
fn ignores_charge_events_and_sorts_by_vin() {
    let mut charge = status("evt-3", "TESTVIN0000000001", "2026-11-20T23:00:00Z", 99);
    charge.event_type = EventType::ChargeStart;
    let out = process(vec![
        status("evt-1", "TESTVIN0000000002", "2026-11-20T09:00:00Z", 10),
        status("evt-2", "TESTVIN0000000001", "2026-11-20T09:00:00Z", 20),
        charge,
    ]);
    let vins: Vec<_> = out.iter().map(|r| r.vin.as_str()).collect();
    assert_eq!(vins, ["TESTVIN0000000001", "TESTVIN0000000002"]);
    assert_eq!(out[0].soc_pct, 20);
}
