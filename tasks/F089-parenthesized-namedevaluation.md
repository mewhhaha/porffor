# F089: Preserve parenthesized assignment syntax for function naming

- **Status:** open
- **Owner:** lila-front vendored boa_parser assignment parser
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F089.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Logical assignment to a parenthesized identifier gives the anonymous RHS the identifier name, while the fixture expects an empty name. Assignment parsing normalizes the target to Identifier and applies set_anonymous_function_definition_name without retaining whether the target syntax was parenthesized.

## Source evidence

- [vendor/boa_parser-0.21.1/src/parser/expression/assignment/mod.rs:344](../vendor/boa_parser-0.21.1/src/parser/expression/assignment/mod.rs#L344): Logical assignment naming is decided from the normalized identifier target.

## Work

Retain the syntactic NamedEvaluation eligibility before assignment-target normalization and apply inference only to the eligible identifier-reference production.

## Validation

Run both short-circuit-compound-assignment-anon-fns variants, keeping unparenthesized name inference green.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F089.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F089-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:staging/sm/expressions/short-circuit-compound-assignment-anon-fns.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1639968: Expected SameValue(«"a"», «""») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
