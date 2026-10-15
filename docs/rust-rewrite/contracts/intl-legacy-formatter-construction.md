# Intl legacy formatter construction

The prepared successor enables ECMA-402's normative optional legacy constructor
mode for DateTimeFormat and NumberFormat. The native verification of the earlier
formatter implementation does not verify these new source bytes. This component
requires fresh compilation, focused Engine controls and unchanged pinned replay
before any passing-test count changes.

The authority is the official ECMA-402 source snapshot
[`730ca2b88c305862db7be2f83c510dcca33ab48c`](https://github.com/tc39/ecma402/tree/730ca2b88c305862db7be2f83c510dcca33ab48c/spec):
`intl.html` defines `%Intl%.[[FallbackSymbol]]`; `datetimeformat.html` defines
`ChainDateTimeFormat` and `UnwrapDateTimeFormat`; `numberformat.html` defines
`ChainNumberFormat` and `UnwrapNumberFormat`. This is the algorithm source
snapshot, not a claim that the separately compiled ECMA-262 identity changed.

## Constructor order

Construction retains the existing prototype reservation, locale observations,
option reads, coercions, provider selection and complete private-record
initialization. Only afterwards does the constructor inspect its original raw
`NewTarget` and `this` for chaining. Date locale methods and Temporal's internal
DateTimeFormat construction do not enter this branch.

When raw `NewTarget` is undefined, the constructor performs
`OrdinaryHasInstance` with its executing function Realm's actual intrinsic
formatter constructor. It does not invoke `Symbol.hasInstance`, consult a
mutable public `Intl`, or substitute the caller's Realm. If that operation is
true, `DefinePropertyOrThrow` installs the fully initialized fresh formatter on
the original `this` at the Realm's fallback Symbol, with writable, enumerable
and configurable all false, then returns that original `this`. Otherwise it
returns the fresh formatter. Prototype and proxy definition throws retain their
exact completion, after initialization has finished. A false proxy definition
result and an incompatible existing nonconfigurable property throw TypeError.

## Method boundary

| Service | Method | Legacy wrapper behavior |
| --- | --- | --- |
| DateTimeFormat | `get format`, `resolvedOptions` | Unwrap, then require the actual DateTimeFormat brand. |
| DateTimeFormat | `formatToParts`, `formatRange`, `formatRangeToParts` | Require the receiver's own actual DateTimeFormat brand. |
| NumberFormat | `get format`, `resolvedOptions` | Unwrap, then require the actual NumberFormat brand. |
| NumberFormat | `formatToParts`, `formatRange`, `formatRangeToParts` | Require the receiver's own actual NumberFormat brand. |

Unwrap rejects primitives first. An already initialized formatter bypasses both
the prototype walk and the fallback property, including when a later legacy
call installed another formatter on it. Otherwise `OrdinaryHasInstance` uses
the actual method Realm's intrinsic constructor. A true result performs an
ordinary `Get` of that Realm's fallback Symbol on the original receiver. The
original proxy remains both target and receiver, preserving its get trap,
inherited accessor `this` and exact abrupt result. The final brand check rejects
missing, primitive or incompatible fallback values. Parts and range methods
never take this route and reject a wrapper before endpoint coercions.

## Hidden Realm identity

One typed private trailing slot in each Realm's traced `IntrinsicTable` retains
the fallback Symbol. It is shared by both services within that Realm and is a
different identity in every other Realm. Existing public object-intrinsic
indices and the RealmRecord GC layout are unchanged. The object-only intrinsic
loader cannot select this Symbol slot. Its dedicated reader creates an ordinary
unregistered Symbol lazily, using the fixed R-owned description
`IntlLegacyConstructedSymbol`; creation invokes no guest operation. The slot
retains it across programs and collections. There is no Store, global, host or
ambient current-Realm fallback.

The component changes the runtime Realm allocation and requires a freshly
produced runtime image. It must not be paired with a previously compiled image
whose Realm table lacks the private slot. The independent ABI6 diagnostic
packets remain immutable and do not establish acceptance of this successor.

## Verification scope

Four authored Engine controls run ordinary source in both sloppy and strict
modes: return identity and descriptors, shared/per-Realm Symbol identity,
initialization before proxy traps and exact throws, ignored `Symbol.hasInstance`,
ordinary Get with its original receiver, direct parts/range brand rejection,
invalid fallback/primitive receivers, actual constructor and method Realms, and
mutable public `Intl` isolation. These controls are pending native execution.

The six observed target IDs are the sloppy and strict modes of the three
unchanged current-pin DateTimeFormat fixtures
`intl-legacy-constructed-symbol.js`, `intl-legacy-constructed-symbol-property.js`
and `intl-legacy-constructed-symbol-on-unwrap.js`. NumberFormat receives the
same specified mode as the coherent counterpart; this component does not
invent an additional observed Bug count. No fixture, runner expectation,
candidate limit or conformance result is changed.
