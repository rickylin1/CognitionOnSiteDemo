#!/usr/bin/env python3
"""Compare a Rust job's output with the legacy golden output, record by record.

Usage: diff.py <actual.jsonl> <golden.jsonl>
Exit 0 on PASS, 1 on FAIL. Python stdlib only.
"""
import json
import sys


def load(path):
    """Read JSONL and canonicalize each record (sorted keys, no whitespace)."""
    with open(path) as f:
        return [
            json.dumps(json.loads(line), sort_keys=True, separators=(",", ":"))
            for line in f
            if line.strip()
        ]


def field_diff(actual, expected):
    a, e = json.loads(actual), json.loads(expected)
    for key in sorted(set(a) | set(e)):
        if a.get(key, "<missing>") != e.get(key, "<missing>"):
            print(f"    {key}: got {a.get(key, '<missing>')!r}, expected {e.get(key, '<missing>')!r}")


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    actual, expected = load(sys.argv[1]), load(sys.argv[2])

    failures = 0
    for i in range(max(len(actual), len(expected))):
        a = actual[i] if i < len(actual) else None
        e = expected[i] if i < len(expected) else None
        if a == e:
            continue
        failures += 1
        print(f"record {i + 1}:")
        if a is None:
            print(f"  missing, expected {e}")
        elif e is None:
            print(f"  unexpected extra record {a}")
        else:
            print(f"  got      {a}\n  expected {e}")
            field_diff(a, e)

    if failures:
        print(f"FAIL: {failures} of {max(len(actual), len(expected))} records differ "
              f"(got {len(actual)} records, expected {len(expected)})")
        return 1
    print(f"PASS: {len(expected)} records match legacy output")
    return 0


if __name__ == "__main__":
    sys.exit(main())
