# T22: general Date string parsing on the Wasm-AOT path

## Baseline and scope

This change starts from `821fccaa69ebd62a048543a70ed8480dd6a69841`.
The source audit found that `emit_date_parse_string` recognized precisely two
complete epoch display strings before falling back to ISO parsing. Consequently,
ordinary non-epoch strings emitted by the existing `toString` and `toUTCString`
formatters had no general parser. The ISO parser also consumed missing month/day
fields when a reduced date was followed by a time, and compared an hour of 24
against the next day's decomposed hour of zero, rejecting end-of-day notation.
These are source-level findings; no baseline full Test262 run is claimed.

The relevant specification sections are [Date Time String Format and expanded
years](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-date-time-string-format)
and [Date.parse](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-date.parse).
The latter describes zero-millisecond round trips through the implementation's
own display methods; this patch implements the formats Lila actually emits.
It deliberately does not introduce an arbitrary permissive date parser.

## Implementation

The ISO and display grammars share a private bounded cursor and a consuming
calendar-component finalizer. All character reads pass through one length-checked
operation. Invalid input remains invalid even if later characters match, and
successful parsing requires the entire input to have been consumed.

ISO date-only forms keep their existing defaults. A time may follow a year,
year-month, or full date. Exactly 24:00, with zero seconds and milliseconds when
present, is validated against the written calendar date before adding one day.
The parser then applies the explicit numeric UTC offset and finally TimeClip.
Invalid calendar dates retain the existing rejection policy; this patch does not
claim that every non-ISO policy choice is mandated by ECMAScript.

Display parsing recognizes weekday and month tokens, a signed or unsigned
four-to-six-digit display year, calendar fields, clock fields, and the complete
UTC suffix emitted by the corresponding formatter. Weekday and calendar fields
are checked together. Negative years and the TimeClip endpoints are included in
the runtime regression matrix. These are general runtime string operations,
not source-text recognition or Test262-path dispatch.

The fallback preserves the original string before the ISO attempt because the
caller may alias input and output locals. Date.parse and string Date construction
continue to share their existing coercion and builtin-dispatch boundaries.
No host import, interpreter fallback, test materialization, or suite pin changes.

## Verification

The committed engine target contains twelve independently named tests:

```sh
cargo test --locked -p lila-engine --test aot_date_parsing -- --test-threads=1
```

Each explicitly selects `ExecutionBackend::WasmAot` and checks the result of
executing the compiled program, rather than accepting an emitted instruction
pattern as execution evidence. The existing AOT regression workflow runs this
target alongside its control-flow, suspension, artifact, and backend checks.
Coverage includes reduced-date precision combinations, leap/month/year rollover,
invalid midnight fields, clipping and offsets, runtime-generated display strings,
all months/weekdays, string construction, detached parse calls, truncated inputs,
Unicode lookalikes, sticky failure state, and abrupt coercion.

Verification results and the exact tested commit are recorded in the pull request.
The test inventory is not a claim that the tests have passed before CI executes.
Published fake-suite and real-suite status numbers are intentionally unchanged.
A complete pinned real Date subtree and full Test262 publication remain separate
verification work; this focused patch does not close T22 or T26.

## Remaining boundaries

Local Date operations still use the existing UTC/fixed-offset profile. This patch
does not add a realm default time-zone provider, transition data, daylight-saving
semantics, locale parsing, toLocaleString round trips, arbitrary RFC date syntax,
or general Temporal coverage. T22 and T23 retain those responsibilities.

A complete current-pin Wasm-AOT aggregate with unchanged test sources and complete
helpers is still the authority for overall conformance. A green focused engine
target, a green fake suite, and a clean shortcut census are distinct forms of
evidence and must not be combined into a claimed ECMAScript completion percentage.
