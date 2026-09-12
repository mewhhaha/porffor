#!/usr/bin/env python3
"""Verify a completed Test262 replay against its frozen inputs and native evidence."""

import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import re
import subprocess


SPEC = importlib.util.spec_from_file_location(
    "execution_replay", Path(__file__).with_name("replay-test262-executions.py"))
REPLAY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REPLAY)


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path):
    def unique_fields(pairs):
        fields = {}
        for key, value in pairs:
            require(key not in fields, f"{path}: duplicate JSON field {key!r}")
            fields[key] = value
        return fields

    try:
        document = json.loads(path.read_text(), object_pairs_hook=unique_fields)
    except (OSError, ValueError) as error:
        raise ValueError(f"{path}: {error}") from error
    require(isinstance(document, dict), f"{path}: expected a JSON object")
    return document


def exact_membership(actual, expected, label):
    missing, extra = sorted(expected - actual), sorted(actual - expected)
    require(not missing and not extra,
            f"{label}: missing {missing[:5]}, extra {extra[:5]}")


def git(directory, *arguments):
    result = subprocess.run(["git", "-C", str(directory), *arguments],
                            capture_output=True, text=True)
    require(result.returncode == 0,
            f"suite Git evidence unavailable ({' '.join(arguments)}): {result.stderr.strip()}")
    return result.stdout.strip()


class SuiteIdentity:
    def __init__(self, directory):
        self.directory = directory.resolve(strict=True)
        self.repository = Path(git(self.directory, "rev-parse", "--show-toplevel"))
        self.prefix = self.directory.relative_to(self.repository).as_posix()
        require(not git(self.repository, "status", "--porcelain", "--untracked-files=all",
                        "--", self.prefix), f"suite contents are dirty: {self.directory}")
        self.tree = git(self.repository, "rev-parse", "--verify",
                        "HEAD^{tree}" if self.prefix == "." else f"HEAD:{self.prefix}")
        self.verified_pins = set()

    def verify_pin(self, pin):
        require(isinstance(pin, str) and re.fullmatch(r"(?:[a-f0-9]{40}|[a-f0-9]{64})", pin),
                f"invalid or unpublishable Test262 pin: {pin!r}")
        if pin in self.verified_pins:
            return
        kind = git(self.repository, "cat-file", "-t", pin)
        require(kind in ("tree", "commit"), f"Test262 pin is not a tree or commit: {pin}")
        expression = (f"{pin}^{{tree}}" if self.prefix == "." else f"{pin}:{self.prefix}")
        tree = pin if kind == "tree" else git(self.repository, "rev-parse", "--verify", expression)
        require(tree == self.tree, f"Test262 pin mismatch: {pin} resolves to {tree}, suite is {self.tree}")
        self.verified_pins.add(pin)

    def source(self, execution):
        relative = PurePosixPath(execution.split(":", 1)[1])
        require(not relative.is_absolute() and ".." not in relative.parts,
                f"execution source escapes suite: {execution}")
        root = (self.directory / "test").resolve(strict=True)
        source = (root / relative).resolve(strict=True)
        require(source.is_relative_to(root), f"execution source escapes suite: {execution}")
        return source


def index_rows(rows, label):
    require(isinstance(rows, list), f"{label}: expected a row array")
    indexed = {}
    for row in rows:
        require(isinstance(row, dict) and isinstance(row.get("execution_id"), str),
                f"{label}: missing execution identity")
        execution = row["execution_id"]
        require(execution not in indexed, f"{label}: duplicate execution {execution}")
        require(REPLAY.parse_executions(execution) == [execution],
                f"{label}: expected one canonical execution identity: {execution!r}")
        require(row.get("outcome") in REPLAY.OUTCOMES, f"{label}: invalid outcome for {execution}")
        indexed[execution] = row
    return indexed


