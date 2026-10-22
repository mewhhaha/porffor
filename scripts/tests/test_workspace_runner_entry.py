"""The workspace planner can be inspected without starting verification."""

import builtins
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import symtable
import tempfile
import unittest


RUNNER = Path(__file__).resolve().parents[1] / "run_workspace_tests.py"


class WorkspaceRunnerEntryTests(unittest.TestCase):
    @unittest.skipUnless(hasattr(os, "sched_getaffinity"), "Linux native CPU admission")
    def test_optimized_partition_driver_rejects_duplicate_cpu_before_output(self):
        cpu = str(min(os.sched_getaffinity(0)))
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "rejected"
            result = subprocess.run([
                sys.executable, "-O", "-B", str(RUNNER.with_name("workspace_test_partitions.py")),
                "--executable", "must-not-run", "--inventory", "missing-inventory",
                "--output", str(output), "--include-ignored", "--cpus", cpu, cpu,
            ], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("AssertionError", result.stderr)
            self.assertFalse(output.exists())

    def test_optimized_python_rejects_invalid_admission_before_creating_output(self):
        for flags in (
            ["--native-target-cpus", "auto"],
            ["--partition-cli"],
            ["--cli-partition-count", "4"],
            ["--partition-cli", "--include-ignored", "--test-processes", "2", "--cli-partition-count", "1"],
            ["--selected-feature-roots", "--all-features"],
        ):
            with self.subTest(flags=flags), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "rejected"
                result = subprocess.run(
                    [
                        sys.executable,
                        "-O",
                        "-B",
                        str(RUNNER),
                        "--output",
                        str(output),
                        *flags,
                    ],
                    capture_output=True,
                    text=True,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("ValueError", result.stderr)
                self.assertFalse(output.exists())

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
            result = subprocess.run(
                [sys.executable, "-B", "-c", probe, str(RUNNER)],
                cwd=directory,
                capture_output=True,
                text=True,
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_runtime_globals_remain_bound_after_extracting_helpers(self):
        module = symtable.symtable(RUNNER.read_text(), str(RUNNER), "exec")
        known = {
            symbol.get_name()
            for symbol in module.get_symbols()
            if symbol.is_assigned() or symbol.is_imported() or symbol.is_namespace()
        }
        known.update(dir(builtins))
        known.update(("__file__", "__name__"))
        unbound = []

        def inspect_scope(scope):
            for symbol in scope.get_symbols():
                if (
                    symbol.is_referenced()
                    and symbol.is_global()
                    and symbol.get_name() not in known
                ):
                    unbound.append(
                        (scope.get_name(), scope.get_lineno(), symbol.get_name())
                    )
            for child in scope.get_children():
                inspect_scope(child)

        inspect_scope(module)
        self.assertEqual(unbound, [])

    def test_planner_retains_feature_disabled_targets_and_exact_selection(self):
        spec = importlib.util.spec_from_file_location("workspace_runner", RUNNER)
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        packages = [
            {
                "name": "example",
                "id": "example-id",
                "features": {"default": ["base"], "base": [], "extra": []},
                "targets": [
                    {"name": "example", "kind": ["lib"], "test": True, "doctest": True},
                    {
                        "name": "base_case",
                        "kind": ["test"],
                        "test": True,
                        "required-features": ["base"],
                    },
                    {
                        "name": "extra_case",
                        "kind": ["test"],
                        "test": True,
                        "required-features": ["extra"],
                    },
                ],
            }
        ]
        metadata = {"packages": packages, "workspace_members": ["example-id"]}
        args = runner.parse_args(["--output", "unused"])
        filters = runner.validate_args(args)
        _, _, cases, disabled, full, _, _, _ = runner.plan_inventory(
            metadata, args, filters
        )
        self.assertEqual(
            [case["target"] for case in cases], ["example", "base_case", "doc"]
        )
        self.assertEqual(cases, full)
        self.assertEqual(
            disabled,
            [
                {
                    "package": "example",
                    "target": "extra_case",
                    "missing_required_features": ["extra"],
                }
            ],
        )
        args.only_targets = ["example:base_case"]
        _, _, selected, _, full, _, _, _ = runner.plan_inventory(
            metadata, args, filters
        )
        self.assertEqual([case["target"] for case in selected], ["base_case"])
        self.assertEqual(len(full), 3)
