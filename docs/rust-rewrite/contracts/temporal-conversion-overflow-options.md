# Temporal conversion overflow options

Status: normative for overflow-option ownership in the five plain Temporal
conversion helpers.

## Boundary

`ToTemporalDate`, `ToTemporalYearMonth`, `ToTemporalTime`,
`ToTemporalDateTime` and `ToTemporalMonthDay` are shared by their corresponding
`from` builtin and by internal conversions whose algorithms do not receive an
overflow options bag. Those helpers previously accepted two local indexes plus
a `read_options` Boolean. The 15 internal producers had to allocate, initialize
and release dummy undefined payload/tag locals only to make the three arguments
well formed.

The helpers now accept only `TemporalConversionOverflowOptions`:

| Variant | Ownership |
| --- | --- |
| `Read { payload_local, tag_local }` | The public `from` builtin owns a real options value and each observable conversion path reads its `overflow` property at the existing specification point. |
| `Omit` | The internal algorithm has no conversion overflow options and carries no placeholder locals that could be mistaken for a real options value. |

All five `from` producers construct `Read`; the remaining 15 producers
construct `Omit`. The five converters consume the domain in 16 direct
exhaustive matches: three each in PlainDate, PlainYearMonth, PlainTime and
PlainMonthDay, and four in PlainDateTime. The private domain has no default,
wildcard, Boolean projection, equality capability or fallback state.

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

The bounded structure target owns the exact data-bearing variants, five typed
consumers, 16 exhaustive decisions, exact five-plus-15 producer census and the
absence of raw read controls or dummy undefined-local lifecycles. It passes all
`3/3` tests, and the exact CLI witness passes `1/1`. Workspace Rust formatting
and the scoped diff check are green.

## Golden impact and deferrals

This is a source-equivalent closure. It preserves each overflow read's branch
location and therefore its observable access order. The 15 `Omit` producers no
longer initialize dummy locals. The shared semantic golden passes `2/2` in
717.58 seconds with 674 dumps, adds this witness plus the independent Promise
combinator Realm and GroupBy result-kind witnesses, removes none and leaves all
671 retained dumps equal after accounting normalization. Broad Date/Temporal
and Test262 trees remain deferred.

This closure does not change overflow validation, calendar field preparation,
arithmetic, time-zone behavior or other Temporal option domains.
