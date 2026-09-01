# Temporal PlainTime field authority

Status: implemented with focused structure verification, 2026-08-27.

## Scope

This contract owns the six `Temporal.PlainTime` core fields where their
declaration index, record offset, valid maximum, nanosecond scale and prototype
accessor meet. It does not own property-bag read order, string parsing,
rounding, arithmetic or the complete Temporal API.

## Semantic law

PlainTime stores and loads `hour`, `minute`, `second`, `millisecond`,
`microsecond` and `nanosecond` in declaration order. `RejectTime` accepts each
field from zero through its valid maximum; `RegulateTime` uses the same maximum
when constraining. Converting a record to nanoseconds uses the corresponding
unit scale. Each prototype accessor selects the same field identity used by
those storage, validation and scalar-conversion paths.

## Rust invariant

The existing closed `TemporalTimeUnit` domain is the sole PlainTime core field
authority. Three exhaustive projections bind every unit to its field index,
heap record offset and valid maximum; the domain's existing exhaustive
nanosecond projection supplies the scale. Allocation, record loading,
rejection, constraint and scalar conversion all walk `TemporalTimeUnit::ALL`
and select each local through the unit's field index instead of pairing
independent positional arrays.

The standard-builtin dispatcher now constructs the exact unit at each of the
six accessor arms. The shared accessor accepts only `TemporalTimeUnit`, so a
several-hundred-variant `StandardBuiltinId` can no longer reach a catch-all
compiler panic. Adding a wall-clock unit therefore requires the compiler to
choose every PlainTime field projection before it builds.

The structure regression pins the three complete mappings, removes the old
offset/maximum/name arrays, owns all five ordered consumers, and checks the six
typed accessor producers and catch-all-free consumer.

## Verification and non-claims

The focused source structure target is the verification owner for this source-
equivalent change. Targeted Rust formatting and diff checks cover the owned
files. This does not change emitted Wasm, observable values, diagnostics or
Realm selection.

The independently owned alphabetical property-bag table remains the authority
for observable property-bag read order and is deliberately outside this
invariant. This change does not complete PlainTime parsing, arithmetic,
calendars, time zones or T22.
