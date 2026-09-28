# F066: Parse let and escaped contextual identifiers as labels where allowed

- **Status:** open
- **Owner:** lila-front vendored boa_parser statement/label parsing
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 3 executions across 2 physical files (Bug 3, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F066.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Sloppy source evaluated from literals rejects let/escaped-let labels with expected semicolon before colon. Statement dispatch treats the contextual keyword as an expression/declaration before considering the permitted LabelIdentifier grammar.

## Source evidence

- [vendor/boa_parser-0.21.1/src/parser/statement/mod.rs:25](../vendor/boa_parser-0.21.1/src/parser/statement/mod.rs#L25): Statement dispatch owns labelled statement versus expression classification.

## Work

Preserve token escape information and apply contextual keyword grammar for labelled statements, keeping strict reserved-word errors intact.

## Validation

Run escaped-let-static-identifier and let-as-label in the attached modes and verify their eval-created syntax outcomes.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F066.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F066-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/syntax/escaped-let-static-identifier.js` — Bug

```text
[origin:unknown] uncaught throw: SyntaxError: wasm-aot completion: object(handle@1634128: parse error: expected token ';', got ':' in expression statement at line 2, col 9)
```

- `sloppy-script:staging/sm/syntax/let-as-label.js` — Bug

```text
[origin:unknown] uncaught throw: SyntaxError: wasm-aot completion: object(handle@1978072: parse error: expected token ';', got ':' in expression statement at line 2, col 4)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
