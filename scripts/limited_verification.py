#!/usr/bin/env python3
"""Run verification under a confirmed cgroup-v2 process-tree memory cap.

Local scopes use at most 4096 MiB, no swap and grouped OOM termination.
An explicit --cloud invocation uses the managed machine's inherited finite
cap instead. Cargo builds and test execution remain serial. Cloud callers may
opt into CPU-quota-bound native compilation with --cloud-cpus auto; explicit
payload worker counts are preserved.
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
CLOUD_RETAINED_MODULES = 2
CLOUD_RETAINED_MODULE_IMAGE_BYTES = 256 * MIB


def positive_mib(value: str) -> int:
    if not value.isascii() or not value.isdecimal() or not 0 < int(value) <= DEFAULT_MEMORY_MIB:
        raise argparse.ArgumentTypeError("memory MiB must be an integer from 1 to 4096")
    return int(value)


def inherited_limit(environment: dict, name: str, ceiling: int) -> int:
    supplied = environment.get(name)
    if supplied is None:
        return ceiling
    raw = supplied.strip()
    if not raw.isascii() or not raw.isdecimal() or len(raw) > 20 or int(raw) == 0:
        raise ValueError(f"{name} must be a positive decimal integer")
    return min(int(raw), ceiling)


def limited_environment(*, cloud_budget: int | None = None,
                        native_jobs: int | None = None) -> dict:
    environment = dict(os.environ)
    environment.update(CARGO_BUILD_JOBS="1", LILA_JOBS="1",
                       RUST_TEST_THREADS="1", RAYON_NUM_THREADS="1")
    if native_jobs is not None:
        if native_jobs < 1:
            raise ValueError("native compilation jobs must be positive")
        for name in ("LILA_JOBS", "RAYON_NUM_THREADS"):
            environment[name] = str(inherited_limit(os.environ, name, native_jobs))
        # run-watched invokes capped.sh, whose local default halves affinity.
        # The cloud quota already bounded this set; retain an explicit share.
        environment.setdefault("LILA_CPU_PERCENT", "100")
    # R's native image exceeds the local 64-MiB retention ceiling. A cloud
    # process may keep R and P together, still below a measured finite cap.
    entries = MAX_RETAINED_MODULES if cloud_budget is None else CLOUD_RETAINED_MODULES
    image_bytes = (MAX_RETAINED_MODULE_IMAGE_BYTES if cloud_budget is None else
                   min(CLOUD_RETAINED_MODULE_IMAGE_BYTES, max(1, cloud_budget // 8)))
    for name, ceiling in (
        ("LILA_MODULE_MEMORY_CACHE_ENTRIES", entries),
        ("LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES", image_bytes),
    ):
        environment[name] = str(inherited_limit(environment, name, ceiling))
    return environment


def kernel_group() -> Path:
    entries = Path("/proc/self/cgroup").read_text().splitlines()
    unified = [entry[3:] for entry in entries if entry.startswith("0::")]
    if len(unified) != 1 or not unified[0].startswith("/"):
        raise ValueError("a unified cgroup-v2 membership is required")
    relative = Path(unified[0].lstrip("/"))
    if ".." in relative.parts:
        raise ValueError("invalid cgroup-v2 membership")
    return Path("/sys/fs/cgroup") / relative


def serial_affinity() -> None:
    inherited = os.sched_getaffinity(0)
    if not inherited:
        raise ValueError("the inherited CPU set is empty")
    os.sched_setaffinity(0, {min(inherited)})
    if len(os.sched_getaffinity(0)) != 1:
        raise ValueError("serial inherited CPU affinity could not be established")


def require_kernel_budget(requested_bytes: int) -> None:
    group = kernel_group()
    limit = (group / "memory.max").read_text().strip()
    if not limit.isascii() or not limit.isdecimal() or not 0 < int(limit) <= requested_bytes:
        raise ValueError("the kernel memory.max does not enforce the requested budget")
    if (group / "memory.swap.max").read_text().strip() != "0":
        raise ValueError("the kernel memory.swap.max must be zero")
    if (group / "memory.oom.group").read_text().strip() != "1":
        raise ValueError("the kernel must terminate the whole group on memory exhaustion")
    serial_affinity()


def require_cloud_budget(*, serial: bool = True) -> int:
    """Inspect the actual inherited cap, including tighter visible ancestors."""
    root = Path("/sys/fs/cgroup")
    group = kernel_group()
    limits = []
    while True:
        limit = (group / "memory.max").read_text().strip()
        if limit != "max":
            if not limit.isascii() or not limit.isdecimal() or int(limit) <= 0:
                raise ValueError("invalid inherited cloud memory.max")
            limits.append(int(limit))
        if group == root:
            break
        group = group.parent
    if not limits:
        raise ValueError("a finite inherited cloud memory.max is required")
    if serial:
        serial_affinity()
    return min(limits)


def cloud_native_affinity() -> int:
    """Use whole CPUs within the inherited affinity and tightest CPU quota."""
    inherited = os.sched_getaffinity(0)
    if not inherited:
        raise ValueError("the inherited CPU set is empty")
    capacity = len(inherited)
    root = Path("/sys/fs/cgroup")
    group = kernel_group()
    while True:
        fields = (group / "cpu.max").read_text().split()
        if len(fields) != 2:
            raise ValueError("invalid inherited cloud cpu.max")
        quota, period = fields
        if not period.isascii() or not period.isdecimal() or int(period) <= 0:
            raise ValueError("invalid inherited cloud cpu.max period")
        if quota != "max":
            if not quota.isascii() or not quota.isdecimal() or int(quota) <= 0:
                raise ValueError("invalid inherited cloud cpu.max quota")
            capacity = min(capacity, max(1, int(quota) // int(period)))
        if group == root:
            break
        group = group.parent
    for name in ("LILA_JOBS", "RAYON_NUM_THREADS"):
        capacity = inherited_limit(os.environ, name, capacity)
    selected = set(sorted(inherited)[:capacity])
    os.sched_setaffinity(0, selected)
    if os.sched_getaffinity(0) != selected:
        raise ValueError("cloud native compilation CPU affinity could not be established")
    return len(selected)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--memory-mib", type=positive_mib,
                        help="local scope budget in MiB (default: 4096)")
    parser.add_argument("--cloud", action="store_true",
                        help="use the managed cloud machine's inherited finite memory cap")
    parser.add_argument("--cloud-cpus", choices=("auto",),
                        help="opt into quota-bound cloud native compilation; tests and Cargo builds stay serial")
    parser.add_argument("--in-scope", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if not args.command or args.command[0] != "--" or len(args.command) == 1:
        parser.error("a command is required after --")
    command = args.command[1:]
    if args.cloud and (args.memory_mib is not None or args.in_scope):
        parser.error("--cloud cannot be combined with local scope options")
    if args.cloud_cpus is not None and not args.cloud:
        parser.error("--cloud-cpus requires --cloud")
    memory_mib = args.memory_mib or DEFAULT_MEMORY_MIB
    budget = memory_mib * MIB
    try:
        if not sys.platform.startswith("linux"):
            raise ValueError("Linux cgroup-v2 memory control is required; verification was not started")
        if args.cloud:
            cap = require_cloud_budget(serial=args.cloud_cpus is None)
            native_jobs = cloud_native_affinity() if args.cloud_cpus is not None else None
            environment = limited_environment(cloud_budget=cap, native_jobs=native_jobs)
            print(f"limited-verification: confirmed inherited cloud memory cap {cap} bytes, "
                  f"{native_jobs or 1} CPU(s), serial Cargo/test defaults, "
                  f"native compilation defaults up to {environment['LILA_JOBS']} jobs; retained module cache "
                  f"{environment['LILA_MODULE_MEMORY_CACHE_ENTRIES']} entries / "
                  f"{environment['LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES']} image bytes",
                  file=sys.stderr, flush=True)
            os.execvpe(command[0], command, environment)
        environment = limited_environment()
        if args.in_scope:
            require_kernel_budget(budget)
            print(f"limited-verification: confirmed kernel memory cap {memory_mib} MiB, "
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
                      "--memory-mib", str(memory_mib), "--", *command]
        os.execvpe(manager, invocation, environment)
    except (OSError, ValueError) as error:
        print(f"limited-verification: {error}; no uncapped fallback", file=sys.stderr)
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
