# DateTimeFormat range locals have one release owner

`PartitionDateTimeRangePattern` reserves two complete component sets plus its
loop, practical-equality and selected-pattern locals. `DtfComponentLocals` and
`DtfRangeLocals` are private non-`Clone`, non-`Copy` carriers for that local
stack. The private, non-capability `DtfRangePattern` exhaustively maps the
fallback, textual month-difference and textual day-difference patterns to the
three runtime codes the emitted formatter recognizes.

The formatter creates one current component view. A range additionally creates
one `DtfRangeLocals`, whose `start` and `end` fields come from the only component
reservation function. Component calculation, practical equality, range
capacity, loop setup, textual interval selection, field visibility and loop
termination borrow those values. After those eight shared observations of the
optional range, the final release consumes and exhaustively destructures it. A
future carrier field therefore requires an explicit release-owner decision
before the crate builds.

The consuming tail releases `pattern`, `practically_equal`, `side_limit` and
`side`, then the end components in reverse reservation order, then the start
components in reverse reservation order. `second_time` is an input local owned
by the caller, not a range reservation, so consuming the carrier deliberately
does not release it. A duplicate component release or a use after the final
handoff is therefore a Rust move error instead of a latent Wasm-local stack
defect.

The structure regression pins the private attribute-free declarations, exact
11/4/8 production identifier census, three produced component values, sole
range producer, borrowed helper signatures, eight shared range routes, one
consuming route and the complete release tail. It also prevents `Clone`, `Copy`,
manual capabilities or an alternate release route from reopening the
lifecycle.

```sh
cargo test -p lila-aot-wasm --test intl_dtf_range_locals_ownership_structure -- --test-threads=1
cargo test -p lila-aot-wasm --test intl_dtf_range_mode_structure -- --test-threads=1
```

The emitted range formatter now selects the two CLDR `en` textual
year/month/day interval shapes only when that is the complete effective field
set. A shared year is emitted once at the suffix; a day-only difference also
emits the shared month once at the prefix. Numeric dates and mixed date/time
formats retain the complete-side fallback rather than being guessed into a
textual pattern. `formatRange` and `formatRangeToParts` use the same selection,
and the latter changes source attribution at field boundaries so shared
literals stay shared.

The exact same-date and en-US range leaves for both APIs pass all `8/8`
sloppy/strict Wasm-AOT executions. Every Parser, EarlyError, Lowering, Runtime,
WasmBackend, HostHarness and Unsupported bucket is zero; all eight outcomes are
`Success`, with `NotImplemented`, `Crash` and `Bug` also zero. This focused
checkpoint does not claim ownership of the separately named single-date locals
or broader DateTimeFormat/Intl conformance.
