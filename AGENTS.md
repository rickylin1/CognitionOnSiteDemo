# AGENTS.md

## Purpose
This repo simulates migrating vehicle-telemetry stream jobs from Scala/Spark Streaming (`legacy-spark/`) to Rust (`rust-processors/`). A port is correct when it produces exactly the legacy output for the same input. We check that by replaying `fixtures/events.jsonl` (a stand-in for the `vehicle-telemetry` Kafka topic) through the Rust port and diffing against `fixtures/golden/<job>.jsonl`. `battery_health` is already ported and is the reference pattern. All data is synthetic.

## Layout
- `legacy-spark/`: Scala jobs. Read them to learn the semantics. They are not built or run here.
- `rust-processors/`: Cargo workspace.
  - `common/`: `Event`, `read_events`, `dedup_by_event_id` (R1), `sort_by_vin_and_time` (R2), `round2` (R4), `write_jsonl`.
  - `battery_health/`: the reference port.
- `fixtures/`: `events.jsonl`, `golden/<job>.jsonl`, and a README explaining every edge case.
- `harness/`: `replay_and_diff.sh <job>` and `diff.py`.

## Commands
```bash
cd rust-processors && cargo test          # unit + integration tests, including log safety
./harness/replay_and_diff.sh <job>        # run from repo root; prints PASS or FAIL
```

## Output format
Golden files and job output are JSONL: one JSON object per line, records in the job's documented sort order. `diff.py` compares parsed values, so key order and `80` vs `80.0` don't matter, but types and exact values do.

- **Timestamps** (`ts`, `session_start`, `session_end`): RFC 3339 UTC strings with a `Z` suffix and whole seconds, exactly as in the input, e.g. `"2026-11-20T09:47:30Z"` (not `+00:00`, no fractional seconds). Serializing a `chrono::DateTime<Utc>` with serde produces this format.
- **Integers:** `soc_pct`, `start_soc_pct`, `end_soc_pct`, `soc_gained_pct`, `duration_min`. `duration_min` is whole minutes, floored (`(end - start).num_minutes()`).
- **Floats:** `battery_temp_c` (or `null`), `lat`, `lon`.
- **`lat`/`lon` (R4):** rounded to 2 decimal places, half away from zero (`HALF_UP` in the Scala job). Use `common::round2` for all location rounding.
  Why: `(x * 100.0).round() / 100.0` misrounds float ties vs Scala's `BigDecimal` (e.g. 37.745 gives 37.74, not 37.75); `round2` rounds the decimal form instead.
- **VIN:** a 17-character string, copied unchanged.

## Porting pattern
Copy `battery_health/` exactly:
1. Make one crate per job: `rust-processors/<job>/`, added to `members` in `rust-processors/Cargo.toml`.
2. Put the logic in a pure `pub fn process(events: Vec<Event>) -> Vec<Output>` in `src/lib.rs`. No I/O and no logging in it.
3. Use `common` for R1 and R2: call `dedup_by_event_id` first, then `sort_by_vin_and_time`. Don't reimplement them.
4. Keep `src/main.rs` thin: `<job> <input.jsonl>`, then read, process, `write_jsonl` to stdout, and print counts only to stderr.
5. Copy the tests: `tests/process.rs` (duplicates, out-of-order input, plus the job's own edge cases) and `tests/log_safety.rs` (point it at the new binary with `env!("CARGO_BIN_EXE_<job>")`).

## Hard rules
- **R1** Deduplicate by `event_id` before anything else (exactly-once).
- **R2** Process each VIN's events in event-time order: sort by `ts`, tie-break by `event_id`. Never rely on file or arrival order.
- **R3** Never log VIN, lat or lon. Logs may contain only counts (events read, duplicates dropped, records written).
- **R4** Outputs may contain location only rounded to 2 decimal places. Full-precision coordinates must never appear in any output.
- Never add new outputs, sinks, files or network calls. A job writes its records to stdout and its counts to stderr, nothing else.
- Never change `fixtures/` or `fixtures/golden/` to make a test pass. If the golden output looks wrong, stop and ask.

## Definition of done for a port
- `cargo test` passes in `rust-processors/`.
- `./harness/replay_and_diff.sh <job>` prints `PASS`.
- `rust-processors/<job>/tests/log_safety.rs` exists and passes.
- The PR includes a data-handling summary: which fields are read, which are output, how R1–R4 are met, and confirmation that no logs, sinks or network calls were added.
