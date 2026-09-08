#!/usr/bin/env python3
"""Replay explicit Test262 execution identities and retain native runner evidence."""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess


OUTCOMES = ("Success", "NotImplemented", "Crash", "Bug")
EXECUTION_MODES = ("sloppy-script", "strict-script", "raw-script", "module", "raw-module")


def read_executions(path):
    executions = [line.strip() for line in path.read_text().splitlines()
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
    (options.output_dir / f"{key}.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("execution_list", type=Path)
    parser.add_argument("--binary", type=Path, default=Path("target/release/lila"))
    parser.add_argument("--suite-root", type=Path, default=Path("test262/vendor/test262"))
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--workers", type=int, choices=range(1, 65), default=2)
    options = parser.parse_args()
    executions = read_executions(options.execution_list)
    options.binary = options.binary.resolve(strict=True)
    options.suite_root = options.suite_root.resolve(strict=True)
    options.output_dir = options.output_dir.resolve()
    options.output_dir.mkdir(parents=True, exist_ok=False)
    with options.binary.open("rb") as binary:
        binary_sha256 = hashlib.file_digest(binary, "sha256").hexdigest()
    execution_list_sha256 = hashlib.sha256(options.execution_list.read_bytes()).hexdigest()
    results = []
    with ThreadPoolExecutor(max_workers=options.workers) as pool:
        futures = [pool.submit(replay, execution, options) for execution in executions]
        for future in as_completed(futures):
            result = future.result()
            results.append(result)
            print(f"{len(results)}/{len(executions)} {result.get('outcome', 'InfrastructureError')} "
                  f"{result['execution_id']}", flush=True)
    results.sort(key=lambda result: result["execution_id"])
    summary = {"binary": str(options.binary), "suite_root": str(options.suite_root),
               "binary_sha256": binary_sha256,
               "execution_list_sha256": execution_list_sha256,
               "execution_list": str(options.execution_list), "total": len(results),
               "outcomes": {outcome: sum(r.get("outcome") == outcome for r in results)
                            for outcome in OUTCOMES}, "results": results}
    (options.output_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    if any("infrastructure_error" in result for result in results):
        return 2
    return int(summary["outcomes"]["Success"] != len(executions))


if __name__ == "__main__":
    raise SystemExit(main())
