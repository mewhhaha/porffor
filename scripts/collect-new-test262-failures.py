#!/usr/bin/env python3
"""Collect exact failed execution IDs from completed native matrix checkpoints."""

import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("aggregate", type=Path)
    parser.add_argument("--exclude", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    options = parser.parse_args()
    aggregate = json.loads(options.aggregate.read_text())
    if aggregate["run_kind"] != "aggregate-matrix":
        parser.error("aggregate must be a native matrix checkpoint")
    excluded = {
        line.strip() for line in options.exclude.read_text().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    }
    completed = set(aggregate["completed_nodes"])
    failures = {}
    observed = 0
    for entry in aggregate["aggregate_entries"]:
        if entry["node_id"] not in completed:
            continue
        checkpoints = list(options.aggregate.parent.glob(f"*-{entry['manifest_hash']}.json"))
        if len(checkpoints) != 1:
            raise ValueError(f"expected one checkpoint for {entry['node_id']}: {checkpoints}")
        checkpoint = json.loads(checkpoints[0].read_text())
        if checkpoint["pinned_revisions"] != aggregate["pinned_revisions"]:
            raise ValueError(f"pin mismatch: {checkpoints[0]}")
        if len(checkpoint["failures"]) != entry["failed"]:
            raise ValueError(f"failure count mismatch: {checkpoints[0]}")
        observed += entry["total"]
        for failure in checkpoint["failures"]:
            identity = failure["test_id"]
            if identity in excluded:
                continue
            if identity in failures:
                raise ValueError(f"duplicate failed execution: {identity}")
            failures[identity] = failure
    if observed != aggregate["total"]:
        raise ValueError(f"completed checkpoint totals differ: {observed} != {aggregate['total']}")
    options.output_dir.mkdir(parents=True, exist_ok=False)
    (options.output_dir / "failures.executions").write_text(
        "".join(identity + "\n" for identity in sorted(failures))
    )
    (options.output_dir / "failures.json").write_text(
        json.dumps(list(failures.values()), indent=2) + "\n"
    )
    print(f"Collected {len(failures)} failed executions outside the excluded cohort")


if __name__ == "__main__":
    main()
