# F037: Enforce Unicode-mode RegExp early syntax errors for dynamic patterns

- **Status:** open
- **Owner:** lila-aot-wasm RegExp construction; lila-ir regexp validation
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 12 executions across 6 physical files (Bug 12, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F037.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Invalid Unicode-mode brackets, class escapes, and identity/control escapes are accepted instead of throwing SyntaxError. Static grammar validation exists, but the runtime-constructed pattern path does not apply equivalent validation before producing a RegExp.

## Source evidence

- [crates/lila-ir/src/regexp.rs:595](../crates/lila-ir/src/regexp.rs#L595): Compiler distinguishes InvalidSyntax from UnsupportedFeature.
- [crates/lila-aot-wasm/src/builtins/string.rs:13918](../crates/lila-aot-wasm/src/builtins/string.rs#L13918): Runtime fallback keeps its own incomplete escape classification.

## Work

Trace runtime construction and flag handling for each invalid family; use a complete RegExp grammar/validation path and keep unsupported matching distinct from invalid syntax.

## Validation

Run attached invalid Unicode pattern tests and constructor-regexp-unicode, checking exact SyntaxError behavior.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F037.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F037-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/RegExp/unicode_restricted_brackets.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1641792: RegExp("]", "u"):  Expected a SyntaxError to be thrown but no exception was thrown at all)
```

- `sloppy-script:built-ins/RegExp/unicode_restricted_character_class_escape.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1636008: RegExp("[\d-a]", "u"):  Expected a SyntaxError to be thrown but no exception was thrown at all)
```

- `sloppy-script:built-ins/RegExp/unicode_restricted_identity_escape.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1638792: Invalid IdentityEscape in AtomEscape: '\\u0000' Expected a SyntaxError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
