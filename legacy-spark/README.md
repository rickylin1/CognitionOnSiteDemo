# legacy-spark

The two production Spark Structured Streaming jobs, in Scala. Both read the Kafka topic `vehicle-telemetry`.

**These files are for reading and porting only. They are not built or run in this repo.**

| Job | Rust port |
|---|---|
| `BatteryHealthJob.scala` | `rust-processors/battery_health/` (the reference port) |
| `ChargingSessionJob.scala` | none yet |

Each step is commented with the shared rule it implements (R1–R4, see `../AGENTS.md`).

In production, the golden outputs come from replaying a Kafka topic through these live jobs. Here, `fixtures/golden/` holds the outputs these jobs give for `fixtures/events.jsonl`, worked out by hand (see `../fixtures/README.md`).