def audit_snapshot(path, execution, outcome, suite):
    snapshot = read_json(path)
    require(snapshot.get("producer") == "lila" and snapshot.get("execution_backend") == "wasm-aot"
            and snapshot.get("run_kind") == "full", f"{path}: not a native full-run snapshot")
    suite.verify_pin(snapshot["pinned_revisions"]["test262"])
    manifest = snapshot.get("manifest_hash")
    require(type(manifest) is int and 0 <= manifest < 2**64,
            f"{path}: invalid manifest hash")
    key = hashlib.sha256(execution.encode()).hexdigest()
    require(path.name == f"{key}-{manifest}.json", f"{path}: manifest filename mismatch")
    require(snapshot.get("completed_test_ids") == [execution],
            f"{path}: completed execution identity mismatch")
    expected = {name: int(name == outcome) for name in REPLAY.OUTCOMES}
    counts = snapshot.get("counts_per_outcome")
    require(type(snapshot.get("total")) is int and snapshot["total"] == 1
            and type(snapshot.get("passed")) is int and snapshot["passed"] == expected["Success"]
            and counts == expected and all(type(count) is int for count in counts.values()),
            f"{path}: native outcome counts mismatch")
    failures = snapshot.get("failures")
    require(isinstance(failures, list) and len(failures) == int(outcome != "Success"),
            f"{path}: failure count mismatch")
    for failure in failures:
        require(failure.get("test_id") == execution and failure.get("outcome") == outcome
                and failure.get("test_path") == execution.split(":", 1)[1],
                f"{path}: failure identity or outcome mismatch")
    kinds = snapshot.get("counts_per_kind")
    require(isinstance(kinds, dict) and all(type(count) is int and count >= 0 for count in kinds.values())
            and Counter({name: count for name, count in kinds.items() if count})
            == Counter(failure.get("kind") for failure in failures), f"{path}: failure kind counts mismatch")
    timeouts = snapshot.get("timeout_test_ids")
    require(timeouts == [] or (timeouts == [execution] and outcome == "Crash"),
            f"{path}: timeout identity or outcome mismatch")
    return bool(timeouts)


