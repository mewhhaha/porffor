"""T00: stalled/cancelled verification cannot leave descendant workers running."""

import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "run-watched.sh"
TREE = '''
import os, subprocess, sys
from pathlib import Path
Path("parent.pid").write_text(str(os.getpid()))
child = subprocess.Popen([sys.executable, "-S", "-c", """
import os, signal, time
from pathlib import Path
signal.signal(signal.SIGTERM, signal.SIG_IGN)
Path('child.pid').write_text(str(os.getpid()))
time.sleep(60)
"""])
child.wait()
'''


def running(pid):
    result = subprocess.run(["ps", "-p", str(pid), "-o", "stat="], capture_output=True, text=True)
    return bool(result.stdout.strip()) and not result.stdout.strip().startswith("Z")


class WatchedCommandTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def invoke(self, source, *extra):
        return subprocess.run(["sh", str(SCRIPT), "--poll", "1", "--stall", "5", *extra,
                               "--", sys.executable, "-S", "-c", source], cwd=self.root,
                              capture_output=True, text=True, timeout=15)

    def start_tree(self, stall):
        process = subprocess.Popen(["sh", str(SCRIPT), "--poll", "1", "--stall", str(stall),
                                    "--", sys.executable, "-S", "-c", TREE], cwd=self.root,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                   start_new_session=True)

        def cleanup():
            # These PIDs belong only to this test's deliberately spawned tree.
            for name in ("child.pid", "parent.pid"):
                path = self.root / name
                if path.exists():
                    try:
                        os.kill(int(path.read_text()), signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
            process.communicate(timeout=5)

        self.addCleanup(cleanup)
        deadline = time.monotonic() + 8
        while not (self.root / "child.pid").exists() and time.monotonic() < deadline:
            self.assertIsNone(process.poll(), "watcher exited before the test tree was ready")
            time.sleep(0.02)
        self.assertTrue((self.root / "child.pid").exists())
        return process, int((self.root / "child.pid").read_text())

    def test_success_and_failure_both_report_exact_exit_status(self):
        for status in (0, 23):
            with self.subTest(status=status):
                result = self.invoke(f"import sys; print('payload'); sys.exit({status})")
                self.assertEqual(result.returncode, status, result.stderr)
                self.assertIn(f"with status {status}", result.stdout)
                self.assertIn("payload", (self.root / "target/watched/run.log").read_text())

    def test_stall_kills_term_resistant_descendant(self):
        process, child = self.start_tree(stall=2)
        stdout, stderr = process.communicate(timeout=15)
        self.assertEqual(process.returncode, 124, (stdout, stderr))
        self.assertIn("STALLED", stderr)
        self.assertFalse(running(child), "stalled descendant survived the guard")

    def test_cancellation_kills_descendants_and_returns_signal_status(self):
        process, child = self.start_tree(stall=30)
        process.send_signal(signal.SIGTERM)
        stdout, stderr = process.communicate(timeout=15)
        self.assertEqual(process.returncode, 143, (stdout, stderr))
        self.assertFalse(running(child), "cancelled descendant survived the guard")

    def test_completed_command_is_not_misclassified_as_stalled(self):
        result = self.invoke("import time; time.sleep(0.2)", "--stall", "1")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn("STALLED", result.stderr)

    def test_cpu_cap_is_still_used(self):
        capped = self.root / "scripts/capped.sh"
        capped.parent.mkdir()
        capped.write_text('#!/bin/sh\nprintf "cap applied\\n"\nexec "$@"\n')
        capped.chmod(0o755)
        result = self.invoke("print('payload')")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.root / "target/watched/run.log").read_text(), "cap applied\npayload\n")

    def test_invalid_options_never_start_a_command(self):
        for options in (("--poll", "0"), ("--stall", "-1"), ("--stall", "x"),
                        ("--label", "../escape")):
            with self.subTest(options=options):
                result = self.invoke("from pathlib import Path; Path('started').touch()", *options)
                self.assertEqual(result.returncode, 2)
                self.assertFalse((self.root / "started").exists())

    def test_missing_command_is_reported_without_traceback(self):
        result = subprocess.run(["sh", str(SCRIPT), "--", "./does-not-exist"], cwd=self.root,
                                capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 127)
        self.assertNotIn("Traceback", result.stderr)


if __name__ == "__main__":
    unittest.main()
