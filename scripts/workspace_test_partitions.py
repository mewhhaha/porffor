"""Run every qualified native libtest ID once in disjoint processes."""

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

from workspace_test_schedule import native_stall_seconds
from native_partition_costs import measured_weight
from native_test_protocol import test_names, summary_counts


parser = argparse.ArgumentParser()
parser.add_argument("--executable", required=True, type=Path)
parser.add_argument("--inventory", required=True, type=Path)
parser.add_argument("--output", required=True, type=Path)
parser.add_argument("--cpus", required=True, nargs="+", type=int)
parser.add_argument("--partition-count", type=int, choices=range(1, 5))
parser.add_argument("--include-ignored", action="store_true", required=True)
parser.add_argument("--inventory-only", action="store_true")
parser.add_argument("--qualified-target", default="scriptw-cli:cli")
parser.add_argument("--stall-seconds", type=int, choices=(900, 3600))
args = parser.parse_args()
package, separator, target = args.qualified_target.partition(":")
if not (separator and package and target and ":" not in target):
    raise AssertionError("An exact package:target is required")
stock_stall = native_stall_seconds(package)
if not (
    args.stall_seconds
    in (
        None,
        stock_stall,
    )
):
    raise AssertionError("The original native watcher allowance cannot change")
args.stall_seconds = stock_stall
root = Path.cwd()
if not (1 <= len(args.cpus) <= 4):
    raise AssertionError("Verification condition failed: 1 <= len(args.cpus) <= 4")
if not (len(set(args.cpus)) == len(args.cpus)):
    raise AssertionError(
        "Verification condition failed: len(set(args.cpus)) == len(args.cpus)"
    )
if not (set(args.cpus) <= os.sched_getaffinity(0)):
    raise AssertionError(
        "Verification condition failed: set(args.cpus) <= os.sched_getaffinity(0)"
    )
partition_count = args.partition_count or len(args.cpus)
if partition_count < len(args.cpus):
    raise ValueError("Partition count cannot leave an admitted CPU unused")
args.output.mkdir(parents=True, exist_ok=False)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def names(text):
    return test_names(text)


def outcomes(text):
    starts = list(re.finditer(r"^test (.+?) \.\.\. ", text, re.M))
    results = {}
    for index, start in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
        block = text[start.end() : end].split("\nfailures:", 1)[0]
        terminal = re.findall(r"^(ok|FAILED|ignored)$", block, re.M)
        if not (len(terminal) == 1):
            raise AssertionError((start.group(1), terminal))
        name = start.group(1).removesuffix(" - should panic")
        if not (name not in results):
            raise AssertionError(name)
        results[name] = terminal[0]
    summaries = summary_counts(text)
    if not (len(summaries) == 1):
        raise AssertionError(summaries)
    counts = {
        key: sum(value == terminal for value in results.values())
        for key, terminal in [
            ("passed", "ok"),
            ("failed", "FAILED"),
            ("ignored", "ignored"),
        ]
    }
    if not (tuple(counts.values()) == tuple(map(int, summaries[0]))):
        raise AssertionError(
            "Verification condition failed: tuple(counts.values()) == tuple(map(int, summaries[0]))"
        )
    if not (counts["ignored"] == 0):
        raise AssertionError("Verification condition failed: counts['ignored'] == 0")
    return results, counts


def weight(name):
    # Scheduling estimates are never used to select, drop or retry a test.
    measured = measured_weight(name)
    if measured is not None:
        return measured
    if name.startswith("frontend_test262_subset::"):
        return 800
    if name.startswith("intl::"):
        return 25
    if name.startswith("frontend::") or name.startswith("known_failures::"):
        return 1
    return 3


def partition_groups(original, count):
    if not (original and len(original) == len(set(original))):
        raise AssertionError(
            "Verification condition failed: original and len(original) == len(set(original))"
        )
    if not (1 <= count <= min(4, len(original))):
        raise AssertionError("Every admitted native partition needs an ID")
    groups = [[] for _ in range(count)]
    loads = [0 for _ in range(count)]
    for name in sorted(original, key=lambda item: (-weight(item), item)):
        index = min(range(len(groups)), key=lambda group: (loads[group], group))
        groups[index].append(name)
        loads[index] += weight(name)
    if not (sorted(name for group in groups for name in group) == sorted(original)):
        raise AssertionError(
            "Verification condition failed: sorted((name for group in groups for name in group)) == sorted(original)"
        )
    if not (all(groups)):
        raise AssertionError("Verification condition failed: all(groups)")
    return groups


