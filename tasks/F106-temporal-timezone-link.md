# F106: Resolve the Pacific/Johnston primary-identity disagreement

- **Status:** open
- **Owner:** lila-intl named_time_zones catalogue generator/provider; lila-aot-wasm Temporal TimeZoneEquals
- **Cause assessment:** suspected
- **Disposition:** required
- **Baseline:** 2 executions across 1 physical files (Bug 2, NotImplemented 0, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F106.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

The pinned catalogue maps Pacific/Johnston to itself as a primary identifier, whereas its underlying IANA backward file and this Test262 pin link it to Pacific/Honolulu. TimeZoneEquals correctly compares the two provider primary identifiers, so the differing catalogue identities produce false. An existing provider test explicitly asserts the self-primary choice. The identity policy must be audited before deciding whether the catalogue generation or upstream expectation needs correction.

## Source evidence

- [crates/lila-intl/data/iana-tzdb-2026a/catalogue.tsv:547](../crates/lila-intl/data/iana-tzdb-2026a/catalogue.tsv#L547): The generated catalogue makes Johnston a separate primary despite identical transition data.
- [crates/lila-intl/data/iana-tzdb-2026a/source/backward:300](../crates/lila-intl/data/iana-tzdb-2026a/source/backward#L300): Pinned IANA source contains the Honolulu-to-Johnston Link.
- [crates/lila-intl/src/provider/named_time_zones/tests.rs:40](../crates/lila-intl/src/provider/named_time_zones/tests.rs#L40): The self-primary decision is also codified in a local provider test.
- [crates/lila-aot-wasm/src/builtins/temporal_time_zone.rs:924](../crates/lila-aot-wasm/src/builtins/temporal_time_zone.rs#L924): The emitted operation compares provider primary identifiers.
- [test262/vendor/test262/test/intl402/Temporal/ZonedDateTime/links.js:101](../test262/vendor/test262/test/intl402/Temporal/ZonedDateTime/links.js#L101): Pinned Test262 expects equality across this alias.

## Work

Audit the primary-zone rules, country/backzone ownership data and the pinned canonical-tz fixture together. Correct the authoritative catalogue generation and matching tests if its policy is wrong; otherwise retain an explicit upstream discrepancy with evidence and pursue the upstream correction. Do not hardcode an equals exception or skip the fixture.

## Validation

Rerun both executions plus the complete time-zone link/canonical identity tests, provider integrity checks and cross-country primary cases after a justified policy/data correction.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F106.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F106-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `sloppy-script:intl402/Temporal/ZonedDateTime/links.js` — Bug

```text
[origin:unknown] uncaught throw: Test262Error: wasm-aot completion: object(handle@2970416: link=Pacific/Johnston, zone=Pacific/Honolulu Expected SameValue(«false», «true») to be true)
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