def audit(directory, compiler_path, origin_path=None):
    directory = directory.resolve(strict=True)
    run = read_json(directory / "run.json")
    summary = read_json(directory / "summary.json")
    compiler = read_json(compiler_path)
    require(all(summary.get(key) == value for key, value in run.items()),
            "summary metadata differs from run.json")
    require(sha256(directory / "compiler") == compiler.get("binary_sha256") == run.get("binary_sha256"),
            "compiler binary SHA-256 mismatch")
    execution_hash = sha256(directory / "executions")
    require(execution_hash == run.get("execution_list_sha256"), "execution list SHA-256 mismatch")
    executions = REPLAY.parse_executions((directory / "executions").read_text())
    rows = index_rows(summary.get("results"), "summary")
    exact_membership(set(rows), set(executions), "summary execution membership")
    require(type(summary.get("total")) is int and summary["total"] == len(rows),
            "summary total does not match execution inventory")
    require(isinstance(run.get("suite_root"), str) and Path(run["suite_root"]).is_absolute(),
            "run.json must record an absolute suite_root")
    suite = SuiteIdentity(Path(run["suite_root"]))
    if "test262_tree_pin" in compiler:
        suite.verify_pin(compiler["test262_tree_pin"])
    origin = read_json(origin_path) if origin_path else None
    old = index_rows(origin.get("rows"), "origin report") if origin is not None else {}
    if origin is not None:
        suite.verify_pin(origin["test262_tree_pin"])
        require(type(origin.get("total")) is int and origin["total"] == len(old), "origin report total mismatch")
        old_counts = origin.get("outcomes")
        require(isinstance(old_counts, dict) and old_counts.keys() <= set(REPLAY.OUTCOMES)
                and all(type(count) is int and count >= 0 for count in old_counts.values())
                and Counter(old_counts) == Counter(row["outcome"] for row in old.values()),
                "origin report outcome counts mismatch")
        for execution, row in old.items():
            require(type(row.get("timeout")) is bool and (not row["timeout"] or row["outcome"] == "Crash"),
                    f"{execution}: origin timeout status missing or inconsistent")
            require(isinstance(row.get("source_sha256"), str) and re.fullmatch(r"[a-f0-9]{64}", row["source_sha256"]),
                    f"{execution}: origin source SHA-256 missing or invalid")
        if "timeouts" in origin:
            require(type(origin["timeouts"]) is int and origin["timeouts"] == sum(row["timeout"] for row in old.values()),
                    "origin report timeout count mismatch")
    keys = {hashlib.sha256(execution.encode()).hexdigest() for execution in executions}
    exact_membership({path.name for path in directory.glob("*.json")} - {"run.json", "summary.json"},
                     {f"{key}.json" for key in keys}, "saved results")
    transcripts = {path.name for path in directory.glob("*.log")}
    # Resume retains interrupted attempts; these are not completed native evidence.
    interrupted = {name for name in transcripts
                   if re.fullmatch(r"[a-f0-9]{64}\.interrupted-\d+\.log", name) and name[:64] in keys}
    exact_membership(transcripts - interrupted, {f"{key}.log" for key in keys}, "native transcripts")
    snapshots = set()
    saved = []
    for execution, row in sorted(rows.items()):
        key = hashlib.sha256(execution.encode()).hexdigest()
        require(row == read_json(directory / f"{key}.json"), f"{execution}: saved result differs from summary")
        require(row.get("transcript") == f"{key}.log" and "infrastructure_error" not in row,
                f"{execution}: invalid native transcript or infrastructure failure")
        require(type(row.get("exit_code")) is int, f"{execution}: invalid native exit code")
        transcript = directory / row["transcript"]
        try:
            outcome = REPLAY.native_outcome(transcript.read_text(), row["exit_code"])
        except ValueError as error:
            raise ValueError(f"{execution}: {error}") from error
        require(row["outcome"] == outcome, f"{execution}: outcome differs from native transcript")
        matches = list((directory / "snapshots").glob(f"{key}-*.json"))
        require(len(matches) == 1, f"{execution}: expected one snapshot, found {len(matches)}")
        snapshot = matches[0]
        snapshots.add(snapshot.name)
        timeout = audit_snapshot(snapshot, execution, outcome, suite)
        record = {"execution_id": execution, "outcome": outcome,
                  "source_sha256": sha256(suite.source(execution)),
                  "snapshot_sha256": sha256(snapshot), "transcript_sha256": sha256(transcript),
                  "result_sha256": sha256(directory / f"{key}.json"), "timeout": timeout}
        if execution in old:
            require(old[execution].get("source_sha256") == record["source_sha256"],
                    f"{execution}: origin source SHA-256 mismatch")
            record["origin_outcome"] = old[execution]["outcome"]
            record["origin_timeout"] = old[execution]["timeout"]
        saved.append(record)
    exact_membership({path.name for path in (directory / "snapshots").glob("*.json")},
                     snapshots, "snapshot inventory")
    counts = Counter(row["outcome"] for row in saved)
    outcomes = {name: counts[name] for name in REPLAY.OUTCOMES}
    require(summary.get("outcomes") == outcomes
            and all(type(count) is int for count in summary["outcomes"].values()),
            "summary outcome counts mismatch")
    transitions = Counter(f"{row['origin_outcome']} -> {row['outcome']}"
                          for row in saved if "origin_outcome" in row)
    repaired = [row for row in saved if row.get("origin_outcome") in REPLAY.OUTCOMES[1:]
                and row["outcome"] == "Success"]
    timeout_rechecks = sum(row["origin_timeout"] for row in repaired)
    return {"schema_version": 1, "test262_tree_pin": suite.tree, "suite_root": str(suite.directory),
            "recorded_test262_pins": sorted(suite.verified_pins), "compiler": compiler,
            "compiler_metadata_sha256": sha256(compiler_path), "run_sha256": sha256(directory / "run.json"),
            "execution_list_sha256": execution_hash, "native_summary_sha256": sha256(directory / "summary.json"),
            "total": len(saved), "outcomes": outcomes, "timeouts": sum(row["timeout"] for row in saved),
            "origin_report_sha256": sha256(origin_path) if origin_path else None,
            "origin_overlap": sum(transitions.values()), "origin_transitions": dict(sorted(transitions.items())),
            "repaired_origin_failures": len(repaired),
            "repaired_origin_timeout_rechecks": timeout_rechecks,
            "repaired_origin_non_timeout_failures": len(repaired) - timeout_rechecks,
            "retained_origin_successes": transitions["Success -> Success"], "rows": saved}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--compiler", type=Path, required=True, help="JSON metadata containing binary_sha256")
    parser.add_argument("--origin", type=Path, help="previous audit report to compare by exact execution and source")
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    try:
        report = audit(options.directory, options.compiler, options.origin)
        require(options.output.resolve() not in {options.compiler.resolve(),
                options.origin.resolve() if options.origin else None}
                and not options.output.resolve().is_relative_to(options.directory.resolve()),
                "audit output must be outside the replay directory and its input reports")
        REPLAY.write_checkpoint(options.output, report)
    except (OSError, ValueError, KeyError, TypeError, AttributeError) as error:
        parser.exit(2, f"audit failed: {error}\n")
    print(json.dumps({key: value for key, value in report.items() if key != "rows"}, indent=2))


if __name__ == "__main__":
    main()
