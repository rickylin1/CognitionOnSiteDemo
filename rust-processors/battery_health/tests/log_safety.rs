//! R3/R4: run the real binary on the fixture and check what leaks.

use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/events.jsonl")
}

/// Every VIN, lat and lon value in the fixture, as serde_json prints it.
fn sensitive_values() -> (Vec<String>, Vec<String>) {
    let text = std::fs::read_to_string(fixture()).unwrap();
    let (mut vins, mut coords) = (Vec::new(), Vec::new());
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line).unwrap();
        vins.push(v["vin"].as_str().unwrap().to_string());
        coords.push(v["lat"].to_string());
        coords.push(v["lon"].to_string());
    }
    (vins, coords)
}

#[test]
fn logs_contain_no_vin_or_location() {
    let output = Command::new(env!("CARGO_BIN_EXE_battery_health"))
        .arg(fixture())
        .output()
        .unwrap();
    assert!(output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let (vins, coords) = sensitive_values();

    for vin in &vins {
        assert!(!stderr.contains(vin.as_str()), "stderr leaks a VIN");
    }
    for coord in &coords {
        assert!(
            !stderr.contains(coord.as_str()),
            "stderr leaks a coordinate"
        );
        // R4: full-precision coordinates must never reach the output either.
        assert!(
            !stdout.contains(coord.as_str()),
            "stdout leaks a full-precision coordinate"
        );
    }
}
