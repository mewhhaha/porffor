# F029: Capture class names, private names and lexical super/newTarget in eval

- **Status:** open
- **Owner:** lila-ir lowering/direct_eval.rs and class definition environments
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 14 executions across 7 physical files (Bug 14, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F029.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Compiled literal eval within class methods/fields and nested arrows loses class/private binding visibility or lexical super/new.target context. Recorded failures are unbound identifiers, nonconstructor super calls, null super bases and undefined new.target. All source strings are finite; this is context transport, not an arbitrary-source boundary.

## Source evidence

- [crates/lila-ir/src/lowering/direct_eval.rs:35](../crates/lila-ir/src/lowering/direct_eval.rs#L35): This planner gathers private names and a derived constructor owner.
- [test262/vendor/test262/test/staging/sm/class/superPropBasicGetter.js:28](../test262/vendor/test262/test/staging/sm/class/superPropBasicGetter.js#L28): Literal eval must see both the home object and parameter v.

## Work

Carry the correct class/private environment, home object, derived-constructor activation, this binding and new.target into each direct-eval specialization; split into narrower fixes after reducing each context variant.

## Validation

Run all attached class/direct-eval cases and verify private setters, class-name reads, derived super calls and nested-arrow new.target.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F029.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F029-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:language/statements/class/elements/private-setter-visible-to-direct-eval.js` — Bug

```text
[origin:unknown] uncaught throw: ReferenceError: wasm-aot completion: object(handle@1959040: unbound identifier)
```

- `sloppy-script:staging/sm/class/derivedConstructorArrowEvalNestedSuperCall.js` — Bug

```text
[origin:unknown] uncaught throw: TypeError: wasm-aot completion: object(handle@1981424: target is not a constructor)
```

- `sloppy-script:staging/sm/class/newTargetEval.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@6865808: Expected SameValue(«undefined», «function assertNewTarget(expected) {
    assert.sameValue(eval('new.target'), expected);
    assert.sameValue((()=>eval('new.target'))(), expected);

    // Also test nestings "by induction"
    assert.sameValue(eval('eval("new.target")'), expected);
    assert.sameValue(eval("eval('eval(`new.target`)')"), expected);
}») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
