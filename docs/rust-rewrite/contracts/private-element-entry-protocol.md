# Private-element heap entries: one closed storage protocol

## Decision

The Wasm backend stores private elements through one closed
`PrivateElementHeapKind` wire domain and one closed legal-entry domain. It does
not assemble a heap row from an arbitrary integer plus independent optional
receiver and value tuples.

ECMA-262 6.2.10 gives a PrivateElement the closed `field`, `method`, or
`accessor` kind. The algorithms in 7.3.26-7.3.31 then find, add, read, and write
those records. Lila preserves those semantics while sharing each private
method or accessor function across all instances of one class evaluation. That
sharing gives the backend five storage rows:

| Heap row | Receiver | Value | Wire word |
| --- | --- | --- | --- |
| `Brand` | present | absent | 0 |
| `Field` | present | present | 1 |
| `SetterDefinition` | absent | present | 2 |
| `MethodDefinition` | absent | present | 3 |
| `GetterDefinition` | absent | present | 4 |

The first two are receiver rows. A private field stores its value directly;
an installed method or accessor stores one `Brand` row on the receiver. The
last three are definition rows keyed only by the private-name token. They hold
the callable values shared by every receiver bearing that brand. A paired
accessor therefore has one receiver brand and two definition rows.

This is a backend storage protocol, not a second specification-level private
element taxonomy. In particular, `Brand`, `GetterDefinition`, and
`SetterDefinition` are implementation rows rather than new ECMAScript private
element kinds.

## The invalid Cartesian product

The former writer accepted these axes independently:

```text
receiver: absent | present
kind:     any u64
value:    absent | present
```

Even if `kind` is restricted by convention to the five known constants, that
is twenty constructible combinations for five legal rows. A getter row with a
receiver and no callable value, a field row without a receiver, or a brand row
with a value all compiled. The definition lookup also accepted the same raw
integer domain, so it could be called with a receiver-only `Brand` or `Field`
tag.

The compiler owns every producer today, but that does not make the states
valid. A new or edited producer could silently write a row that private read,
write, or brand-check later interpreted as a different semantic operation.

## Construction

`PrivateElementHeapKind` owns the stable `0..=4` encoding. Its `wire_word`
projection is an exhaustive match rather than a representation cast. Adding a
kind therefore requires choosing its wire representation at compile time.

The row writer accepts a legal-entry enum whose variants carry exactly the
locals their row needs:

```text
Brand(receiver)
Field(receiver, value)
SetterDefinition(value)
MethodDefinition(value)
GetterDefinition(value)
```

Exhaustive projections derive the stored kind, optional receiver, and optional
value. There is no public tuple-field construction, `Default`, raw-kind
constructor, or three-axis writer left for a caller to misuse.

Definition lookup accepts the narrower
`PrivateElementDefinitionKind::{Setter, Method, Getter}` domain. Receiver-only
kinds cannot enter that operation.

## Consumption

Private read and write first find the receiver row for `(receiver, token)`.
That row is valid only when its kind is `Field` or `Brand`:

- `Field` reads or replaces the stored value;
- `Brand` redirects to the shared method/getter/setter definitions;
- any other wire word is compiler-owned heap corruption and traps.

The trap is not an ECMAScript exception path. No valid program can manufacture
or mutate these private heap records, and treating a corrupt definition row as
a brand would hide a compiler error behind plausible JavaScript behavior.

The method/getter lookup order and all existing brand, extensibility,
duplicate-installation, abrupt-completion, and field-initialization ordering
remain unchanged.

## Enforced invariants

1. The five wire words remain unique, dense, and stable at `0..=4`.
2. A receiver is present exactly for `Brand` and `Field` rows.
3. A value is absent exactly for `Brand` rows.
4. Definition lookup can name only setter, method, or getter rows.
5. Read and write accept only field or brand rows from receiver lookup.
6. Adding a legal row requires updating exhaustive kind/receiver/value
   projections; omission is a compile error.

## Verification boundary

A focused Rust test fixes the wire words and the receiver/value projections.
Existing CLI fixtures already exercise every legal row, paired accessors,
static private callables, wrong-direction access, abrupt getters, duplicate
installation, and non-extensible receivers. This retyping adds no duplicate
JavaScript fixture.

Cargo and Test262 verification remain deferred to the centralized batch after
the active low-memory matrix releases the build/runtime lease.

## Nonclaims

This seam does not add auto-accessors, decorators, a new private-name model,
cross-realm proof, or a new object representation. It does not change the
private-element algorithms or broaden the supported class surface. It makes
the backend's existing five-row representation explicit and rejects corrupt
states; T09 remains in progress.
