#!/usr/bin/env python3
"""Run a disjoint share of the complete compiled AOT unit-test inventory.

Each test runs in a fresh process to bound retained compiler memory. A failed,
ignored, missing, or timed-out test fails the shard; no failure list is applied.
"""

import argparse
import json
import re
import subprocess
from pathlib import Path


def parse_inventory(output: str) -> list[str]:
    names = [line.removesuffix(": test") for line in output.splitlines() if line.endswith(": test")]
    totals = re.findall(r"(?m)^(\d+) tests?, (\d+) benchmarks?$", output)
    if len(totals) != 1 or int(totals[0][0]) != len(names) or int(totals[0][1]) != 0:
        raise ValueError("libtest inventory is incomplete or contains unexpected benchmarks")
    if not names or len(set(names)) != len(names):
        raise ValueError("libtest inventory must be nonempty and contain unique names")
    return sorted(names)


def select_shard(names: list[str], index: int, count: int) -> list[str]:
    if not 0 <= index < count <= len(names):
        raise ValueError("shard index/count must select a nonempty share of the inventory")
    return names[index::count]


def passed_exactly_one(output: str, total: int) -> bool:
    summaries = re.findall(
        r"(?m)^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; "
        r"(\d+) measured; (\d+) filtered out; finished in .+$", output
    )
    return summaries == [("1", "0", "0", "0", str(total - 1))]


def build_test_binary() -> Path:
    result = subprocess.run(
        ["cargo", "test", "--locked", "-p", "lila-aot-wasm", "--lib", "--no-run",
         "--message-format=json"],
        stdout=subprocess.PIPE, text=True,
    )
    executables = set()
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message.get("reason") == "compiler-message":
            print(message["message"].get("rendered", ""), end="", flush=True)
        if (message.get("reason") == "compiler-artifact"
                and message["target"]["name"] == "lila_aot_wasm"
                and message["profile"]["test"] and message.get("executable")):
            executables.add(message["executable"])
    result.check_returncode()
    if len(executables) != 1:
        raise ValueError(f"expected exactly one AOT test executable, found {executables}")
    return Path(executables.pop())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("index", type=int, help="zero-based shard index")
    parser.add_argument("count", type=int, help="number of disjoint shards")
    parser.add_argument("--timeout", type=int, default=600, help="seconds per test")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")

    binary = build_test_binary()
    inventory = subprocess.check_output([str(binary), "--list"], text=True)
    names = parse_inventory(inventory)
    selected = select_shard(names, args.index, args.count)
    print(f"Compiled inventory: {len(names)}; shard {args.index}/{args.count}: {len(selected)} tests", flush=True)
    failures = []
    for name in selected:
        print(f"::group::{name}", flush=True)
        try:
            result = subprocess.run(
                [str(binary), name, "--exact", "--nocapture", "--test-threads=1"],
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                timeout=args.timeout,
            )
            print(result.stdout, end="", flush=True)
            if result.returncode != 0 or not passed_exactly_one(result.stdout, len(names)):
                failures.append(name)
        except subprocess.TimeoutExpired as error:
            output = error.stdout or b""
            print(output.decode(errors="replace") if isinstance(output, bytes) else output, flush=True)
            print(f"Timed out after {args.timeout}s: {name}", flush=True)
            failures.append(name)
        finally:
            print("::endgroup::", flush=True)
    print(f"Shard complete: {len(selected) - len(failures)} passed, {len(failures)} failed", flush=True)
    for name in failures:
        print(f"FAILED: {name}", flush=True)
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
