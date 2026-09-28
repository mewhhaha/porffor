# F015: Align RegExp Unicode property data with pinned Test262

- **Status:** open
- **Owner:** lila-ir regexp.rs; ICU property data
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 138 executions across 69 physical files (Bug 138, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F015.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Generated Unicode-property fixtures disagree on specific newly assigned or changed code points. RegExp property ranges are read from pinned ICU sets/maps; the mismatch pattern indicates the runtime data version differs from the suite generator data. Confirm the data versions before changing range semantics.

## Source evidence

- [crates/lila-ir/src/regexp.rs:2619](../crates/lila-ir/src/regexp.rs#L2619): Properties are populated from ICU set/map data.
- [crates/lila-ir/src/regexp.rs:2660](../crates/lila-ir/src/regexp.rs#L2660): Script extensions and scripts share the pinned property provider.

## Work

Determine the pinned suite Unicode data version and update/regenerate one authoritative property dataset used by categories, scripts, script extensions, binary properties, and complements.

## Validation

Run all attached generated property executions and compare positive/negative membership at the reported code points.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F015.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F015-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:built-ins/RegExp/property-escapes/generated/Alphabetic.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@94311080: `\p{Alphabetic}` should match U+00088F (`࢏`))
```

- `sloppy-script:built-ins/RegExp/property-escapes/generated/Assigned.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@150430184: `\p{Assigned}` should match U+01FAC8 (`🫈`))
```

- `sloppy-script:built-ins/RegExp/property-escapes/generated/Case_Ignorable.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@3850192: `\p{Case_Ignorable}` should match U+010EC5 (`𐻅`))
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
