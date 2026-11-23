//! Usage: battery_health <input.jsonl>
//! Records go to stdout as JSONL. Only counts go to stderr (R3).

use std::process::ExitCode;

use common::{dedup_by_event_id, read_events, write_jsonl};

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: battery_health <input.jsonl>");
        return ExitCode::from(2);
    };
    let events = match read_events(&path) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("battery_health: failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    };

    let read = events.len();
    let duplicates = read - dedup_by_event_id(events.clone()).len();
    let records = battery_health::process(events);

    match write_jsonl(&records) {
        Ok(written) => {
            eprintln!("battery_health: events read={read} duplicates dropped={duplicates} records written={written}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("battery_health: failed to write output: {e}");
            ExitCode::FAILURE
        }
    }
}
