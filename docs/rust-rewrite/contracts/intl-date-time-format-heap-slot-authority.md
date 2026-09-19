# Intl.DateTimeFormat heap-slot identity authority

The passive Intl.DateTimeFormat record has exactly twenty-three capability-free
`IntlDateTimeFormatHeapSlot` identities in storage order. Locale, calendar,
numbering system, time-zone identifier and bound formatter are the five traced
payload fields; the remaining eighteen fields are untraced scalars. Every field
is eight bytes wide. One private exhaustive metadata projection owns each
record name, slot name, offset, width and pointer classification, and the typed
registry fixes their order.

`TimeZoneFixedSeconds` stores signed seconds for fixed selections, while
`TimeZoneKind` stores the closed named/fixed discriminator. The former cached
GMT-name payload is removed and its slot is no longer a GC root. Names selected
for individual endpoints are packed strings owned by the component locals and
released with those locals. See the
[named-zone implementation](../intl-named-time-zones.md).

The closed identity prevents callers from pairing a field name with an
unrelated offset or pointer bit. It derives and implements no clone, copy,
debug, equality, ordering, hashing or default capability.

Verification targets:

```sh
cargo test -p lila-aot-wasm --test intl_date_time_format_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::intl_date_time_format_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
cargo test -p lila-engine --test aot_intl_named_time_zones
```
