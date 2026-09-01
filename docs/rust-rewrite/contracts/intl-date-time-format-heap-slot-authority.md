# Intl.DateTimeFormat heap-slot identity authority

## Closed passive layout

The passive Intl.DateTimeFormat record has exactly twenty-three capability-free
`IntlDateTimeFormatHeapSlot` identities in storage order. Locale, calendar,
numbering system, time-zone identifier, GMT name and bound formatter are the six
traced payload fields; the remaining seventeen fields are untraced scalars. All
fields retain their existing 8-byte widths and offsets. One private exhaustive
metadata projection owns every record name, slot name, offset, width and pointer
classification, and the typed registry fixes their order.

The former free-form table could pair an arbitrary field name with the wrong
offset or pointer bit. The closed identity makes those combinations unavailable
outside its sole exhaustive projection. The domain derives and implements no
clone, copy, debug, equality, ordering, hashing or default capability.

## Passive boundary

This is a source-equivalent passive metadata migration. The former 23-row,
165-line layout has SHA-256
`7c2284a3fc1325cf43f042d1df6240f96b1c273836bada75c9cd2a8410d7d6a9`.
The 257-line typed owner has SHA-256
`4679b3d4ffae6088c8dca5c580b8356278e91b64624e965b6ef6270a9cb5dd59`.
It changes no Intl.DateTimeFormat allocation, field access, option handling,
formatting, emitted Wasm, root scanning or collector execution. It claims no
new Intl behavior, Test262 pass or published conformance change.

## Verification

```sh
cargo test -p lila-aot-wasm --test intl_date_time_format_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::intl_date_time_format_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

At the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the structure target
passes `4/4`, the focused slot-identity unit passes `1/1`, and both heap-layout
registry controls pass `2/2`. No runtime CLI, Test262 leaf or semantic golden is
required for this source-equivalent passive metadata migration. It claims no new Intl behavior
and no published conformance-count change.
