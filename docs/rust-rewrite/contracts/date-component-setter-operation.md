# Date component-setter operation

Status: implemented and focused-verified for the Wasm-AOT Date
component-setter compiler family, 2026-08-26.

## Closed operation domain

`DateComponentSetterOperation` names the seven component replacement shapes:

- `FullYear`;
- `Month`;
- `Date`;
- `Hours`;
- `Minutes`;
- `Seconds`; and
- `Milliseconds`.

The operation deliberately implements neither `PartialEq` nor `Eq`. Setter
semantics may not be projected through equality, a Boolean, a wildcard or an
`is_*` helper. Every semantic decision is a direct exhaustive match, so adding
an eighth replacement shape requires reviewing the complete setter algorithm
before the compiler builds.

This domain does not represent local versus UTC time. The current Date backend
has the documented UTC/fixed-offset limitation, and each local/UTC setter pair
currently emits identical component arithmetic. A time-coordinate domain
belongs to the future default-time-zone boundary once it has a real semantic
consumer; adding it here would make an unused state look implemented.

## Producer census

The `StandardBuiltinId` dispatcher has fourteen producers in seven exact pairs:

| Operation | Local producer | UTC producer |
| --- | --- | --- |
| `FullYear` | `DatePrototypeSetFullYear` | `DatePrototypeSetUtcFullYear` |
| `Month` | `DatePrototypeSetMonth` | `DatePrototypeSetUtcMonth` |
| `Date` | `DatePrototypeSetDate` | `DatePrototypeSetUtcDate` |
| `Hours` | `DatePrototypeSetHours` | `DatePrototypeSetUtcHours` |
| `Minutes` | `DatePrototypeSetMinutes` | `DatePrototypeSetUtcMinutes` |
| `Seconds` | `DatePrototypeSetSeconds` | `DatePrototypeSetUtcSeconds` |
| `Milliseconds` | `DatePrototypeSetMilliseconds` | `DatePrototypeSetUtcMilliseconds` |

No other producer exists. The shared emitter accepts only the operation; it
cannot receive an unrelated `StandardBuiltinId`.

## Exhaustive semantic projections

The emitter contains exactly five `match operation` projections:

1. maximum argument count and therefore the ordered `ToNumber` sweep;
2. invalid receiver-date initialization;
3. whether component replacement executes for an invalid receiver date;
4. the mandatory and optional components replaced; and
5. whether the computed value is written for an invalid receiver date.

The operation matrix is:

| Operation | Maximum arguments | Replaced components | Invalid receiver date |
| --- | ---: | --- | --- |
| `FullYear` | 3 | year, optional month/date | seed the epoch components and repair |
| `Month` | 2 | month, optional date | preserve NaN |
| `Date` | 1 | date | preserve NaN |
| `Hours` | 4 | hour, optional minute/second/millisecond | preserve NaN |
| `Minutes` | 3 | minute, optional second/millisecond | preserve NaN |
| `Seconds` | 2 | second, optional millisecond | preserve NaN |
| `Milliseconds` | 1 | millisecond | preserve NaN |

The separate `standard_builtin_length` table remains a read-only global
`StandardBuiltinId` projection because it serves every builtin family. The
structure guard pins its fourteen Date lengths to this operation matrix rather
than exposing a Date-private type through planning.

Before this closure, the emitter accepted the entire builtin catalog. Two raw
matches repeated the fourteen identities and ended in `unreachable!()`, while
one `is_full_year` Boolean controlled three invalid-date decisions. A partially
added setter could compile and inherit the non-FullYear default until compiler
execution reached a panic. The closed domain makes that partial migration a
Rust compile error.

## Durable regression

`crates/lila-aot-wasm/tests/date_component_setter_operation_structure.rs`
pins:

- the exact seven-variant declaration without equality capability;
- exactly five exhaustive operation projections;
- absence of raw setter builtins, Boolean projections, wildcards and
  `unreachable!()` in the emitter;
- all fourteen dispatcher producers and their seven exact pairings; and
- parity between emitted argument counts and the read-only builtin-length
  matrix.

These are bounded source-structure mutation guards. They supplement rather
than replace behavioral execution.

## Focused evidence

The existing `wasm_date_component_setters.js` CLI fixture covers six operation
shapes, both dispatcher families, optional arguments, overflow normalization,
FullYear invalid-date repair and ordinary invalid-date propagation. The pinned
`built-ins/Date/prototype/setUTCMinutes/this-value-valid-date.js` Test262 leaf
supplies the one operation shape absent from that fixture. The existing
`setUTCMonth/arg-coercion-order.js` leaf now runs its unchanged pinned Test262
source with the full merged `assert.js` and vendored `compareArray.js`
preludes. An exact materialization invariant pins both helper origins and the
concatenated source bytes so this coercion-order witness cannot silently return
to a handwritten rewrite.

The bounded structure target passes `4/4`, the exact existing CLI fixture
passes `1/1`, and the exact `setUTCMinutes` and unchanged-source `setUTCMonth`
Test262 leaves each pass both ordinary executions `2/2` with every failure
bucket at zero. The `setUTCMonth` provenance invariant passes `1/1`. Focused
compilation emitted pre-existing warnings; the initial structure run exposed
and the final source removed one transient private-interface warning from this
lane.

This source-only closure reserves the same locals and emits the same selected
instruction sequence. The shared semantic golden passes `2/2` in 722.99
seconds with 678 dumps; this closure adds no fixture, and all 674 retained dumps
are equal after accounting normalization. There is no inventory or
published-count change.

## Nonclaims

This boundary does not implement a default time zone, local-time conversion,
DST behavior, Date parsing, getter closure, `setTime`, Annex B `setYear`, Realm
changes or broader Date/Temporal conformance. It does not close T22.

## Batch AU dispatcher boundary

The seven-case operation is now a private `DateComponentSetterOperation` with
no derived capabilities, and the raw setter emitter is private to `date.rs`.
It exposes seven fixed Date setter entries to the fourteen local/UTC catalog
IDs; standard dispatch can neither construct nor pass the raw operation. The
frozen 306-line domain/emitter selection has SHA-256
`53813c73ebb92bdaa9541b57c83694c11c4f3dcc214c8cc27f056eb980d44240`.
Restoring only the former derive and visibility reproduces that source exactly.
At the 2026-08-28 Batch AU checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, the exact setter CLI fixture passes `1/1`, and
the focused `setUTCMinutes` leaf passes both sloppy/strict Wasm-AOT executions
`2/2` with every failure bucket at zero. This source-equivalent boundary claims
no new Date behavior, local-time/default-time-zone support, broader conformance
or published conformance-count change.
