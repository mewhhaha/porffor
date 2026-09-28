# F085: Split oversized emitted Wasm functions

- **Status:** open
- **Owner:** lila-aot-wasm function emission and static specialization
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F085.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Wasmtime rejects the generated string-upper-lower-mapping module because one function body exceeds its 7,654,321-byte validation limit. This is a code size/codegen failure, despite its saved boa-parser origin label.

## Source evidence

- [crates/lila-aot-wasm/src/emitted_function.rs:36](../crates/lila-aot-wasm/src/emitted_function.rs#L36): Shared emitted function definition owner.
- [test262/vendor/test262/test/staging/sm/String/string-upper-lower-mapping.js:7](../test262/vendor/test262/test/staging/sm/String/string-upper-lower-mapping.js#L7): Original generated Unicode mapping fixture.

## Work

Identify which lowering or specialization expands the generated Unicode mapping fixture into the oversized function, and split/reuse generated helpers or data tables without source-specific test hacks.

## Validation

Compile and execute both original string-upper-lower-mapping modes; check emitted function sizes stay under the target runtime limit.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F085.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F085-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/String/string-upper-lower-mapping.js` — Bug

```text
[origin:boa-parser] wasmtime module validation failed: failed to parse WebAssembly module: function body size count exceeds limit of 7654321 (at offset 0x3ef322)
```

- `strict-script:staging/sm/String/string-upper-lower-mapping.js` — Bug

```text
[origin:boa-parser] wasmtime module validation failed: failed to parse WebAssembly module: function body size count exceeds limit of 7654321 (at offset 0x3ecd8a)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
