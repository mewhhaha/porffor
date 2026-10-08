"""Resource launcher controls: absent or ineffective caps never start payloads."""

from contextlib import ExitStack
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch


SOURCE = Path(__file__).resolve().parents[1] / "limited_verification.py"
spec = importlib.util.spec_from_file_location("limited_verification", SOURCE)
launcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(launcher)


class PayloadWouldStart(Exception):
    pass


class KernelBudgetTests(unittest.TestCase):
    def scope(self, memory="4294967296", swap="0", group="1"):
        files = {"/proc/self/cgroup": "0::/test.scope\n",
                 "/sys/fs/cgroup/test.scope/memory.max": memory,
                 "/sys/fs/cgroup/test.scope/memory.swap.max": swap,
                 "/sys/fs/cgroup/test.scope/memory.oom.group": group}
        stack = ExitStack()
        stack.enter_context(patch.object(launcher.sys, "platform", "linux"))
        stack.enter_context(patch.object(launcher.Path, "read_text",
                                        lambda path: files[str(path)]))
        affinity = [{2, 4, 6}]
        stack.enter_context(patch.object(launcher.os, "sched_getaffinity",
                                        lambda pid: affinity[0], create=True))
        stack.enter_context(patch.object(launcher.os, "sched_setaffinity",
                                        lambda pid, cpus: affinity.__setitem__(0, cpus), create=True))
        return stack

    def test_ineffective_budget_never_executes_payload(self):
        for settings in ({"memory": "max"}, {"memory": "4294967297"},
                         {"memory": "0"}, {"swap": "1"}, {"group": "0"}):
            with self.subTest(settings=settings), self.scope(**settings), \
                    patch.object(launcher.os, "execvpe") as execute:
                self.assertEqual(launcher.main(["--in-scope", "--", "payload"]), 2)
                execute.assert_not_called()

    def test_confirmed_budget_preserves_argv_and_serializes_descendants(self):
        argv = ["payload", "two words", "$HOME", "*"]
        with self.scope(), patch.object(launcher.os, "execvpe",
                                        side_effect=PayloadWouldStart) as execute:
            with self.assertRaises(PayloadWouldStart):
                launcher.main(["--in-scope", "--", *argv])
            command, arguments, environment = execute.call_args.args
            self.assertEqual(command, "payload")
            self.assertEqual(arguments, argv)
            self.assertEqual(launcher.os.sched_getaffinity(0), {2})
            for setting in ("CARGO_BUILD_JOBS", "LILA_JOBS", "RUST_TEST_THREADS", "RAYON_NUM_THREADS"):
                self.assertEqual(environment[setting], "1")
            self.assertEqual(environment["LILA_MODULE_MEMORY_CACHE_ENTRIES"], "1")
            self.assertLessEqual(int(environment["LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES"]), 64 * 1024 * 1024)

    def test_large_inherited_caches_are_capped_and_stricter_image_limits_survive(self):
        for inherited, expected in (("536870912", "67108864"), ("1048576", "1048576")):
            with self.subTest(inherited=inherited), self.scope(), \
                    patch.dict(launcher.os.environ, {
                        "LILA_MODULE_MEMORY_CACHE_ENTRIES": "64",
                        "LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES": inherited,
                    }), patch.object(launcher.os, "execvpe", side_effect=PayloadWouldStart) as execute:
                with self.assertRaises(PayloadWouldStart):
                    launcher.main(["--in-scope", "--", "payload"])
                environment = execute.call_args.args[2]
                self.assertEqual(environment["LILA_MODULE_MEMORY_CACHE_ENTRIES"], "1")
                self.assertEqual(environment["LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES"], expected)

    def test_invalid_cache_limits_never_reach_engine_fallback_defaults(self):
        for setting in ("LILA_MODULE_MEMORY_CACHE_ENTRIES", "LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES"):
            for invalid in ("0", "-1", "unlimited", "1.5", "", "9" * 21):
                with self.subTest(setting=setting, invalid=invalid), \
                        patch.dict(launcher.os.environ, {setting: invalid}), \
                        patch.object(launcher.shutil, "which") as manager, \
                        patch.object(launcher.os, "execvpe") as execute:
                    self.assertEqual(launcher.main(["--", "payload"]), 2)
                    manager.assert_not_called()
                    execute.assert_not_called()

    def test_missing_scope_manager_has_no_uncapped_fallback(self):
        with patch.object(launcher.sys, "platform", "linux"), \
                patch.object(launcher.shutil, "which", return_value=None), \
                patch.object(launcher.os, "execvpe") as execute:
            self.assertEqual(launcher.main(["--", "payload"]), 2)
            execute.assert_not_called()

    def test_budget_override_above_four_gib_never_launches_a_scope(self):
        for amount in ("4097", "65536"):
            with self.subTest(amount=amount), \
                    patch.object(launcher.shutil, "which") as manager, \
                    patch.object(launcher.os, "execvpe") as execute:
                with self.assertRaises(SystemExit) as failure:
                    launcher.main(["--memory-mib", amount, "--", "payload"])
                self.assertEqual(failure.exception.code, 2)
                manager.assert_not_called()
                execute.assert_not_called()

    def test_scope_launch_requires_aggregate_kernel_properties(self):
        with patch.object(launcher.sys, "platform", "linux"), \
                patch.object(launcher.shutil, "which", return_value="/usr/bin/systemd-run"), \
                patch.object(launcher.os, "execvpe", side_effect=PayloadWouldStart) as execute:
            with self.assertRaises(PayloadWouldStart):
                launcher.main(["--", "payload", "two words", "$HOME", "*"])
            command, argv, _ = execute.call_args.args
            self.assertEqual(command, "/usr/bin/systemd-run")
            for setting in ("--scope", "--expand-environment=no", "--property=MemoryMax=4294967296",
                            "--property=MemorySwapMax=0", "--property=OOMPolicy=kill"):
                self.assertIn(setting, argv)
            self.assertEqual(argv[-5:], ["--", "payload", "two words", "$HOME", "*"])


if __name__ == "__main__":
    unittest.main()