def admit_partition_rows(waiting, busy_cpus):
    """Take at most one waiting chunk per free CPU, preserving per-CPU order."""
    occupied = set(busy_cpus)
    ready, remaining = [], []
    for row in waiting:
        if row["cpu"] in occupied:
            remaining.append(row)
        else:
            ready.append(row)
            occupied.add(row["cpu"])
    return ready, remaining


def partition_label(output, index):
    # A target basename repeats across workspace runs, while watched logs live
    # in one shared directory. Bind each label to its complete output owner.
    output = Path(output).resolve()
    owner = hashlib.sha256(os.fsencode(output)).hexdigest()
    prefix = re.sub(r"[^A-Za-z0-9_.-]", "-", output.name)[:64] or "native"
    return f"{prefix}-{owner}-partition-{index}"


original = names(args.inventory.read_text())
if not (original and len(original) == len(set(original))):
    raise AssertionError(
        "Verification condition failed: original and len(original) == len(set(original))"
    )
checked = subprocess.run(
    [str(args.executable.resolve()), "--list"], capture_output=True, text=True
)
if not (checked.returncode == 0):
    raise AssertionError("Verification condition failed: checked.returncode == 0")
if not (names(checked.stdout) == original):
    raise AssertionError("The original complete native inventory changed")
full_inventory = args.output / "complete.inventory.log"
full_inventory.write_text(checked.stdout + checked.stderr)
groups = partition_groups(original, partition_count)
executable_sha = digest(args.executable)
receipt = {
    "policy": "Complete logical target: every original qualified ID exactly once in disjoint native libtest processes, each on one inherited CPU and one test thread; original bodies and deadlines unchanged; zero ignored required.",
    "driver_sha256": digest(Path(__file__)),
    "qualified_target": args.qualified_target,
    "stall_seconds": args.stall_seconds,
    "inherited_retained_modules": {
        "entries_per_process": os.environ.get("SCRIPTW_MODULE_MEMORY_CACHE_ENTRIES"),
        "image_bytes_per_process": os.environ.get(
            "SCRIPTW_MODULE_MEMORY_CACHE_LIMIT_BYTES"
        ),
        "policy": "Inherited environment; each child retains the existing limited_verification cloud ceiling. No cache or memory limit changed.",
    },
    "executable": str(args.executable),
    "executable_sha256": executable_sha,
    "original_inventory": str(args.inventory),
    "original_inventory_sha256": digest(args.inventory),
    "complete_inventory": str(full_inventory),
    "complete_inventory_sha256": digest(full_inventory),
    "complete_test_count": len(original),
    "cpus": args.cpus,
    "partition_count": partition_count,
    "maximum_native_processes": len(args.cpus),
    "partitions": [],
    "source_authority": "Outer workspace receipt binds the exact Cargo-selected executable and immutable source fence.",
}


