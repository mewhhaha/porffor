#!/usr/bin/env python3
"""Collect exact failed execution IDs from completed native matrix checkpoints."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


OUTCOMES = ("Success", "NotImplemented", "Bug", "Crash")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_input(path):
    content = path.read_bytes()
    return content, {"name": path.name, "sha256": hashlib.sha256(content).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("aggregate", type=Path)
    parser.add_argument("--exclude", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    options = parser.parse_args()
    aggregate_bytes, aggregate_source = read_input(options.aggregate)
    aggregate = json.loads(aggregate_bytes)
    if aggregate["run_kind"] != "aggregate-matrix":
        parser.error("aggregate must be a native matrix checkpoint")
    exclusion_bytes, exclusion_source = read_input(options.exclude)
    exclusion_ids = [
        line.strip() for line in exclusion_bytes.decode().splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    excluded = set(exclusion_ids)
    require(len(excluded) == len(exclusion_ids), "duplicate excluded execution")
    completed = set(aggregate["completed_nodes"])
    require(len(completed) == len(aggregate["completed_nodes"]), "duplicate completed node")
    represented_nodes = set()
    observed_ids = set()
    observed_failures = {}
    observed_outcomes = Counter()
    leaves = []
    for entry in aggregate["aggregate_entries"]:
        if entry["node_id"] not in completed:
            continue
        require(entry["node_id"] not in represented_nodes, f"duplicate completed entry: {entry['node_id']}")
        represented_nodes.add(entry["node_id"])
        checkpoints = list(options.aggregate.parent.glob(f"*-{entry['manifest_hash']}.json"))
        if len(checkpoints) != 1:
            raise ValueError(f"expected one checkpoint for {entry['node_id']}: {checkpoints}")
        checkpoint_bytes, checkpoint_source = read_input(checkpoints[0])
        checkpoint = json.loads(checkpoint_bytes)
        if checkpoint["pinned_revisions"] != aggregate["pinned_revisions"]:
            raise ValueError(f"pin mismatch: {checkpoints[0]}")
        require(checkpoint["producer"] == aggregate["producer"]
                and checkpoint["execution_backend"] == aggregate["execution_backend"],
                f"producer or backend mismatch: {checkpoints[0]}")
        require(checkpoint["manifest_hash"] == entry["manifest_hash"], f"manifest mismatch: {checkpoints[0]}")
        require(checkpoint["run_kind"] in ("matrix-filter-leaf", "matrix-chunk-leaf"),
                f"checkpoint is not a completed matrix leaf: {checkpoints[0]}")
        if len(checkpoint["failures"]) != entry["failed"]:
            raise ValueError(f"failure count mismatch: {checkpoints[0]}")
        identities = set(checkpoint["completed_test_ids"])
        require(len(identities) == len(checkpoint["completed_test_ids"]) == checkpoint["total"],
                f"completed execution inventory mismatch: {checkpoints[0]}")
        require(not observed_ids & identities, f"duplicate completed execution across leaves: {checkpoints[0]}")
        observed_ids.update(identities)
        failures = {}
        for failure in checkpoint["failures"]:
            identity = failure["test_id"]
            if identity in failures:
                raise ValueError(f"duplicate failed execution: {identity}")
            require(identity in identities, f"failure absent from completed execution inventory: {identity}")
            require(failure["outcome"] in OUTCOMES[1:], f"invalid failure outcome: {identity}")
            failures[identity] = failure
        counts = Counter(failure["outcome"] for failure in failures.values())
        counts["Success"] = len(identities) - len(failures)
        outcomes = {name: counts[name] for name in OUTCOMES}
        require(checkpoint["counts_per_outcome"] == entry["counts_per_outcome"] == outcomes,
                f"outcome count mismatch: {checkpoints[0]}")
        require(checkpoint["total"] == entry["total"]
                and checkpoint["passed"] == entry["passed"] == counts["Success"],
                f"checkpoint total or passed count mismatch: {checkpoints[0]}")
        observed_outcomes.update(counts)
        observed_failures.update(failures)
        leaves.append({"node_id": entry["node_id"], **checkpoint_source,
                       "total": len(identities), "passed": counts["Success"],
                       "failed": len(failures), "outcomes": outcomes})
    require(represented_nodes == completed, "completed nodes lack aggregate entries")
    if len(observed_ids) != aggregate["total"]:
        raise ValueError(f"completed checkpoint totals differ: {len(observed_ids)} != {aggregate['total']}")
    outcomes = {name: observed_outcomes[name] for name in OUTCOMES}
    require(aggregate["counts_per_outcome"] == outcomes and aggregate["passed"] == outcomes["Success"],
            "aggregate outcome or passed counts differ from completed checkpoints")
    failures = {identity: failure for identity, failure in observed_failures.items() if identity not in excluded}
    outputs = {
        "failures.executions": "".join(identity + "\n" for identity in sorted(failures)),
        "failures.json": json.dumps(list(failures.values()), indent=2) + "\n",
    }
    remaining_outcomes = Counter(failure["outcome"] for failure in failures.values())
    provenance = {
        "schema_version": 1,
        "producer": aggregate["producer"],
        "execution_backend": aggregate["execution_backend"],
        "pinned_revisions": aggregate["pinned_revisions"],
        "aggregate": aggregate_source,
        "exclusion": {**exclusion_source, "execution_count": len(excluded),
                      "observed_execution_count": len(excluded & observed_ids),
                      "excluded_failure_count": len(excluded & observed_failures.keys())},
        "completed_leaves": sorted(leaves, key=lambda leaf: leaf["node_id"]),
        "observed": {"completed_nodes": len(completed), "total": len(observed_ids),
                     "passed": outcomes["Success"], "failed": len(observed_failures), "outcomes": outcomes},
        "collected": {"failed": len(failures), "outcomes": {name: remaining_outcomes[name] for name in OUTCOMES}},
        "outputs": {name: {"sha256": hashlib.sha256(content.encode()).hexdigest()} for name, content in outputs.items()},
    }
    options.output_dir.mkdir(parents=True, exist_ok=False)
    for name, content in outputs.items():
        (options.output_dir / name).write_bytes(content.encode())
    (options.output_dir / "provenance.json").write_bytes((json.dumps(provenance, indent=2) + "\n").encode())
    print(f"Collected {len(failures)} failed executions outside the excluded cohort")


if __name__ == "__main__":
    main()
