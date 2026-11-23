# fixtures

All data is synthetic. The VINs, coordinates and timestamps are made up.

- `events.jsonl`: 24 lines, 23 unique events, 4 vehicles, all on 2026-11-20 (UTC). Lines are in **arrival** order, which is not event-time (`ts`) order. Replaying this file stands in for replaying the `vehicle-telemetry` Kafka topic.
- `golden/<job>.jsonl`: what the legacy Spark job outputs for `events.jsonl`, worked out by hand from the job semantics in `../AGENTS.md`.

Do not edit these files to make a test pass.

## Edge cases (line numbers refer to `events.jsonl`)

| Edge case | Where |
|---|---|
| Duplicate | `evt-0013` (TESTVIN0000000001 battery_status) is on lines 13 and 14, with an identical payload. |
| Out-of-order charge | TESTVIN0000000002: `charge_stop` `evt-0019` (13:15:59, line 20) arrives before `charge_start` `evt-0020` (12:00:00, line 21). |
| Out-of-order battery_status | TESTVIN0000000004: `evt-0014` (12:00, line 15) arrives before `evt-0015` (09:00, line 16) and `evt-0017` (10:30, line 18). |
| Open session | TESTVIN0000000003: `charge_start` `evt-0023` (18:00, line 24) has no later `charge_stop`. |
| Orphan stop | TESTVIN0000000004: `charge_stop` `evt-0001` (05:00, line 1) has no `charge_start` before it. |

## How each VIN's golden records are derived

**TESTVIN0000000001**
- battery_health: the latest status is `evt-0013` at 10:00 (soc 87, 29.5 °C). Its duplicate is dropped first (R1).
- charging_session: start `evt-0009` 09:00 (soc 60) → stop `evt-0012` 09:47:30 (soc 88). Duration 47.5 min, floored to 47; gained 28. lat/lon come from the start (37.77493, -122.41942) and round to 37.77, -122.42. The stop's GPS fix (37.77511) would round to 37.78, so using the wrong event shows up in the diff.

**TESTVIN0000000002**
- battery_health: the latest status is `evt-0022` at 13:30 (soc 74, 31.5 °C).
- charging_session: in event-time order, start `evt-0020` 12:00:00 (soc 30) → stop `evt-0019` 13:15:59 (soc 75). Duration 75.98 min, floored to 75; gained 45; lat/lon 40.71, -74.01. In arrival order the stop would come first, be treated as an orphan, and the session would be lost.

**TESTVIN0000000003**
- battery_health: the latest status is `evt-0010` at 09:00 (soc 69, 27.5 °C).
- charging_session: start `evt-0004` 07:00 (soc 20) → stop `evt-0008` 08:30 (soc 70). Duration 90; gained 50; lat/lon 47.61, -122.33. Start `evt-0023` at 18:00 is never closed, so it is not emitted.

**TESTVIN0000000004**
- battery_health: the latest status is `evt-0014` at 12:00 (soc 54, 26.5 °C), even though it arrives before the 09:00 and 10:30 readings.
- charging_session: the orphan stop `evt-0001` at 05:00 is ignored. Start `evt-0016` 10:00 (soc 15) → stop `evt-0018` 11:05:20 (soc 55). Duration 65.33 min, floored to 65; gained 40; lat/lon 34.05, -118.24.

Golden files are sorted by `vin` (charging_session: then by `session_start`).

No `lat`/`lon` value sits exactly on a 2-decimal rounding tie (`x.xx5`), so any correct rounding method gives the golden values.
