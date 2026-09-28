# F115: Account for runtime eval source without a compiled specialization

- **Status:** open
- **Owner:** lila-ir dynamic_source and eval_sources; lila-aot-wasm dynamic construction
- **Cause assessment:** confirmed
- **Disposition:** aot-dynamic-boundary
- **Baseline:** 151 executions across 81 physical files (Bug 0, NotImplemented 151, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F115.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The saved execution selected eval text/environment for which the compiler emitted no specialization. These records explicitly report the Wasm-AOT dynamic-code-generation boundary; arbitrary runtime source would require a parser/evaluator in the artifact and is outside the product contract. This grouping records the observed refusal, not a proof that every individual fixture is impossible to precompile: literal and bounded generated sources should be reviewed for finite specialization before declaring them irreducible.

## Source evidence

- [crates/lila-ir/src/dynamic_source.rs:177](../crates/lila-ir/src/dynamic_source.rs#L177): Typed runtime operation formats this exact missing-specialization diagnosis.
- [AGENTS.md:23](../AGENTS.md#L23): The product contract permits explicit dynamic-code-generation unsupported outcomes, without silent skips.

## Work

Keep every execution visible as unsupported. Review each recorded source/closure environment and move demonstrably finite specialization gaps into required implementation work. Preserve a typed unsupported outcome for genuinely runtime-generated source; never bundle a JS VM or skip these tests.

## Validation

For each resolved finite case, run the original fixture in both modes. For irreducible cases, verify the explicit dynamic-source diagnostic and that the result remains in total failure accounting.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F115.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F115-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:annexB/built-ins/RegExp/RegExp-leading-escape-BMP.js` — NotImplemented

```text
[origin:unknown] unsupported dynamic-source operation `eval` selected during Wasm execution without a compiled source/environment specialization
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
