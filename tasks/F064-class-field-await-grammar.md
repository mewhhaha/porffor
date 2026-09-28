# F064: Track Await grammar parameters correctly inside class field initializers

- **Status:** open
- **Owner:** lila-front vendored boa_parser class parser
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 3 executions across 2 physical files (Bug 3, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F064.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

A script field initializer using await as an identifier is rejected, while a module field initializer using await as an expression is accepted when SyntaxError is required. Class field grammar is using the enclosing async function Await parameter incorrectly.

## Source evidence

- [vendor/boa_parser-0.21.1/src/parser/expression/primary/class_expression/mod.rs:1](../vendor/boa_parser-0.21.1/src/parser/expression/primary/class_expression/mod.rs#L1): Class expression parser owns the grammar context.
- [test262/vendor/test262/test/staging/sm/fields/await-identifier-script.js:14](../test262/vendor/test262/test/staging/sm/fields/await-identifier-script.js#L14): Valid script field references the outer binding named await.
- [test262/vendor/test262/test/staging/sm/fields/await-identifier-module-3.js:15](../test262/vendor/test262/test/staging/sm/fields/await-identifier-module-3.js#L15): Module negative requires rejection of the await expression in a field.

## Work

Give field initializers their own grammar context and apply module reserved-word restrictions separately from permission to parse AwaitExpression.

## Validation

Run all three attached script/module await-identifier executions and adjacent computed-name cases.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F064.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F064-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:staging/sm/fields/await-identifier-module-3.js` — Bug

```text
[origin:boa-parser] negative test expected parse error but compile succeeded
```

- `sloppy-script:staging/sm/fields/await-identifier-script.js` — Bug

```text
[origin:boa-parser] parse error: unexpected token ';', primary expression at line 228, col 14
```

- `strict-script:staging/sm/fields/await-identifier-script.js` — Bug

```text
[origin:boa-parser] parse error: unexpected token ';', primary expression at line 229, col 14
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
