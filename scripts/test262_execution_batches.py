"""Exact replay batches consume native run-list snapshots, never invented verdicts."""

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import time

OUTCOMES = ("Success", "NotImplemented", "Crash", "Bug")


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def native_inventory(transcript):
    lines = transcript.splitlines()
    if not lines or not re.fullmatch(r"count: [0-9]+", lines[0]):
        raise ValueError("native inventory lacks its exact count")
    count = int(lines[0].split(": ", 1)[1])
    ids = lines[1:]
    if (
        len(ids) != count
        or len(set(ids)) != count
        or any(
            not re.fullmatch(
                r"(?:sloppy-script|strict-script|raw-script|module|raw-module):[^\s]+\.js",
                value,
            )
            for value in ids
        )
    ):
        raise ValueError(
            "native inventory is truncated or contains invalid or repeated IDs"
        )
    return set(ids)


def owned_file(options, name):
    if (
        not isinstance(name, str)
        or Path(name).name != name
        or not (options.output_dir / name).resolve().is_relative_to(options.output_dir)
    ):
        raise ValueError("saved replay evidence is outside its invocation directory")
    return options.output_dir / name


def snapshot_verdicts(snapshot, executions, identity, exit_code=None):
    expected = set(executions)
    completed = snapshot.get("completed_test_ids", [])
    if (
        not expected
        or len(expected) != len(executions)
        or not isinstance(completed, list)
        or any(not isinstance(value, str) for value in completed)
        or len(set(completed)) != len(completed)
        or set(completed) != expected
        or type(snapshot.get("total")) is not int
        or snapshot.get("total") != len(expected)
        or snapshot.get("snapshot_version") != 8
        or snapshot.get("execution_backend") != "wasm-aot"
        or snapshot.get("run_kind") != "full"
        or snapshot.get("compiler_identity") != identity
    ):
        raise ValueError(
            "native batch snapshot has inconsistent IDs, compiler or execution scope"
        )
    pins = snapshot.get("pinned_revisions", {})
    if set(pins) != {"ecma262", "test262"} or any(
        not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{40}", value)
        for value in pins.values()
    ):
        raise ValueError("native batch snapshot lacks exact pinned revisions")
    counts = snapshot_counts(snapshot, len(expected))
    verdicts, failed = snapshot_failure_verdicts(snapshot, expected, counts)
    timeouts = snapshot.get("timeout_test_ids", [])
    if len(set(timeouts)) != len(timeouts) or not set(timeouts).issubset(failed):
        raise ValueError("native batch timeout identities disagree with its failures")
    if exit_code is not None and (
        exit_code not in (0, 1)
        or (exit_code == 0) != (counts["Success"] == len(expected))
    ):
        raise ValueError("native batch verdict disagrees with its command exit status")
    return verdicts


def snapshot_counts(snapshot, expected_total):
    counts = snapshot.get("counts_per_outcome", {})
    if (
        set(counts) != set(OUTCOMES)
        or any(type(value) is not int or value < 0 for value in counts.values())
        or sum(counts.values()) != expected_total
        or type(snapshot.get("passed")) is not int
        or snapshot.get("passed") != counts["Success"]
    ):
        raise ValueError("native batch outcome counts do not reconcile")
    return counts


def snapshot_failure_verdicts(snapshot, expected, counts):
    verdicts = {execution: "Success" for execution in expected}
    failures = snapshot.get("failures", [])
    failed = set()
    for failure in failures:
        execution, outcome = failure.get("test_id"), failure.get("outcome")
        if (
            execution not in expected
            or execution in failed
            or outcome not in OUTCOMES[1:]
        ):
            raise ValueError(
                "native batch has an unknown, repeated or incorrectly classified failure"
            )
        failed.add(execution)
        verdicts[execution] = outcome
    if {
        outcome: sum(value == outcome for value in verdicts.values())
        for outcome in OUTCOMES
    } != counts or len(failures) != len(expected) - counts["Success"]:
        raise ValueError("native batch failure identities disagree with its counts")
    return verdicts, failed


def frozen_executions(options, executions, inventory):
    """Bind the entire requested denominator, including absent native IDs."""
    if not executions or len(set(executions)) != len(executions):
        raise ValueError("requested executions must be nonempty and unique")
    present = [execution for execution in executions if execution in inventory]
    absent = [execution for execution in executions if execution not in inventory]
    fixture_hashes = {}
    test_root = (options.suite_root / "test").resolve()
    for execution in executions:
        relative = execution.split(":", 1)[1]
        path = (test_root / relative).resolve()
        if not path.is_relative_to(test_root):
            raise ValueError("requested fixture is outside the selected test directory")
        fixture_hashes[relative] = digest(path) if path.is_file() else None
    return present, absent, fixture_hashes


