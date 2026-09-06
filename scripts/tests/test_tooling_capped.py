"""T00: CPU caps narrow inherited affinity rather than assuming CPU zero."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "capped.sh"


class CpuCapTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        (self.bin / "awk").symlink_to(shutil.which("awk"))
        self.executable("getconf", "print('8')\n")
        self.executable("taskset", '''
import json, os, sys
args = sys.argv[1:]
if args[0] == "-pc":
    if os.environ.get("QUERY_FAIL"):
        sys.exit(1)
    print("pid " + args[1] + "'s current affinity list: " + os.environ.get("AFFINITY", "8-15"))
else:
    with open("selection.json", "w") as stream:
        json.dump(args, stream)
    os.execvp(args[2], args[2:])
''')
        self.executable("probe", '''
import json, os, sys
print(json.dumps({"args": sys.argv[1:], "jobs": os.environ["CARGO_BUILD_JOBS"]}))
sys.exit(int(os.environ.get("PROBE_STATUS", "0")))
''')

    def executable(self, name, source):
        path = self.bin / name
        path.write_text(f"#!{sys.executable} -S\n" + source)
        path.chmod(0o755)

    def run_cap(self, *args, **settings):
        env = dict(os.environ, PATH=str(self.bin), LILA_CPU_PERCENT="50")
        env.update(settings)
        return subprocess.run(["/bin/sh", str(SCRIPT), "probe", *args], cwd=self.root,
                              env=env, capture_output=True, text=True, timeout=10)

    def selection(self):
        return json.loads((self.root / "selection.json").read_text())

    def test_sparse_affinity_is_narrowed_and_jobs_match(self):
        result = self.run_cap(AFFINITY="2,4-6,10")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.selection()[:2], ["-c", "2,4"])
        self.assertEqual(json.loads(result.stdout)["jobs"], "2")

    def test_single_cpu_and_full_share_stay_inside_original_set(self):
        for affinity, percent, wanted, jobs in (("7", "1", "7", "1"),
                                                ("4,7-8", "100", "4,7-8", "3")):
            with self.subTest(affinity=affinity, percent=percent):
                result = self.run_cap(AFFINITY=affinity, LILA_CPU_PERCENT=percent)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.selection()[1], wanted)
                self.assertEqual(json.loads(result.stdout)["jobs"], jobs)

    def test_invalid_percentages_fail_before_execution(self):
        for percent in ("0", "00", "08", "101", "-1", "1+1", "9x", "99999999999999999999999"):
            with self.subTest(percent=percent):
                self.assertEqual(self.run_cap(LILA_CPU_PERCENT=percent).returncode, 2)
                self.assertFalse((self.root / "selection.json").exists())

    def test_affinity_query_failure_never_executes_command(self):
        result = self.run_cap(QUERY_FAIL="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cannot read inherited", result.stderr)
        self.assertFalse((self.root / "selection.json").exists())

    def test_malformed_affinity_is_rejected(self):
        for affinity in ("", "bad", "4-2", "2,,3", "3,2", "2-4,4"):
            with self.subTest(affinity=affinity):
                self.assertNotEqual(self.run_cap(AFFINITY=affinity).returncode, 0)
                self.assertFalse((self.root / "selection.json").exists())

    def test_non_linux_fallback_is_explicit(self):
        (self.bin / "taskset").unlink()
        result = self.run_cap()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["jobs"], "4")
        self.assertIn("limiting job counts only", result.stderr)

    def test_command_arguments_and_failure_status_are_preserved(self):
        result = self.run_cap("two words", "*", PROBE_STATUS="17")
        self.assertEqual(result.returncode, 17)
        self.assertEqual(json.loads(result.stdout)["args"], ["two words", "*"])

    @unittest.skipUnless(hasattr(os, "sched_getaffinity") and shutil.which("taskset"),
                         "real affinity integration requires Linux/taskset")
    def test_real_restricted_affinity(self):
        available = sorted(os.sched_getaffinity(0))[-2:]
        inherited = ",".join(map(str, available))
        result = subprocess.run([shutil.which("taskset"), "-c", inherited, "/bin/sh", str(SCRIPT),
                                 sys.executable, "-S", "-c",
                                 "import os; print(','.join(map(str, sorted(os.sched_getaffinity(0)))))"],
                                env=dict(os.environ, LILA_CPU_PERCENT="50"),
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(available[0]))


if __name__ == "__main__":
    unittest.main()
