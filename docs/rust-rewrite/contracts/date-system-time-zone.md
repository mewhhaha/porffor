# Date system-zone GC boundary

Status: source-authored for the atomic T05 GC cutover on 2026-10-04. The current
source has no compile, Wasm validation, host ABI, runtime or pinned-suite proof.
MAIN remains integration 117.

The immutable Realm system-zone owner crosses the host boundary through
`system_time_zone_snapshot: () -> (ref ByteArray)`. The Engine binds the emitted
module's exact canonical mutable packed-u8 array type and allocates the response
in that type. No JavaScript object handle or linear-memory pointer crosses this
boundary. A bounded private byte reader validates the complete snapshot before
publishing its GC identifier and typed kind/fixed-offset projection. The native
Realm owner remains the authority for admitted primary named-zone identifiers.

Named-zone requests use `intl_provider_call: (ref ByteArray) ->
(ref null ByteArray)` and a closed operation prefix. Native kernel operations
return certified data. The emitted Date decoder checks complete byte extents,
offset bounds, candidate normalization, ordering and forward equations before
using the earliest overlap candidate. Gap containment checks subtract bounded
offsets from the proved local coordinate, preventing overflow from malformed
transition words. Date selects the certified before offset for a gap and applies
TimeClip after UTC selection. It does not discard candidates outside Instant
range before making the Date selection.

Native offset and candidate bounds derive from the same pure Rust constructors
that validate provider data. The shared codec owns private UTF-8 wire bytes;
JavaScript results use immutable UTF-16 StringValue records. UTC/ISO formatting,
generic toJSON and Date @@toPrimitive retain whole completions. Date-to-Temporal
widening converts integral milliseconds to GC BigInt before multiplying by one
million and publishes a complete intrinsic TemporalInstantObject.

Existing Engine Date system-zone controls cover fixed offsets, primary aliases,
whole-second history, gaps, overlaps, TimeClip edges, created Realms, workers and
injected clocks. They remain unrun. Formatting and source hashes cannot establish
runtime support, and no README conformance counts have been changed.
