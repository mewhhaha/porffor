# F055: Parse nested property and super destructuring assignment targets

- **Status:** open
- **Owner:** lila-front vendored boa_parser assignment target conversion
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F055.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The large order/order-super fixtures are rejected by AssignTarget conversion as invalid left-hand sides before their runtime evaluation-order assertions. A valid nested property/super assignment pattern shape is missing from AST target conversion; exact rejected subpattern needs reduction.

## Source evidence

- [vendor/boa_parser-0.21.1/src/parser/expression/assignment/mod.rs:306](../vendor/boa_parser-0.21.1/src/parser/expression/assignment/mod.rs#L306): Exact parser branch returning Invalid left-hand side in assignment.

## Work

Recover the test-source location from the prepared-source offset, reduce the rejected pattern, and teach AssignTarget conversion the valid form without relaxing early errors for invalid targets.

## Validation

Run both order fixtures in both modes, then execute their complete effect-order assertions.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F055.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F055-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/destructuring/order-super.js` — Bug

```text
[origin:boa-parser] parse error: Invalid left-hand side in assignment at line 708, col 7
```

- `sloppy-script:staging/sm/destructuring/order.js` — Bug

```text
[origin:boa-parser] parse error: Invalid left-hand side in assignment at line 719, col 3
```

- `strict-script:staging/sm/destructuring/order-super.js` — Bug

```text
[origin:boa-parser] parse error: Invalid left-hand side in assignment at line 709, col 7
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
