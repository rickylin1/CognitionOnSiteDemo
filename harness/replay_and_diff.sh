#!/usr/bin/env bash
# Usage: ./harness/replay_and_diff.sh <job>
# Replays fixtures/events.jsonl (our stand-in for the Kafka topic) through the
# Rust port of <job> and diffs the result against the legacy Spark golden output.
set -euo pipefail

job="${1:?usage: $0 <job>}"
root="$(cd "$(dirname "$0")/.." && pwd)"

if [ ! -d "$root/rust-processors/$job" ]; then
  echo "No Rust port of $job yet"
  exit 1
fi

mkdir -p "$root/out"
cargo run --quiet --manifest-path "$root/rust-processors/Cargo.toml" -p "$job" -- \
  "$root/fixtures/events.jsonl" > "$root/out/$job.jsonl"

python3 "$root/harness/diff.py" "$root/out/$job.jsonl" "$root/fixtures/golden/$job.jsonl"
