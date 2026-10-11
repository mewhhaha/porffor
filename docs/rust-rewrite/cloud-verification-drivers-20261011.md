# Cloud campaign and closure drivers — 2026-10-11

The differential grammar wrapper and closure wrapper accept `--cloud` before
their positional arguments. The robustness driver accepts the same option.
Each cloud route uses the existing verification authority to inspect the
actual inherited cgroup-v2 cap, including tighter visible ancestor limits.
Unbounded or unreadable budgets reject execution. Local invocations retain
the existing 4096 MiB/no-swap/grouped-OOM systemd policy.

After activating the environment and building the selected feature-enabled
CLI and worker, use fresh output directories:

```sh
bash scripts/run-differential-campaign-tier.sh --cloud ./target/debug/lila target/differential-fresh pr-fast
python3 -B scripts/limited_verification.py --cloud -- \
  python3 -B scripts/run-robustness-campaign-tier.py --cloud ./target/debug/lila target/robustness-fresh pr-fast
LILA_BIN=./target/release/lila bash scripts/check-test262-closure.sh --cloud
```

The grammar wrapper keeps all eleven grammars, two cases/sixteen replays for
the fast tier and sixty-four cases/replays for nightly. The robustness tier
keeps its seven fast and nineteen nightly targets, exact input bytes and
300,000 ms attempt deadline. Both drivers retain red evidence and continue
independent targets. Closure still synchronizes the pinned suite before
invoking the native owner of two independently fresh full families. No
compiler, fixture, deadline, result-comparison or zero-failure condition changes.

Ten focused driver controls pass, including the five new budget/argument
controls. Stubbed launchers exercise argument delivery and red continuation;
they provide no native campaign or conformance acceptance. Real kernel
admission is covered by the existing limited-verification controls. The complete
tooling suite passes all 437 tests, with zero failures/errors/skips in 140.569
test seconds and 143 watched seconds. Native campaigns and current full pinned
closure remain pending for this batch. The
[receipt](cloud-verification-drivers-20261011.json) retains exact commands,
hashes and the earlier stale-inventory guard result.