def prepare(options, manifest, executions, write_checkpoint):
    command = [str(options.binary), "--jobs", "1", "compiler-identity"]
    completed = subprocess.run(command, capture_output=True, check=True)
    (options.output_dir / "compiler-identity.stdout").write_bytes(completed.stdout)
    (options.output_dir / "compiler-identity.stderr").write_bytes(completed.stderr)
    identity = json.loads(completed.stdout)
    if (
        set(identity)
        != {
            "source_fingerprint_scheme",
            "source_fingerprint",
            "source_revision",
            "executable_sha256",
        }
        or not isinstance(identity["source_fingerprint_scheme"], str)
        or not identity["source_fingerprint_scheme"]
        or not re.fullmatch(r"[0-9a-f]{64}", identity["source_fingerprint"])
        or not isinstance(identity["source_revision"], dict)
        or not identity["source_revision"].get("kind")
        or identity.get("executable_sha256") != manifest["binary_sha256"]
    ):
        raise ValueError("native batch identity differs from the frozen compiler bytes")
    inventory_command = [
        str(options.binary),
        "--jobs",
        "1",
        "test262",
        "list-all",
        "--suite-root",
        str(options.suite_root),
    ]
    discovery = subprocess.run(inventory_command, capture_output=True, check=True)
    inventory_stdout = options.output_dir / "native-inventory.stdout"
    inventory_stdout.write_bytes(discovery.stdout)
    (options.output_dir / "native-inventory.stderr").write_bytes(discovery.stderr)
    inventory = native_inventory(discovery.stdout.decode("utf-8"))
    present, absent, fixture_hashes = frozen_executions(options, executions, inventory)
    # Absent IDs still run through the old exact native CLI and remain red.
    batch = options.output_dir / "batch-executions"
    batch.write_text("".join(execution + "\n" for execution in present))
    manifest.update(
        compiler_identity=identity,
        batch_execution_list=str(batch),
        batch_execution_list_sha256=digest(batch),
        batch_executions=present,
        absent_executions=absent,
        batch_threads=options.workers,
        fixture_sha256=fixture_hashes,
        native_inventory={
            "command": inventory_command,
            "stdout": inventory_stdout.name,
            "stdout_sha256": digest(inventory_stdout),
        },
    )
    write_checkpoint(options.output_dir / "run.json", manifest)


def validate_resume(options, manifest, executions):
    if manifest["batch_threads"] != options.workers:
        raise ValueError("resume worker count differs from the native batch")
    if (
        manifest["batch_execution_list"] != str(options.output_dir / "batch-executions")
        or digest(manifest["batch_execution_list"])
        != manifest["batch_execution_list_sha256"]
    ):
        raise ValueError("resume native batch input differs from its frozen list")
    batch = Path(manifest["batch_execution_list"]).read_text().splitlines()
    absent = manifest["absent_executions"]
    if (
        batch != manifest["batch_executions"]
        or len(batch) + len(absent) != len(executions)
        or len(set(batch + absent)) != len(executions)
        or set(batch + absent) != set(executions)
    ):
        raise ValueError(
            "resume native batch does not retain the complete requested denominator"
        )
    reference = manifest["native_inventory"]
    inventory_path = owned_file(options, reference["stdout"])
    if digest(inventory_path) != reference["stdout_sha256"]:
        raise ValueError("resume native inventory changed")
    inventory = native_inventory(inventory_path.read_text())
    present, missing, fixture_hashes = frozen_executions(options, executions, inventory)
    if batch != present or absent != missing:
        raise ValueError("resume partition disagrees with the native inventory")
    if fixture_hashes != manifest["fixture_sha256"]:
        raise ValueError("resume fixture bytes differ from the frozen replay input")


def verify_saved_result(result, manifest, options, snapshots):
    if digest(owned_file(options, result["transcript"])) != result["transcript_sha256"]:
        raise ValueError("saved replay transcript changed")
    if result.get("protocol") == "native-execution-list-v1":
        reference = result["native_snapshot"]
        path = owned_file(options, reference["path"])
        if not re.fullmatch(r"native-batch-[0-9a-f]{64}\.snapshot", path.name):
            raise ValueError("saved native replay snapshot has an invalid sealed name")
        if path not in snapshots:
            if digest(path) != reference["sha256"]:
                raise ValueError("saved native replay snapshot changed")
            snapshots[path] = (
                reference["sha256"],
                snapshot_verdicts(
                    json.loads(path.read_text()),
                    manifest["batch_executions"],
                    manifest["compiler_identity"],
                ),
            )
        if reference["sha256"] != snapshots[path][0] or result.get(
            "outcome"
        ) != snapshots[path][1].get(result["execution_id"]):
            raise ValueError("saved replay result disagrees with native evidence")
    elif (
        result["execution_id"] not in manifest["absent_executions"]
        or "infrastructure_error" not in result
        or "outcome" in result
        or result["exit_code"] == 0
    ):
        raise ValueError(
            "saved absent-ID error is not bound to the native batch denominator"
        )


