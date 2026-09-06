#!/usr/bin/env python3
"""T00: supervise a command's POSIX process group with a monotonic stall limit."""

import argparse
from collections import deque
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time


def positive_seconds(value: str) -> int:
    if not value.isascii() or not value.isdecimal() or not 0 < int(value) <= 2147483647:
        raise argparse.ArgumentTypeError("seconds must be an integer from 1 to 2147483647")
    return int(value)


def tail(path: Path, count: int) -> str:
    # Read only a bounded suffix; compiler logs can be gigabytes long.
    with path.open("rb") as stream:
        stream.seek(0, os.SEEK_END)
        stream.seek(max(0, stream.tell() - 8192))
        return "\n".join(deque(stream.read().decode(errors="replace").splitlines(), maxlen=count))


def terminate_group(process: subprocess.Popen, grace: float = 5.0) -> None:
    """TERM then KILL the owned group, even when its leader already exited."""
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        process.wait()
        return
    deadline = time.monotonic() + grace
    while time.monotonic() < deadline:
        process.poll()  # Reap the leader without mistaking it for the whole tree.
        try:
            os.killpg(process.pid, 0)
        except ProcessLookupError:
            process.wait()
            return
        time.sleep(0.05)
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait()


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--label", default="run")
    parser.add_argument("--stall", type=positive_seconds, default=300)
    parser.add_argument("--poll", type=positive_seconds, default=15)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if not args.command or args.command[0] != "--" or len(args.command) == 1:
        parser.error("a command is required after --")
    if not args.label or Path(args.label).name != args.label or args.label in (".", ".."):
        parser.error("--label must be a filename, not a path")
    command = args.command[1:]
    capped = Path("./scripts/capped.sh")
    if os.access(capped, os.X_OK):
        command = [str(capped.resolve()), *command]
    log = Path("target/watched") / f"{args.label}.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    print(f"run-watched: {' '.join(args.command[1:])}", flush=True)
    print(f"run-watched: log {log}, stall limit {args.stall}s", flush=True)

    interrupted = 0
    stop = threading.Event()

    def request_stop(signum, _frame):
        nonlocal interrupted
        if not interrupted:
            interrupted = signum
        stop.set()

    previous_handlers = {sig: signal.signal(sig, request_stop)
                         for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)}
    process = None
    started = time.monotonic()
    status = 1
    try:
        with log.open("wb") as output:
            process = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT,
                                       start_new_session=True)
            last_growth = started
            previous_size = 0
            while True:
                result = process.poll()
                if interrupted:
                    status = 128 + interrupted
                    break
                if result is not None:
                    status = result if result >= 0 else 128 - result
                    break
                stop.wait(args.poll)
                # Completion wins over a stall at the same polling boundary.
                if interrupted or process.poll() is not None:
                    continue
                now = time.monotonic()
                size = log.stat().st_size
                if size > previous_size:
                    last_growth = now
                    print(f"run-watched: {int(now - started)}s elapsed, {size} bytes, "
                          f"last: {tail(log, 1)[:100]}", flush=True)
                previous_size = size
                if now - last_growth >= args.stall:
                    print(f"run-watched: STALLED — no output for {int(now - last_growth)}s. Killing group.",
                          file=sys.stderr, flush=True)
                    print(tail(log, 5), file=sys.stderr, flush=True)
                    status = 124
                    break
    except OSError as error:
        print(f"run-watched: {error}", file=sys.stderr, flush=True)
        status = 127 if process is None else 1
    finally:
        try:
            if process is not None:
                terminate_group(process)
        finally:
            for sig, handler in previous_handlers.items():
                signal.signal(sig, handler)
    print(f"run-watched: finished in {int(time.monotonic() - started)}s "
          f"with status {status} ({log})", flush=True)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
