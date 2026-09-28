# F052: Refresh decimal numbering data to include Tolong Siki (tols)

- **Status:** open
- **Owner:** lila-intl number_format profiles and pinned CLDR data
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 4 executions across 2 physical files (Bug 4, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F052.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The pinned CLDR 47 number profile has 77 numeric systems and no tols mapping, while this Test262 pin requires its U+11DE0..U+11DE9 digits. resolve_number_locale discards any requested system absent from the profile, leaving the default latn. supportedValuesOf reads the same inventory. Both failures follow from missing data rather than digit arithmetic.

## Source evidence

- [crates/lila-intl/src/number_format/profiles/mod.rs:467](../crates/lila-intl/src/number_format/profiles/mod.rs#L467): The formatter loads the pinned binary profile; inspection of its paired profiles.json.gz found no tols system.
- [crates/lila-intl/src/number_format/configuration.rs:243](../crates/lila-intl/src/number_format/configuration.rs#L243): Unsupported requested systems are filtered out and the default numbering remains.
- [crates/lila-intl/src/number_format/tests.rs:126](../crates/lila-intl/src/number_format/tests.rs#L126): Existing profile coverage asserts 77 systems.
- [test262/vendor/test262/harness/testIntl.js:2334](../test262/vendor/test262/harness/testIntl.js#L2334): Pinned harness requires Tolong Siki decimal digits.

## Work

Refresh the authoritative Unicode/CLDR inputs and generated number-system inventory with provenance and fingerprints; include tols in both formatting and supportedValuesOf. Audit all required simple mappings against the data rather than adding a test-specific override.

## Validation

Rerun both listed fixtures in both modes and verify all ten digits for every required numbering system, supportedValuesOf agreement, supplementary-plane UTF-16 handling, and regenerated-data integrity.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F052.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F052-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Intl/supportedValuesOf/numberingSystems-with-simple-digit-mappings.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@4022072: tols with simple digit mappings is supported)
```

- `sloppy-script:intl402/NumberFormat/prototype/format/numbering-systems.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@5289832: numberingSystem: tols, digit: 0 Expected SameValue(«"0"», «"𑷠"») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
