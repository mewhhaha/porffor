# Repairs from the September 2026 partial Wasm-AOT baseline

The input cohort is 592 distinct non-passing executions in twelve completed
matrix nodes, measured with compiler `c5115bf03ba3fdec1a8eb9b3436a939431b2ca1f`
and Test262 tree `aa55200d1310384c5cf69ea95b2a2ecba457007b`. It is a partial
baseline, not full-suite conformance. The continuing baseline retains its
unchanged compiler, suite, cache and snapshots in the original checkout; this
repair branch uses a separate worktree and bounded verification service.

## Shared causes addressed

- Suspend-owned physical bindings include uncaptured block and loop bindings;
  captured Environment Records remain the sole owner of captured cells.
- Synchronous generator loops retain their continuation state and can advance
  through iterations whose single conditional yield is not taken.
- Promise capability creation requires an explicit executor Realm context.
  Await rejection retains its intrinsic Promise context instead of treating
  an ordinary lexical environment as a builtin function object.
- Rejection while unwrapping a synchronous iterator value uses synchronous
  IteratorClose. Its original throw wins, the iterator closes once, and the
  returned object's `done` and `value` properties are not observed.
- Invalid RegExp syntax is rejected before committing source, flags or matcher
  state. A later failing lastIndex write retains a successfully installed pattern.
- Arguments @@isConcatSpreadable uses ordinary symbol-property storage and Get;
  the obsolete private Boolean slot and its competing reads/writes are removed.
- Zero-argument Function constructors create distinct functions with an empty
  body and their constructor's Realm/prototype metadata. Generator, async and
  async-generator bodies pass through the ordinary IR and Wasm dispatchers;
  invocation executes the corresponding function protocol.

## Dynamic-source boundary

Source compilation after AOT remains explicitly unsupported. Known textual eval
and Function subsets are separate compiler-feature work; absence of that support
must not become a fake SyntaxError, runtime sentinel, source substitution or pass.
Possible intrinsic-call identity retained after invalidation authorizes rejection,
not an exact-call optimization. Definite overwrite and shadowing must remain valid.

When source proof is lost before a call, the actual runtime intrinsic rejects
unsupported source through the typed host capability boundary. The engine retains
the operation across Wasmtime and agent-worker errors; Test262 records
Unsupported/NotImplemented. This is separate from JavaScript exceptions: catching
an error or expecting a runtime SyntaxError/TypeError cannot turn missing compiler
support into a pass. Non-string `%eval%` arguments retain their ordinary return
semantics.

## Repeat the recorded failures

`test262/replays/observed-20260908.executions` contains the exact 592 execution
identities, including strict/sloppy modes. Its original outcomes were 518
NotImplemented, 68 Bug and 6 Crash. The list is an input cohort, not an expected
failure list: every execution is run and every non-passing result remains red.

After building `target/release/lila`, run the cohort with two case workers and
one compiler worker per case:

```sh
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 \
  ./scripts/run-watched.sh --label observed-replay --stall 900 -- \
  python3 scripts/replay-test262-executions.py \
  test262/replays/observed-20260908.executions \
  --output-dir target/observed-replay --workers 2
```

Use a fresh output directory each time. Each native runner transcript and parsed
outcome is retained separately; `summary.json` records the executable and input
list hashes. Missing, partial or inconsistent native reports are infrastructure
errors. The command exits zero only when every execution passes, one for ordinary
non-passing results and two for an inconsistent native report. It forces isolated
case runners so their timeouts remain effective.

The watcher limits stalls and CPU use. On the Linux development host, the build
and replay services additionally use `MemoryHigh=10G`, `MemoryMax=12G`,
`MemorySwapMax=0` and `OOMPolicy=stop`. These are cgroup limits; lowering Cargo's
job count or the module cache alone is not a hard RAM limit. Builds use explicit
`cargo build --release --locked -j 2 -p lila-cli`.

To rerun only the failures from a completed replay, generate another exact list
from its summary and use a new output directory:

```sh
python3 - <<'PY'
import json
from pathlib import Path
summary = json.loads(Path('target/observed-replay/summary.json').read_text())
executions = [result['execution_id'] for result in summary['results']
              if result.get('outcome') != 'Success']
Path('target/remaining.executions').write_text('\n'.join(executions) + '\n')
PY
```

An empty list means there are no failures to replay. Focused replay is the repair
loop; adjacent passing tests and periodic full baselines are still needed to find
regressions outside the cohort.

## Verification

Centralized compilation and runtime replays are pending. The PR records exact
executions and candidate outcomes separately from the frozen baseline. New
full-suite counts require a fresh complete publication from the Rust publisher;
focused tests do not authorize editing the generated README status block.
