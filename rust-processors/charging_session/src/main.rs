//! Usage: charging_session <input.jsonl>
//! Records go to stdout as JSONL. Only counts go to stderr (R3).

use std::process::ExitCode;

use common::{dedup_by_event_id, read_events, write_jsonl};

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: charging_session <input.jsonl>");
        return ExitCode::from(2);
    };
    let events = match read_events(&path) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("charging_session: failed to read input: {e}");
            return ExitCode::FAILURE;
        }
    };

    let read = events.len();
    let duplicates = read - dedup_by_event_id(events.clone()).len();
    let records = charging_session::process(events);

    match write_jsonl(&records) {
        Ok(written) => {
            eprintln!("charging_session: events read={read} duplicates dropped={duplicates} records written={written}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("charging_session: failed to write output: {e}");
            ExitCode::FAILURE
        }
    }
}
