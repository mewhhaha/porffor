#!/usr/bin/env python3
"""Execute every compiled engine regression in a fresh, memory-bounded process.

No selection/skip list is accepted. Each exact invocation must pass once with the
rest of the complete inventory filtered out, and every inventory entry must run.
"""

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

from run_aot_unit_shard import parse_inventory, passed_exactly_one


def build_test_binary(target: str) -> Path:
    result = subprocess.run(
        ["cargo", "test", "--locked", "-p", "lila-engine", "--test", target,
         "--no-run", "--message-format=json"],
        stdout=subprocess.PIPE, text=True, check=True,
    )
    executables = set()
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-message":
            print(message["message"].get("rendered", ""), end="", flush=True)
        if (message.get("reason") == "compiler-artifact"
                and message["target"]["name"] == target
                and message["profile"]["test"] and message.get("executable")):
            executables.add(message["executable"])
    if len(executables) != 1:
        raise ValueError(f"expected one {target} executable, found {executables}")
    return Path(executables.pop())


def run_inventory(binary: Path, output_dir: Path, timeout: int) -> dict:
    if timeout <= 0:
        raise ValueError("timeout must be positive")
    inventory = subprocess.run(
        [str(binary), "--list"], check=True, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True,
    ).stdout
    names = parse_inventory(inventory)
    output_dir.mkdir(parents=True, exist_ok=True)
    (output_dir / "inventory.txt").write_text(inventory)
    results = []
    print(f"Complete compiled inventory: {len(names)} tests", flush=True)
    for name in names:
        print(f"::group::{name}", flush=True)
        try:
            result = subprocess.run(
                [str(binary), name, "--exact", "--nocapture", "--test-threads=1"],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                text=True, timeout=timeout,
            )
            output = result.stdout
            passed = result.returncode == 0 and passed_exactly_one(output, len(names))
            status = "passed" if passed else "failed"
        except subprocess.TimeoutExpired as error:
            output = error.stdout or ""
            if isinstance(output, bytes):
                output = output.decode(errors="replace")
            output += f"\nTimed out after {timeout}s: {name}\n"
            status = "timeout"
        print(output, end="", flush=True)
        print("::endgroup::", flush=True)
        log_name = hashlib.sha256(name.encode()).hexdigest() + ".txt"
        (output_dir / log_name).write_text(output)
        results.append({"name": name, "status": status, "log": log_name})
    summary = {"total": len(names), "passed": sum(r["status"] == "passed" for r in results),
               "results": results}
    (output_dir / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(f"Inventory complete: {summary['passed']}/{summary['total']} passed", flush=True)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", help="complete lila-engine integration test target")
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=600, help="seconds per test")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("timeout must be positive")
    summary = run_inventory(build_test_binary(args.target), args.output_dir, args.timeout)
    return int(summary["passed"] != summary["total"])


if __name__ == "__main__":
    raise SystemExit(main())
