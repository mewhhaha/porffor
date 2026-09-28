# F047: Route String.localeCompare through Intl.Collator

- **Status:** open
- **Owner:** lila-aot-wasm builtins/standard.rs and intl_collator.rs
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 6 executions across 3 physical files (Bug 6, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F047.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

String.prototype.localeCompare reads only the receiver and argument 0, NFC-normalizes them, and compares UTF-16 code units. It never reads locales/options or initializes a Collator. This explains wrong locale ordering, ignored Object.prototype defaults, and omitted locale-validation errors.

## Source evidence

- [crates/lila-aot-wasm/src/builtins/standard.rs:30462](../crates/lila-aot-wasm/src/builtins/standard.rs#L30462): This branch emits only string coercion and normalization.
- [crates/lila-aot-wasm/src/builtins/standard.rs:30477](../crates/lila-aot-wasm/src/builtins/standard.rs#L30477): The localeCompare path compares normalized strings directly instead of using the Collator provider.
- [test262/vendor/test262/test/intl402/String/prototype/localeCompare/throws-same-exceptions-as-Collator.js:6](../test262/vendor/test262/test/intl402/String/prototype/localeCompare/throws-same-exceptions-as-Collator.js#L6): The fixture checks that the convenience method observes the same locale/options validation as Collator.

## Work

Keep receiver and comparison-value coercions in specification order, then construct/initialize the same Collator configuration used by Intl.Collator with arguments 1 and 2 and call its comparison kernel.

## Validation

Run all six recorded executions and the localeCompare subtree; cover default locale, inherited options, invalid locales, observable conversion order, and sign agreement with Intl.Collator.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F047.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F047-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/String/prototype/localeCompare/default-options-object-prototype.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1634632: Expected SameValue(«1», «-1») to be true)
```

- `sloppy-script:intl402/String/prototype/localeCompare/returns-same-results-as-Collator.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1813984: Actual [!A, A, C, O, X, Z, Z., b, d, od, oe, of, y, ö, ö, 吉野家, 𠮷野家] and expected [!A, A, b, C, d, O, ö, ö, od, oe, of, X, y, Z, Z., 吉野家, 𠮷野家] should have the same contents. (Testing with locales undefined; options undefined.))
```

- `sloppy-script:intl402/String/prototype/localeCompare/throws-same-exceptions-as-Collator.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@1642680: String.prototype.localeCompare didn't throw exception for locales null. Expected a TypeError to be thrown but no exception was thrown at all)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
