"""Actual-child GNU-time accounting plus a nonce-bound checked-result pipe.

The ready clock starts before Popen and stops when this parent reads the
complete valid marker. Post-result child provenance work is still included in
whole-child wall/CPU/RSS. No duration is subtracted from another duration.
"""
import os
from pathlib import Path
import re
import secrets
import select
import signal
import subprocess
import tempfile
import threading
import time

READY_FD = 'SCRIPTW_BOUNDARY_READY_FD'
READY_NONCE = 'SCRIPTW_BOUNDARY_READY_NONCE'
MAX_MARKER_BYTES = 256


def decode_ready_marker(raw, nonce):
    match = re.fullmatch(rb'SWREADY1 ([0-9a-f]{32}) ([0-9a-f]{64})\n', raw)
    if match is None or match[1].decode('ascii') != nonce:
        raise ValueError('invalid, duplicate or wrong-nonce result-ready marker')
    return dict(nonce=nonce, stdout_sha256=match[2].decode('ascii'))


def _wait_for_ready_process(command, *, cwd, env, stdout, stderr, timeout, nonce):
    read_fd, write_fd = os.pipe()
    os.set_blocking(read_fd, False)
    environment = dict(env, **{READY_FD: str(write_fd), READY_NONCE: nonce})
    expired = threading.Event()
    observed = dict(nonce=nonce, status='missing', raw_hex='')
    started = time.perf_counter_ns()

    def collect():
        raw = bytearray()
        try:
            while True:
                readable, _, _ = select.select([read_fd], [], [], .1)
                if not readable:
                    continue
                chunk = os.read(read_fd, MAX_MARKER_BYTES + 1)
                if not chunk:
                    break
                raw.extend(chunk)
                if len(raw) > MAX_MARKER_BYTES:
                    observed.update(status='invalid', error='result-ready marker exceeded bound')
                    break
                if b'\n' in raw:
                    try:
                        marker = decode_ready_marker(bytes(raw), nonce)
                        if 'elapsed_ns' not in observed:
                            observed.update(marker, status='valid', elapsed_ns=time.perf_counter_ns() - started)
                    except ValueError as error:
                        observed.update(status='invalid', error=str(error))
                        break
            if raw and b'\n' not in raw:
                observed.update(status='invalid', error='truncated result-ready marker')
            observed['raw_hex'] = bytes(raw[:MAX_MARKER_BYTES]).hex()
        except Exception as error:
            observed.update(status='invalid', error=str(error))

    child = None
    watchdog = None
    try:
        child = subprocess.Popen(list(map(str, command)), cwd=cwd, env=environment,
            stdout=stdout, stderr=stderr, start_new_session=True, pass_fds=(write_fd,))
        os.close(write_fd)
        write_fd = None

        def stop():
            expired.set()
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass

        watchdog = threading.Timer(timeout, stop)
        watchdog.daemon = True
        watchdog.start()
        # Observe readiness on this parent thread before waiting for final
        # provenance and exit. A blocked/missing writer is bounded by watchdog.
        collect()
        try:
            _, status, usage = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
        except BaseException:
            stop()
            _, status, _ = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
            raise
        elapsed = time.perf_counter_ns() - started
        return dict(exit_code=child.returncode, elapsed_ns=elapsed,
            user_cpu_ns=round(usage.ru_utime * 1e9), system_cpu_ns=round(usage.ru_stime * 1e9),
            launch_tree_peak_rss_bytes=usage.ru_maxrss * 1024, timed_out=expired.is_set(),
            result_ready=observed, result_ready_elapsed_ns=observed.get('elapsed_ns'))
    except BaseException:
        if child is not None and child.returncode is None:
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            _, status, _ = os.wait4(child.pid, 0)
            child.returncode = os.waitstatus_to_exitcode(status)
        raise
    finally:
        if watchdog is not None:
            watchdog.cancel()
            watchdog.join()
        if write_fd is not None:
            os.close(write_fd)
        os.close(read_fd)


def measure_ready_process(command, *, cwd, env, stdout, stderr, timeout, time_binary):
    if not hasattr(os, 'wait4') or not Path('/proc/self/status').exists() or timeout <= 0:
        raise ValueError('ready accounting requires Linux wait4, /proc and a positive deadline')
    nonce = secrets.token_hex(16)
    with tempfile.NamedTemporaryFile() as accounting:
        result = _wait_for_ready_process(
            [str(time_binary), '-q', '-f', '%M', '-o', accounting.name, '--', *map(str, command)],
            cwd=cwd, env=env, stdout=stdout, stderr=stderr, timeout=timeout, nonce=nonce)
        raw = Path(accounting.name).read_text().strip()
    result['peak_rss_bytes'] = int(raw) * 1024 if raw.isdecimal() else None
    result['child_peak_rss_kib'] = int(raw) if raw.isdecimal() else None
    result['memory_scope'] = 'runtime-child-lifetime-high-water-rss;excludes-python-launcher'
    result['cpu_scope'] = 'gnu-time-supervisor-and-runtime-child'
    result['ready_scope'] = 'parent-before-Popen to complete nonce-bound checked-result marker arrival;includes-supervisor-and-launch;excludes-post-result-provenance;no-subtraction'
    if result['exit_code'] == 0 and result['peak_rss_bytes'] is None:
        raise ValueError('GNU time did not produce actual-child RSS accounting')
    return result


def validate_ready(accounting, child, expected_sha256):
    observation = accounting.get('result_ready', {})
    marker = child.get('result_ready')
    if (observation.get('status') != 'valid' or not isinstance(marker, dict)
            or marker.get('nonce') != observation.get('nonce')
            or marker.get('stdout_sha256') != expected_sha256
            or observation.get('stdout_sha256') != expected_sha256):
        raise ValueError('missing, corrupt or unbound checked-result ready marker')
    elapsed = accounting.get('result_ready_elapsed_ns')
    if (type(elapsed) is not int or elapsed < 0 or elapsed > accounting['elapsed_ns']
            or accounting.get('timed_out') or accounting.get('exit_code') != 0):
        raise ValueError('invalid or failed result-ready process observation')
