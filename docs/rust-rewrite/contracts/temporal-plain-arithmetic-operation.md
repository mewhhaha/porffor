# Temporal plain arithmetic operation

Status: normative for the shared `add` / `subtract` dispatch of the four plain
Temporal types.

## Boundary

`Temporal.PlainDate`, `Temporal.PlainYearMonth`, `Temporal.PlainTime` and
`Temporal.PlainDateTime` each have one shared arithmetic emitter for their
`add` and `subtract` prototype methods. The standard-builtin dispatcher
previously selected the direction with eight raw Boolean arguments. A
transposed Boolean compiled and made one named builtin perform the opposite
operation.

All eight producers now construct only
`TemporalPlainArithmeticOperation::{Add, Subtract}`. Each of the four emitters
consumes that operation with a direct exhaustive match before the shared
arithmetic:

| Receiver family | `Add` | `Subtract` |
| --- | --- | --- |
| PlainDate | Preserve the converted duration fields | Negate all ten duration fields |
| PlainYearMonth | Preserve the converted duration fields | Negate all ten duration fields |
| PlainTime | Preserve the converted duration fields | Negate all ten duration fields |
| PlainDateTime | Preserve the converted duration fields | Negate all ten duration fields |

The domain is visible only within the builtin module tree. It has no default,
wildcard, string or Boolean projection. Adding an operation requires every
plain arithmetic emitter to state its behavior, while adding a builtin producer
requires a named operation at its call site.

## Observable witness

`wasm_temporal_plain_arithmetic_operation.js` independently executes both
directions for all four receiver families. Its known results distinguish the
operation before formatting or locale behavior can obscure it:

- PlainDate moves two days forward and backward;
- PlainYearMonth moves two months forward and backward;
- PlainTime moves two hours forward and backward; and
- PlainDateTime moves one day and two hours forward and backward.

Every assertion throws a receiver-and-operation label, so a transposed producer
identifies the exact failed mapping.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test temporal_plain_arithmetic_operation_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_distinguishes_plain_temporal_add_and_subtract -- --exact --test-threads=1
cargo fmt --all -- --check
git diff --check
```

The bounded structure target owns the exact two-variant domain, four exhaustive
consumers, eight named producers and absence of raw operation Booleans at those
boundaries. It passes all `3/3` tests, and the exact CLI witness passes `1/1`.
Rust formatting, the module-boundary policy, `cargo xc` and the diff check are
green. The following workspace semantic golden passes `2/2` in 771.49 seconds
with 669 dumps, adds only this witness, removes none, and leaves 667 of 668
retained dumps equal after accounting normalization. The sole retained
structural change is the independent Promise callback Realm witness. No
Test262 tree was run for this source-equivalent closure.

## Deferrals

This source-equivalent type closure does not change Temporal arithmetic,
calendar or time-zone semantics. It does not merge the already typed
`Temporal.Duration` or `Temporal.ZonedDateTime` operation domains, type the
remaining Temporal field-reader policies, run a broad Test262 tree, or publish
conformance status.
