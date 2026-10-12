"""CPU ownership checks for a pollable serial Cargo build."""

import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parents[1] / "workspace_build_cpus.py"
spec = importlib.util.spec_from_file_location("workspace_build_cpus", SOURCE)
cpus = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cpus)


class BuildCpuTests(unittest.TestCase):
    def make_task(self, root, pid, tid, children=""):
        path = root / str(pid) / "task" / str(tid)
        path.mkdir(parents=True)
        (path / "children").write_text(children)

    def test_only_build_descendants_and_their_threads_receive_idle_cpus(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_task(root, 10, 10, "20")
            self.make_task(root, 10, 11, "30")
            self.make_task(root, 20, 20)
            self.make_task(root, 30, 30)
            self.make_task(root, 99, 99)
            masks = {tid: {1} for tid in [10, 11, 20, 30, 99]}
            with patch.object(cpus.os, "sched_getaffinity", side_effect=lambda tid: masks[tid]), \
                    patch.object(cpus.os, "sched_setaffinity", side_effect=lambda tid, mask: masks.__setitem__(tid, set(mask))):
                changes = cpus.expand_build_tree(10, {0, 1, 2, 3}, root)
            self.assertEqual({row["tid"] for row in changes}, {10, 11, 20, 30})
            self.assertEqual(masks[99], {1})
            self.assertTrue(all(row["after"] == [0, 1, 2, 3] for row in changes))

    def test_unexpected_or_already_complete_masks_are_never_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for tid in [10, 11, 12]:
                self.make_task(root, 10, tid)
            masks = {10: {0, 1}, 11: {7}, 12: {0, 7}}
            with patch.object(cpus.os, "sched_getaffinity", side_effect=lambda tid: masks[tid]), \
                    patch.object(cpus.os, "sched_setaffinity") as change:
                self.assertEqual(cpus.expand_build_tree(10, {0, 1}, root), [])
            change.assert_not_called()

    def test_exited_children_and_threads_do_not_fail_a_build(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_task(root, 10, 10, "20")
            with patch.object(cpus.os, "sched_getaffinity", side_effect=ProcessLookupError), \
                    patch.object(cpus.os, "sched_setaffinity") as change:
                self.assertEqual(cpus.expand_build_tree(10, {0, 1}, root), [])
            change.assert_not_called()

    def test_kernels_without_children_files_use_exact_parent_ownership(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for pid, parent in [(10, 1), (20, 10), (30, 20), (99, 1)]:
                self.make_task(root, pid, pid)
                (root / str(pid) / "task" / str(pid) / "children").unlink()
                (root / str(pid) / "status").write_text(f"Pid:\t{pid}\nPPid:\t{parent}\n")
            masks = {pid: {1} for pid in [10, 20, 30, 99]}
            with patch.object(cpus.os, "sched_getaffinity", side_effect=lambda tid: masks[tid]), \
                    patch.object(cpus.os, "sched_setaffinity", side_effect=lambda tid, mask: masks.__setitem__(tid, set(mask))):
                changes = cpus.expand_build_tree(10, {0, 1}, root)
            self.assertEqual({row["pid"] for row in changes}, {10, 20, 30})
            self.assertEqual(masks[99], {1})

    def test_an_empty_budget_is_rejected_before_any_process_is_touched(self):
        with patch.object(cpus.os, "sched_setaffinity") as change:
            with self.assertRaisesRegex(ValueError, "nonempty"):
                cpus.expand_build_tree(10, set())
        change.assert_not_called()


if __name__ == "__main__":
    unittest.main()
