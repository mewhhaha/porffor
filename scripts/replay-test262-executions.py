#!/usr/bin/env python3
"""Replay explicit Test262 execution identities and retain native runner evidence."""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import time


OUTCOMES = ("Success", "NotImplemented", "Crash", "Bug")
EXECUTION_MODES = ("sloppy-script", "strict-script", "raw-script", "module", "raw-module")


def write_checkpoint(path, checkpoint):
    pending = path.with_suffix(path.suffix + ".pending")
    pending.write_text(json.dumps(checkpoint, indent=2) + "\n")
    pending.replace(path)


def parse_executions(source):
    executions = [line.strip() for line in source.splitlines()
                  if line.strip() and not line.lstrip().startswith("#")]
    if not executions:
        raise ValueError("execution list is empty")
    if len(set(executions)) != len(executions):
        raise ValueError("execution list contains duplicate identities")
    if any(not re.fullmatch("(?:" + "|".join(EXECUTION_MODES) + r"):[^\s]+\.js", execution)
           for execution in executions):
        raise ValueError("each line must be an exact mode:path.js execution identity")
    return executions


def native_outcome(transcript, exit_code):
    if re.findall(r"^total: (\d+)$", transcript, re.M) != ["1"]:
        raise ValueError("native runner did not report exactly one execution")
    counts = {}
    for outcome in OUTCOMES:
        matches = re.findall(r"^  " + outcome + r": (\d+)$", transcript, re.M)
        if len(matches) != 1:
            raise ValueError(f"missing or repeated native outcome count: {outcome}")
        counts[outcome] = int(matches[0])
    if sum(counts.values()) != 1:
        raise ValueError("native outcome counts do not reconcile to one execution")
    outcome = next(outcome for outcome, count in counts.items() if count == 1)
    if exit_code not in (0, 1) or (exit_code == 0) != (outcome == "Success"):
        raise ValueError("native outcome and command exit status disagree")
    return outcome


def replay(execution, options):
    key = hashlib.sha256(execution.encode()).hexdigest()
    transcript = options.output_dir / f"{key}.log"
    if transcript.exists():
        transcript.rename(transcript.with_suffix(f".interrupted-{time.time_ns()}.log"))
    command = [str(options.binary), "--jobs", "1", "test262", "run", execution,
               "--execution-backend", "wasm-aot", "--suite-root", str(options.suite_root),
               "--snapshot-dir", str(options.output_dir / "snapshots"),
               "--snapshot-name", key, "--threads", "1", "--timeout-ms", "60000"]
    environment = os.environ.copy()
    environment["LILA_TEST262_FORCE_CASE_RUNNER"] = "1"
    environment.pop("LILA_TEST262_DISABLE_CASE_RUNNER", None)
    with transcript.open("w") as output:
        completed = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT,
                                   env=environment)
    result = {"execution_id": execution, "exit_code": completed.returncode,
              "transcript": transcript.name}
    try:
        result["outcome"] = native_outcome(transcript.read_text(), completed.returncode)
    except ValueError as error:
        result["infrastructure_error"] = str(error)
    write_checkpoint(options.output_dir / f"{key}.json", result)
    return result