def replay(options, manifest, executions, results, replay_one, write_checkpoint):
    completed_ids = {result["execution_id"] for result in results}
    for execution in manifest["absent_executions"]:
        if execution not in completed_ids:
            result = replay_one(execution, options)
            result["transcript_sha256"] = digest(
                options.output_dir / result["transcript"]
            )
            if result.get("outcome") or result["exit_code"] == 0:
                raise ValueError(
                    "an absent execution unexpectedly produced native case evidence"
                )
            key = hashlib.sha256(execution.encode()).hexdigest()
            write_checkpoint(options.output_dir / f"{key}.json", result)
            results.append(result)
    present = manifest["batch_executions"]
    if not present or set(present).issubset(completed_ids):
        return
    invocation = time.time_ns()
    stdout = options.output_dir / f"batch-{invocation}.stdout"
    stderr = options.output_dir / f"batch-{invocation}.stderr"
    command = [
        str(options.binary),
        "--jobs",
        "1",
        "test262",
        "run-list",
        manifest["batch_execution_list"],
        "--execution-backend",
        "wasm-aot",
        "--suite-root",
        str(options.suite_root),
        "--snapshot-dir",
        str(options.output_dir / "snapshots"),
        "--snapshot-name",
        "exact-batch",
        "--threads",
        str(options.workers),
        "--timeout-ms",
        "60000",
        "--reuse-case-workers",
    ]
    if options.resume:
        command.append("--resume")
    environment = os.environ.copy()
    environment["SCRIPTW_TEST262_FORCE_CASE_RUNNER"] = "1"
    environment.pop("SCRIPTW_TEST262_DISABLE_CASE_RUNNER", None)
    started = time.monotonic()
    previous_signal = signal.getsignal(signal.SIGTERM)

    def interrupted(_signum, _frame):
        raise KeyboardInterrupt("native batch interrupted; its checkpoint is resumable")

    signal.signal(signal.SIGTERM, interrupted)
    with stdout.open("xb") as a, stderr.open("xb") as b:
        child = subprocess.Popen(command, stdout=a, stderr=b, env=environment)
        try:
            next_update = started + 30
            while child.poll() is None:
                time.sleep(0.2)
                now = time.monotonic()
                if now >= next_update:
                    print(
                        f"native replay batch: {now - started:.0f}s; {stderr.stat().st_size} diagnostic bytes",
                        flush=True,
                    )
                    next_update = now + 30
            code = child.wait()
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait()
            signal.signal(signal.SIGTERM, previous_signal)
    invocation_path = options.output_dir / f"batch-{invocation}.json"
    write_checkpoint(
        invocation_path,
        {
            "command": command,
            "exit_code": code,
            "seconds": time.monotonic() - started,
            "stdout": stdout.name,
            "stdout_sha256": digest(stdout),
            "stderr": stderr.name,
            "stderr_sha256": digest(stderr),
        },
    )
    candidates = list((options.output_dir / "snapshots").glob("exact-batch-*.json"))
    if len(candidates) != 1:
        raise ValueError("native batch did not produce exactly one execution snapshot")
    snapshot = candidates[0]
    verdicts = snapshot_verdicts(
        json.loads(snapshot.read_text()), present, manifest["compiler_identity"], code
    )
    snapshot_hash, transcript_hash = digest(snapshot), digest(stdout)
    sealed = options.output_dir / ("native-batch-" + snapshot_hash + ".snapshot")
    if not sealed.exists():
        shutil.copyfile(snapshot, sealed)
    if digest(sealed) != snapshot_hash:
        raise ValueError("native batch changed while sealing its results")
    for execution in present:
        if execution in completed_ids:
            continue
        result = {
            "execution_id": execution,
            "exit_code": int(verdicts[execution] != "Success"),
            "outcome": verdicts[execution],
            "protocol": "native-execution-list-v1",
            "transcript": stdout.name,
            "transcript_sha256": transcript_hash,
            "native_snapshot": {"path": sealed.name, "sha256": snapshot_hash},
            "invocation": invocation_path.name,
        }
        key = hashlib.sha256(execution.encode()).hexdigest()
        write_checkpoint(options.output_dir / f"{key}.json", result)
        results.append(result)
