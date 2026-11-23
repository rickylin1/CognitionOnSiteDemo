#!/usr/bin/env python3
"""Compare a Rust job's output with the legacy golden output, record by record.

Usage: diff.py <actual.jsonl> <golden.jsonl>
Exit 0 on PASS, 1 on FAIL. Python stdlib only.

Records are compared as parsed JSON values, not as text: key order and
number formatting don't matter (80 == 80.0), but types do (true != 1, "80" != 80).
Floats are compared exactly, so a rounding difference in lat/lon is a FAIL.
"""
import json
import sys

MISSING = object()


def load(path):
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def same(a, b):
    """JSON value equality: ints and floats compare by value, nothing else is coerced."""
    if isinstance(a, bool) or isinstance(b, bool):
        return type(a) is type(b) and a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a == b
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return type(a) is type(b) and a == b


def show(value):
    return "<missing>" if value is MISSING else json.dumps(value)


def differing_fields(actual, expected):
    keys = sorted(set(actual) | set(expected))
    return [k for k in keys if not same(actual.get(k, MISSING), expected.get(k, MISSING))]


def canonical(record):
    return json.dumps(record, sort_keys=True)


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    actual, expected = load(sys.argv[1]), load(sys.argv[2])
    total = max(len(actual), len(expected))

    failures, bad_fields = 0, set()
    for i in range(total):
        a = actual[i] if i < len(actual) else MISSING
        e = expected[i] if i < len(expected) else MISSING
        if a is not MISSING and e is not MISSING and same(a, e):
            continue
        failures += 1
        print(f"record {i + 1}:")
        if a is MISSING:
            print(f"  missing, expected {canonical(e)}")
        elif e is MISSING:
            print(f"  unexpected extra record {canonical(a)}")
        else:
            print(f"  got      {canonical(a)}\n  expected {canonical(e)}")
            for field in differing_fields(a, e):
                bad_fields.add(field)
                print(f"  field '{field}': got {show(a.get(field, MISSING))}, "
                      f"expected {show(e.get(field, MISSING))}")

    if failures:
        fields = f"; fields that differ: {', '.join(sorted(bad_fields))}" if bad_fields else ""
        print(f"FAIL: {failures} of {total} records differ "
              f"(got {len(actual)} records, expected {len(expected)}){fields}")
        return 1
    print(f"PASS: {len(expected)} records match legacy output")
    return 0


if __name__ == "__main__":
    sys.exit(main())
