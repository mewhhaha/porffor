# F042: Retain global class and lexical bindings across evalScript

- **Status:** open
- **Owner:** lila-ir global instantiation; lila-aot-wasm realm_eval_script
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 7 executions across 4 physical files (Bug 7, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F042.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

evalScript-created lexical/class bindings are missing from later lookup/deletion and class redeclaration checks. The global declarative record is not consistently retained/shared with subsequent compiled script specializations; class bindings are especially visible in the failures. Literal evalScript source is already compiled, so this is required environment semantics.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/host/realm_eval_script.rs:1](../crates/lila-aot-wasm/src/builtins/host/realm_eval_script.rs#L1): Host evalScript dispatch and realm environment owner.
- [test262/vendor/test262/test/language/global-code/script-decl-lex.js:47](../test262/vendor/test262/test/language/global-code/script-decl-lex.js#L47): The later class binding must remain visible outside evalScript.

## Work

Keep one realm global declarative environment across compiled evalScript invocations, register class/let/const uniformly, and enforce lexical-var conflicts and nondeletability.

## Validation

Run all attached script-decl-lex and script-decl-var-collision executions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F042.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F042-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/global-code/script-decl-lex-deletion.js` — Bug

```text
[origin:unknown] uncaught throw: ReferenceError: wasm-aot completion: object(handle@1988720: unbound identifier)
```

- `sloppy-script:language/global-code/script-decl-lex-lex.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2011584: `class` binding Expected a SyntaxError to be thrown but no exception was thrown at all)
```

- `sloppy-script:language/global-code/script-decl-var-collision.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2014192: `var` on `class` binding Expected a SyntaxError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
