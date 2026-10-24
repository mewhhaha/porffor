# Native program compilation and policy repayment

`Engine::profile_program_native_compilation` compiles already emitted P with a
fresh Wasmtime Engine, module caching disabled and no incremental compiler
cache. It reports actual Engine policy, Engine creation time, P validation and
native compilation time, native text bytes and serialized native bytes.
Serialization runs after the compilation timer. The method neither loads R nor
executes the program. It does not change ordinary policy defaults.

`runtime_policy_break_even` keeps one policy per child. Each child first records
one discarded cold-P observation and the selected measured observations, then
uses the real product execution path for warmups and measured fresh-Realm
invocations. Explicit policy cohorts use matching optimized R. Engine creation,
JS compilation and R production remain outside the cold-P timer. RSS includes
prior compiler/image allocator retention; it is not live JavaScript usage.

`scripts/policy_break_even.py` checks the independently calculated result with
Node, retains P and the raw reports, and rejects changed executable, source,
P/R bytes, requested/actual policy, output, sample inventory and cache states.
Its 17 controls cover a small program, retained and ephemeral rows at two
sizes, and strict/sloppy calls with 0/1/2/3/4/8 arguments. Five cohorts isolate
Fast/16/SinglePass, Fast/64/SinglePass, Speed/16/SinglePass and
Speed/16/Backtracking, plus Speed/16/SinglePass with bounded native Wasm
inlining. The inlining cohort recompiles matching R with the same actual
inlining mode and pinned bounds; it cannot reuse the disabled-inlining image.
The first four cohorts are the default. Whole-R inlining retains compiler
intermediate functions and may exceed the verification budget. Request it with
`--cohort speed-16-bounded-inlining` in a separate supervised 4-GiB scope;
retain a resource rejection instead of raising the cap. Select both Speed
cohorts explicitly to produce their P-only repayment comparison.
The [2026-10-24 whole-program trial](../performance-six-workstreams-20261024.md)
was OOM-killed at that cap after its disabled control passed; no bounded
whole-runtime speedup or repayment was admitted. Defaults remain disabled.
Node runs with its normal compilation flags and its
application compile cache disabled.

The descriptive repayment count is:

```
ceil(max(0, Speed_P_compile - Fast_P_compile) /
     (Fast_execution - Speed_execution))
```

No positive observed execution saving means no available repayment estimate.
The report also gives the corresponding whole-invocation estimate, including
setup costs. These estimates reuse compiled P across invocations; they do not
predict a fresh-process startup benefit, establish statistical significance or
authorize changing defaults. Native inlining affects both R and P execution;
its P-only repayment estimate excludes R production and cannot establish whole
program repayment. Retain paired controls when choosing a policy.

Build the example once with the rest of a coherent batch. Run measurements
sequentially, idle of native builds/tests, inside the existing resource policy:

```sh
python3 scripts/limited_verification.py -- python3 scripts/policy_break_even.py \
  --runner target/release/examples/runtime_policy_break_even \
  --output-dir target/policy-break-even-new \
  --time /usr/bin/time --idle-machine
```

GNU time supplies actual whole-child CPU and RSS accounting. The wrapper
requires a kernel-enforced maximum of 4 GiB, zero swap, grouped OOM termination
and one inherited CPU. Positive child cache budgets and GC initial capacity do
not replace that process-tree limit.
