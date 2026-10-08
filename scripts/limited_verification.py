#!/usr/bin/env python3
"""Run later verification under an inherited cgroup-v2 memory budget.

This never runs verification without a confirmed kernel process-tree cap.
The maximum and default budget is 4096 MiB with no swap; worker defaults and CPU affinity
are serial. It is a launcher, not permission to verify during source authoring.
"""

import argparse
import os
from pathlib import Path
import shutil
import sys


MIB = 1024 * 1024
DEFAULT_MEMORY_MIB = 4096
MAX_RETAINED_MODULES = 1
MAX_RETAINED_MODULE_IMAGE_BYTES = 64 * MIB


def positive_mib(value: str) -> int:
    if not value.isascii() or not value.isdecimal() or not 0 < int(value) <= DEFAULT_MEMORY_MIB:
        raise argparse.ArgumentTypeError("memory MiB must be an integer from 1 to 4096")
    return int(value)


def limited_environment() -> dict:
    environment = dict(os.environ)
    environment.update(CARGO_BUILD_JOBS="1", LILA_JOBS="1",
                       RUST_TEST_THREADS="1", RAYON_NUM_THREADS="1")
    for name, ceiling in (
        ("LILA_MODULE_MEMORY_CACHE_ENTRIES", MAX_RETAINED_MODULES),
        ("LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES", MAX_RETAINED_MODULE_IMAGE_BYTES),
    ):
        supplied = environment.get(name)
        if supplied is None:
            value = ceiling
        else:
            raw = supplied.strip()
            if not raw.isascii() or not raw.isdecimal() or len(raw) > 20 or int(raw) == 0:
                raise ValueError(f"{name} must be a positive decimal integer")
            value = min(int(raw), ceiling)
        environment[name] = str(value)
    return environment


def require_kernel_budget(requested_bytes: int) -> None:
    entries = Path("/proc/self/cgroup").read_text().splitlines()
    unified = [entry[3:] for entry in entries if entry.startswith("0::")]
    if len(unified) != 1 or not unified[0].startswith("/"):
        raise ValueError("a unified cgroup-v2 membership is required")
    relative = Path(unified[0].lstrip("/"))
    if ".." in relative.parts:
        raise ValueError("invalid cgroup-v2 membership")
    group = Path("/sys/fs/cgroup") / relative
    limit = (group / "memory.max").read_text().strip()
    if not limit.isascii() or not limit.isdecimal() or not 0 < int(limit) <= requested_bytes:
        raise ValueError("the kernel memory.max does not enforce the requested budget")
    if (group / "memory.swap.max").read_text().strip() != "0":
        raise ValueError("the kernel memory.swap.max must be zero")
    if (group / "memory.oom.group").read_text().strip() != "1":
        raise ValueError("the kernel must terminate the whole group on memory exhaustion")
    inherited = os.sched_getaffinity(0)
    if not inherited:
        raise ValueError("the inherited CPU set is empty")
    os.sched_setaffinity(0, {min(inherited)})
    if len(os.sched_getaffinity(0)) != 1:
        raise ValueError("serial inherited CPU affinity could not be established")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--memory-mib", type=positive_mib, default=DEFAULT_MEMORY_MIB)
    parser.add_argument("--in-scope", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if not args.command or args.command[0] != "--" or len(args.command) == 1:
        parser.error("a command is required after --")
    command = args.command[1:]
    budget = args.memory_mib * MIB
    try:
        environment = limited_environment()
        if not sys.platform.startswith("linux"):
            raise ValueError("Linux cgroup-v2 memory control is required; verification was not started")
        if args.in_scope:
            require_kernel_budget(budget)
            print(f"limited-verification: confirmed kernel memory cap {args.memory_mib} MiB, "
                  "swap 0, one CPU and serial worker defaults; retained module cache "
                  f"{environment['LILA_MODULE_MEMORY_CACHE_ENTRIES']} entry / "
                  f"{environment['LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES']} image bytes",
                  file=sys.stderr, flush=True)
            os.execvpe(command[0], command, environment)
        manager = shutil.which("systemd-run")
        if manager is None:
            raise ValueError("systemd-run is required; verification was not started")
        invocation = [manager, "--user", "--scope", "--collect", "--quiet",
                      "--no-ask-password", "--expand-environment=no",
                      f"--property=MemoryMax={budget}", "--property=MemorySwapMax=0",
                      "--property=OOMPolicy=kill", "--", sys.executable,
                      str(Path(__file__).resolve()), "--in-scope",
                      "--memory-mib", str(args.memory_mib), "--", *command]
        os.execvpe(manager, invocation, environment)
    except (OSError, ValueError) as error:
        print(f"limited-verification: {error}; no uncapped fallback", file=sys.stderr)
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
