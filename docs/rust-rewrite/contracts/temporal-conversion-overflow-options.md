# Temporal conversion overflow options

Status: normative for overflow-option ownership in the five plain Temporal
conversion helpers. The current PlainDate shared-helper successor and its
retargeted controls are authored and await the joined checkpoint.

## Boundary

`ToTemporalDate`, `ToTemporalYearMonth`, `ToTemporalTime`,
`ToTemporalDateTime` and `ToTemporalMonthDay` are shared by their corresponding
`from` builtin and by internal conversions whose algorithms do not receive an
overflow options bag. Those helpers previously accepted two local indexes plus
a `read_options` Boolean. The 15 internal producers had to allocate, initialize
and release dummy undefined payload/tag locals only to make the three arguments
well formed.

The builtin emitters accept `TemporalConversionOverflowOptions`:

| Variant | Ownership |
| --- | --- |
| `Read(&ValueLocals)` | The public `from` builtin owns a real options value and each observable conversion path reads its `overflow` property at the existing specification point. |
| `Omit` | The internal algorithm has no conversion overflow options and carries no placeholder locals that could be mistaken for a real options value. |

The public `from` producers construct `Read`; internal conversions without an
overflow-options operand construct `Omit`. The private domain has no default,
wildcard, Boolean projection, equality capability or fallback state.

PlainDate's facade maps these variants exhaustively to the registered
`OptionalValue` helper operand. Presence is separate from the value tag, so
`Read(undefined)` remains distinct from `Omit`. The complete private conversion
uses the guarded parameter at each original overflow-read point. The other
four converters consume the same enum directly. See
[Shared PlainDate conversion](temporal-plain-date-shared-conversion.md) for
whole-Completion transport, called-Realm ownership and temporary-record lifetime.

## Observable witness

`wasm_temporal_conversion_overflow_options.js` executes the five public `from`
producers with a shared `overflow` getter and checks that it is read exactly
five times. It also executes every internal producer: `compare`, `equals` and
`until` for the four full plain receiver families, PlainMonthDay `equals`,
PlainDate `toPlainDateTime` and PlainDateTime `withPlainTime`. Each converted
branded argument has a throwing own `overflow` getter, proving that the
internal conversions omit that read while still returning their known result.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test temporal_conversion_overflow_options_structure
cargo test -p lila-cli --test cli date::run_wasm_backend_preserves_temporal_conversion_overflow_options -- --exact --test-threads=1
```

The bounded structure target checks the data-bearing variants, actual typed
consumers, closed optional operand, absence of raw read controls and dummy
undefined-local lifecycles, plus Normal-only projection and caller throw
propagation. The current retargeted target has not run. The preceding enum
checkpoint passed `3/3` structure tests and the CLI witness passed `1/1`; those
results do not verify the new shared helper.

## Golden impact and deferrals

The original enum closure preserved each overflow read's branch location and
removed dummy locals from internal producers. Its semantic golden passed `2/2` in
717.58 seconds with 674 dumps, adds this witness plus the independent Promise
combinator Realm and GroupBy result-kind witnesses, removes none and leaves all
671 retained dumps equal after accounting normalization. This is historical
evidence, predating shared PlainDate conversion. Broad Date/Temporal and
Test262 trees remain deferred.

This closure does not change overflow validation, calendar field preparation,
arithmetic, time-zone behavior or other Temporal option domains.