def run_replay(options):
    execution_list_bytes = options.execution_list.read_bytes()
    executions = parse_executions(execution_list_bytes.decode("utf-8"))
    options.binary = options.binary.resolve(strict=True)
    options.suite_root = options.suite_root.resolve(strict=True)
    options.output_dir = options.output_dir.resolve()
    frozen_execution_list = options.output_dir / "executions"
    binary_source = options.binary
    options.binary = options.output_dir / "compiler"
    manifest_path = options.output_dir / "run.json"
    results = []
    if options.resume:
        manifest = json.loads(manifest_path.read_text())
        frozen_binary_sha256 = hashlib.sha256(options.binary.read_bytes()).hexdigest()
        if (frozen_binary_sha256 != manifest["binary_sha256"]
                or hashlib.sha256(binary_source.read_bytes()).hexdigest() != frozen_binary_sha256):
            raise ValueError("resume compiler differs from the recorded frozen executable")
        if (frozen_execution_list.read_bytes() != execution_list_bytes
                or hashlib.sha256(execution_list_bytes).hexdigest() != manifest["execution_list_sha256"]):
            raise ValueError("resume execution list differs from the recorded frozen input")
        if (manifest["suite_root"] != str(options.suite_root)
                or manifest["binary"] != str(options.binary)
                or manifest["execution_list"] != str(frozen_execution_list)):
            raise ValueError("resume paths differ from the recorded run")
        execution_set = set(executions)
        for result_path in sorted(options.output_dir.glob("*.json")):
            if not re.fullmatch(r"[a-f0-9]{64}", result_path.stem):
                continue
            result = json.loads(result_path.read_text())
            execution = result["execution_id"]
            if (execution not in execution_set
                    or hashlib.sha256(execution.encode()).hexdigest() != result_path.stem
                    or result["transcript"] != result_path.with_suffix(".log").name):
                raise ValueError(f"saved result has inconsistent execution identity: {result_path}")
            outcome = native_outcome(result_path.with_suffix(".log").read_text(), result["exit_code"])
            if result.get("outcome") != outcome or "infrastructure_error" in result:
                raise ValueError(f"saved result disagrees with its native transcript: {result_path}")
            results.append(result)
        print(f"Resuming {len(results)}/{len(executions)} verified completed executions", flush=True)
    else:
        frozen_execution_list.write_bytes(execution_list_bytes)
        binary_digest = hashlib.sha256()
        # Every case executes this copy, even if another build replaces the CLI.
        with binary_source.open("rb") as source, options.binary.open("xb") as frozen:
            before = os.fstat(source.fileno())
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                frozen.write(chunk)
                binary_digest.update(chunk)
            after = os.fstat(source.fileno())
            if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
                raise ValueError("compiler executable changed while it was being frozen")
            os.fchmod(frozen.fileno(), stat.S_IMODE(before.st_mode))
        manifest = {"binary": str(options.binary), "binary_source": str(binary_source),
                    "suite_root": str(options.suite_root),
                    "binary_sha256": binary_digest.hexdigest(),
                    "execution_list_sha256": hashlib.sha256(execution_list_bytes).hexdigest(),
                    "execution_list": str(frozen_execution_list),
                    "execution_list_source": str(options.execution_list.resolve())}
        write_checkpoint(manifest_path, manifest)
    completed_executions = {result["execution_id"] for result in results}
    with ThreadPoolExecutor(max_workers=options.workers) as pool:
        futures = [pool.submit(replay, execution, options) for execution in executions
                   if execution not in completed_executions]
        for future in as_completed(futures):
            result = future.result()
            results.append(result)
            print(f"{len(results)}/{len(executions)} {result.get('outcome', 'InfrastructureError')} "
                  f"{result['execution_id']}", flush=True)
    results.sort(key=lambda result: result["execution_id"])
    summary = {**manifest, "total": len(results),
               "outcomes": {outcome: sum(r.get("outcome") == outcome for r in results)
                            for outcome in OUTCOMES}, "results": results}
    write_checkpoint(options.output_dir / "summary.json", summary)
    if any("infrastructure_error" in result for result in results):
        return 2
    return int(summary["outcomes"]["Success"] != len(executions))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("execution_list", type=Path)
    parser.add_argument("--binary", type=Path, default=Path("target/release/lila"))
    parser.add_argument("--suite-root", type=Path, default=Path("test262/vendor/test262"))
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--workers", type=int, choices=range(1, 65), default=2)
    parser.add_argument("--resume", action="store_true",
                        help="verify saved inputs/results and run only unfinished executions")
    options = parser.parse_args()
    if options.resume:
        options.output_dir = options.output_dir.resolve(strict=True)
    else:
        options.output_dir.mkdir(parents=True, exist_ok=False)
    with (options.output_dir / ".replay.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise ValueError("another replay already owns this evidence directory") from None
        return run_replay(options)


if __name__ == "__main__":
    raise SystemExit(main())
