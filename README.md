# telemetry-migration-demo

A small, fully synthetic demo of moving vehicle-telemetry stream jobs from Scala/Spark Streaming to Rust.

- `battery_health` **has been ported** to Rust. That port is the pattern every later port copies.
- `charging_session` **has not been ported yet.**

In production, events come from the Kafka topic `vehicle-telemetry`. Here, replaying `fixtures/events.jsonl` stands in for that topic, and `fixtures/golden/` holds what the legacy Spark jobs output for it.

## Layout

```
legacy-spark/       Scala Spark jobs (read-only reference, not built here)
rust-processors/    Cargo workspace: common/ (event model, R1/R2 helpers) + one crate per ported job
fixtures/           events.jsonl (synthetic input) + golden/ (legacy outputs)
harness/            replay_and_diff.sh + diff.py: does the Rust port match legacy?
```

## Try it

```bash
cd rust-processors && cargo test && cd ..
./harness/replay_and_diff.sh battery_health     # PASS: 4 records match legacy output
./harness/replay_and_diff.sh charging_session   # No Rust port of charging_session yet
```

Requires a Rust toolchain and Python 3. The rules every job follows (R1–R4) and the porting pattern are in [AGENTS.md](AGENTS.md).
