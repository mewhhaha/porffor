"""Release a finished test's CPU restriction on the existing Cargo process tree."""

import os
from pathlib import Path


def expand_build_tree(pid, cpus, proc_root=Path("/proc")):
    """Expand only descendant task masks already contained in the allowed budget.

    The caller owns the build child and ensures all native wrappers have exited.
    Cargo stays serial. A task with a different mask is left alone; vanished
    tasks are normal when rustc finishes. No native test process is visited.
    """
    cpus = set(cpus)
    if not cpus:
        raise ValueError("the compilation CPU budget must remain nonempty")
    pending = [pid]
    seen = set()
    changed = []
    parents = None
    while pending:
        process = pending.pop()
        if process in seen:
            continue
        seen.add(process)
        try:
            tasks = list((proc_root / str(process) / "task").iterdir())
        except FileNotFoundError:
            continue
        for task in tasks:
            try:
                try:
                    pending.extend(int(child) for child in (task / "children").read_text().split())
                except FileNotFoundError:
                    # Some kernels omit task/children. Take one parent snapshot
                    # for this transition, then walk only the build descendants.
                    if parents is None:
                        parents = {}
                        for entry in proc_root.iterdir():
                            if not entry.name.isdigit():
                                continue
                            try:
                                status = (entry / "status").read_text()
                                parent = next(line.split()[1] for line in status.splitlines()
                                              if line.startswith("PPid:"))
                                parents.setdefault(int(parent), []).append(int(entry.name))
                            except (FileNotFoundError, StopIteration):
                                continue
                    pending.extend(parents.get(process, []))
                tid = int(task.name)
                previous = os.sched_getaffinity(tid)
                if previous and previous < cpus:
                    os.sched_setaffinity(tid, cpus)
                    changed.append({"pid": process, "tid": tid,
                                    "before": sorted(previous),
                                    "after": sorted(os.sched_getaffinity(tid))})
            except (FileNotFoundError, ProcessLookupError):
                continue
    return changed