def save():
    (args.output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")


# Validate every actual filtered inventory before starting any native test body.
for index, group in enumerate(groups):
    cpu = args.cpus[index % len(args.cpus)]
    group.sort()
    command = [str(args.executable.resolve()), "--exact", *group]
    listed = subprocess.run([*command, "--list"], capture_output=True, text=True)
    inventory = args.output / f"partition-{index}.inventory.log"
    inventory.write_text(listed.stdout + listed.stderr)
    if not (listed.returncode == 0 and names(listed.stdout) == group):
        raise AssertionError(
            "Verification condition failed: listed.returncode == 0 and names(listed.stdout) == group"
        )
    receipt["partitions"].append(
        {
            "index": index,
            "cpu": cpu,
            "state": "queued",
            "ids": group,
            "inventory_log": str(inventory),
            "inventory_sha256": digest(inventory),
            "command": [*command, "--test-threads=1", "--include-ignored"],
        }
    )
save()
if args.inventory_only:
    receipt["state"] = "inventory-only; no native test body executed or verdict claimed"
    save()
    print(
        json.dumps(
            {
                "complete_test_count": len(original),
                "partition_counts": [len(group) for group in groups],
                "exact_disjoint_union": True,
            },
            indent=2,
        )
    )
    raise SystemExit(0)
active = []
waiting = list(receipt["partitions"])
started = time.monotonic()
last_update = 0.0
while waiting or active:
    ready, waiting = admit_partition_rows(waiting, {row["cpu"] for _, _, row in active})
    for row in ready:
        label = partition_label(args.output, row["index"])
        log = args.output / f'partition-{row["index"]}.launcher.log'
        stream = log.open("w")
        launch = [
            "taskset",
            "-c",
            str(row["cpu"]),
            "./scripts/run-watched.sh",
            "--label",
            label,
            "--stall",
            str(args.stall_seconds),
            "--poll",
            "1",
            "--",
            "python3",
            "-B",
            "scripts/limited_verification.py",
            "--cloud",
            "--",
            *row["command"],
        ]
        row["launch_command"] = launch
        row["transcript_log"] = str(Path("target/watched") / (label + ".log"))
        row["state"] = "running"
        active.append(
            (subprocess.Popen(launch, stdout=stream, stderr=subprocess.STDOUT), stream, row)
        )
    save()
    now = time.monotonic()
    if now - last_update >= 20:
        print(
            "PARTITION PROGRESS",
            json.dumps(
                {
                    row["index"]: Path(row["transcript_log"]).stat().st_size
                    for _, _, row in active
                    if Path(row["transcript_log"]).exists()
                }
            ),
            flush=True,
        )
        last_update = now
    pending = []
    for process, stream, row in active:
        status = process.poll()
        if status is None:
            pending.append((process, stream, row))
            continue
        stream.close()
        row["exit_code"] = status
        transcript = Path(row["transcript_log"])
        if transcript.exists():
            row["transcript_sha256"] = digest(transcript)
        try:
            actual, counts = outcomes(transcript.read_text())
            if not (set(actual) == set(row["ids"])):
                raise AssertionError(
                    "Verification condition failed: set(actual) == set(row['ids'])"
                )
            row["native_outcomes"] = actual
            row["counts"] = counts
            row["inventory_matches"] = True
            row["state"] = (
                "passed" if status == 0 and counts["failed"] == 0 else "failed"
            )
        except (AssertionError, OSError) as error:
            row["state"] = "incomplete"
            row["inventory_matches"] = False
            row["validation_error"] = str(error)
        print(
            "PARTITION END", row["index"], row["state"], row.get("counts"), flush=True
        )
        save()
    active = pending
    if active:
        time.sleep(0.1)

if not (digest(args.executable) == executable_sha):
    raise AssertionError("Native artifact changed during execution")
receipt["seconds"] = round(time.monotonic() - started, 3)
receipt["inventory_matches"] = all(
    row["inventory_matches"] for row in receipt["partitions"]
)
if receipt["inventory_matches"]:
    actual = {
        name: result
        for row in receipt["partitions"]
        for name, result in row["native_outcomes"].items()
    }
    if not (len(actual) == len(original) and set(actual) == set(original)):
        raise AssertionError(
            "Verification condition failed: len(actual) == len(original) and set(actual) == set(original)"
        )
    receipt["counts"] = {
        key: sum(row["counts"][key] for row in receipt["partitions"])
        for key in ("passed", "failed", "ignored")
    }
save()
# A composite view retains all native result lines, with one recomputed summary.
# Original complete per-process transcripts are retained and hash-bound above.
for row in receipt["partitions"]:
    transcript = Path(row["transcript_log"])
    if transcript.exists():
        text = re.sub(r"^test result: .*\n?", "", transcript.read_text(), flags=re.M)
        print(text, end="\n", flush=True)
if receipt["inventory_matches"]:
    counts = receipt["counts"]
    state = (
        "ok"
        if all(row["state"] == "passed" for row in receipt["partitions"])
        else "FAILED"
    )
    print(
        f'test result: {state}. {counts["passed"]} passed; {counts["failed"]} failed; {counts["ignored"]} ignored; 0 measured; 0 filtered out;',
        flush=True,
    )
raise SystemExit(
    not receipt["inventory_matches"]
    or any(row["state"] != "passed" for row in receipt["partitions"])
)
