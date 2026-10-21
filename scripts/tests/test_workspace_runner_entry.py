"""The workspace planner can be inspected without starting verification."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


RUNNER = Path(__file__).resolve().parents[1] / "run_workspace_tests.py"


class WorkspaceRunnerEntryTests(unittest.TestCase):
    def test_import_ignores_argv_and_does_not_start_processes_or_write_files(self):
        probe = """
import runpy, subprocess, sys
from pathlib import Path
def forbidden(*args, **kwargs):
    raise AssertionError('import started a process or wrote a file')
subprocess.Popen = forbidden
Path.mkdir = forbidden
Path.write_text = forbidden
runner = sys.argv[1]
sys.argv = ['unrelated-program', '--not-a-runner-option']
namespace = runpy.run_path(runner, run_name='imported_workspace_runner')
assert all(callable(namespace[name]) for name in
           ('parse_args', 'validate_args', 'plan_inventory', 'run', 'main'))
assert list(Path.cwd().iterdir()) == []
"""
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run([sys.executable, "-B", "-c", probe, str(RUNNER)],
                                    cwd=directory, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_planner_retains_feature_disabled_targets_and_exact_selection(self):
        spec = importlib.util.spec_from_file_location("workspace_runner", RUNNER)
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        packages = [{"name": "example", "id": "example-id", "features": {
            "default": ["base"], "base": [], "extra": []}, "targets": [
                {"name": "example", "kind": ["lib"], "test": True, "doctest": True},
                {"name": "base_case", "kind": ["test"], "test": True,
                 "required-features": ["base"]},
                {"name": "extra_case", "kind": ["test"], "test": True,
                 "required-features": ["extra"]}]}]
        metadata = {"packages": packages, "workspace_members": ["example-id"]}
        args = runner.parse_args(["--output", "unused"])
        filters = runner.validate_args(args)
        _, _, cases, disabled, full, _, _, _ = runner.plan_inventory(metadata, args, filters)
        self.assertEqual([case["target"] for case in cases], ["example", "base_case", "doc"])
        self.assertEqual(cases, full)
        self.assertEqual(disabled, [{"package": "example", "target": "extra_case",
                                     "missing_required_features": ["extra"]}])
        args.only_targets = ["example:base_case"]
        _, _, selected, _, full, _, _, _ = runner.plan_inventory(metadata, args, filters)
        self.assertEqual([case["target"] for case in selected], ["base_case"])
        self.assertEqual(len(full), 3)
