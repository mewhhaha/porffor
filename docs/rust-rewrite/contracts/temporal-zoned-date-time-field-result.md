# `Temporal.ZonedDateTime` field-result ownership

Status: source-equivalent T22 ownership invariant implemented with focused
verification, 2026-08-27.

## Boundary

`emit_temporal_zoned_date_time_iso_field` emits one of twelve accessor bodies.
Nine bodies leave one Number payload on the Wasm stack. `monthCode`, `era` and
`eraYear` instead write the complete result payload/tag pair themselves. The
private domain naming that distinction is:

```rust
enum ZdtFieldResult {
    NumberOnStack,
    WrittenByCallee,
}
```

Every invocation creates one result value from the exhaustive field match and
immediately consumes it in the sole result-publication match. The domain has no
clone, copy, debug, equality or default capability. A second consuming
observation therefore fails to compile instead of duplicating or moving result
publication silently.

## Preserved behavior

This hardening removes derived capabilities only. It does not change any field
arm, emitted instruction, helper call, result tag or temporary-local order.
The nine numeric arms still publish the stack value as Number. `monthCode`
still publishes String, while `era` and `eraYear` retain the calendar emitter's
String, Number or Undefined result pair.

The type does not prove that an arm selected the correct result variant. That
mapping remains guarded structurally: the exact complete twelve-arm block and
the final consuming match are pinned together with no unchecked gap.

## Durable source guard

The Rust-lexical structure target pins:

- an attribute-free private two-variant declaration with no manual capability,
  alias, representation or cast route;
- exactly 15 source-wide `ZdtFieldResult` identifiers: one declaration, twelve
  producers and two consumer arms;
- exactly ten qualified `NumberOnStack` routes and four qualified
  `WrittenByCallee` routes, including their final consumer arms;
- exactly two `delivery` identifiers, its inferred binding and its consuming
  match;
- all twelve complete field bodies in order, each bound to its exact result
  variant; and
- the exact final Number publication, empty self-written arm and publication
  before temporary-local release, with no wildcard or secondary observation.

The lexical normalizer ignores comments and formatting, preserves literal
contents, canonicalizes raw identifiers and is exercised by normal, byte, C,
raw, raw-byte and raw-C strings plus character, byte-character and lifetime
syntax.

## Focused evidence

```sh
cargo test -p lila-aot-wasm \
  --test temporal_zoned_date_time_field_result_structure -- --test-threads=1
cargo test -p lila-aot-wasm \
  --test temporal_zoned_date_time_calendar_coercion_structure -- --test-threads=1
cargo test -p lila-cli --test cli -- \
  --exact date::run_wasm_backend_succeeds_for_temporal_zoned_date_time_era_fixture
```

The CLI fixture covers the numeric tail and the self-written `era`/`eraYear`
path, including ISO Undefined results, gregory String/Number pairs, the BCE
boundary, fixed offsets and sub-millisecond components. The dedicated and
neighboring structure targets pass `3/3` each, the exact CLI fixture passes
`1/1`, and workspace formatting plus the owned diff check are green.

## Nonclaims

This invariant does not add a Temporal accessor, time-zone data, calendar
support or general Temporal conformance. It does not make selection of the
correct result variant a Rust type proof, change `ZonedDateTimeField`, or alter
the standard builtin dispatcher. It changes no published conformance count and
does not close T22.
