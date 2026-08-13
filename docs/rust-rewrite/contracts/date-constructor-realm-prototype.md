# Date constructor realm prototype

`Date` uses `OrdinaryCreateFromConstructor(NewTarget, "%Date.prototype%", …)`
in each of its zero-, one- and multiple-argument construction algorithms.  The
shared `GetPrototypeFromConstructor` rule is observable when
`NewTarget.prototype` is not an object: the fallback is the `%Date.prototype%`
intrinsic of `GetFunctionRealm(NewTarget)`, not the entry realm's Date
prototype.

The normative algorithms are
[`Date ( ...values )`](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-date-constructor)
and
[`GetPrototypeFromConstructor`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-getprototypefromconstructor).
The pinned cases are:

- `built-ins/Date/proto-from-ctor-realm-zero.js`;
- `built-ins/Date/proto-from-ctor-realm-one.js`; and
- `built-ins/Date/proto-from-ctor-realm-two.js`.

The 2026-08-13 current-pin Wasm-AOT baseline passed 75 of the 78 tests in the
`built-ins/Date` matrix leaf.  Those three cases were its exact failures; each
constructed a Date with an other-realm function whose `prototype` was `null`
and observed the entry-realm fallback.

## Closed fallback domain

`OrdinaryDefaultPrototype` is the closed domain of ordinary-object intrinsic
defaults consumed after `GetFunctionRealm`.  `Date` is a member alongside
Object, String, Number and Boolean and maps to exactly one realm-intrinsic slot.
Array remains separate because `%Array.prototype%` has an Array layout and tag.

The generic new-target prototype operation has a distinct
`RequiredResolvedRealmOrdinary` policy.  Its primitive-prototype arm must:

1. resolve the original new target's function realm after the observable
   `Get(NewTarget, "prototype")`;
2. route revoked and invalid realm results before exposing a realm local;
3. load the selected required realm slot, trapping missing bootstrap state;
4. consume the non-copyable prototype witness together with its Object tag.

It may not use the generic `CurrentGlobal`, optional realm-slot/global fallback,
or function-snapshot policies.  The Date constructor selects this policy once,
after all arity-specific time-value computation and before allocating the
branded Date object.  This ordering is observable: the zero-argument host clock
read, one-argument primitive conversion and multiple-argument numeric
conversions all precede `Get(NewTarget, "prototype")`.  An object-valued
`NewTarget.prototype` still wins without consulting the fallback, and its
representation tag is carried with its payload into allocation so Object,
Function and Array prototypes keep their exact identity and behavior.

## Storage and publication

The realm-intrinsics record owns one pointer slot for `%Date.prototype%`.  The
record size, layout descriptor and closed `NonArrayRealmIntrinsicSlot` mapping
move together.  Both producers publish the slot:

- entry-realm bootstrap publishes `DATE_PROTOTYPE_GLOBAL_INDEX`; and
- `$262.createRealm()` publishes its newly allocated `date_prototype_local`.

A missing slot in a resolved realm is an internal bootstrap invariant failure,
not permission to substitute the entry global.  This keeps the stored
prototype identity and the constructor fallback inseparable.

## Observable regression

The durable CLI fixture constructs Dates through an other-realm function with
a primitive `prototype` in all three constructor arities.  It checks the exact
other-realm prototype identity and then borrows `Date.prototype.getTime` to
prove that prototype selection did not replace the Date allocation or its
`[[DateValue]]` brand.  It checks Object-, Function- and Array-valued custom
prototypes so neither the fallback nor allocation can erase their tag.  Proxy
getters pin a single prototype read after one- and multiple-argument coercion,
and a revoked Proxy pins the required TypeError route.  Static source structure
keeps the zero-argument clock read and every arity's final value computation
before prototype resolution.

## Deferred gates

The implementation batch intentionally performs static source and diff checks
only while the low-RAM current-pin matrix owns Cargo and Test262 resources.
After that runner releases them, verification must include:

```sh
cargo test -p lila-aot-wasm date_constructor_realm_ --quiet
cargo test -p lila-cli run_wasm_backend_uses_new_target_realm_for_date_prototype --quiet
./target/debug/lila test262 run built-ins/Date/proto-from-ctor-realm --execution-backend wasm --timeout-ms 180000 --threads 1
```

The final current-SHA closure remains the complete T22 Date/Temporal ladder and
the full low-RAM current-pin publication path.

## Non-claims

This seam does not add a default time-zone provider, DST transitions, another
Temporal class, custom calendar/time-zone protocols, locale behavior or Date
parsing/formatting coverage. It does not claim the complete Date or Temporal
trees are green. It changes Date construction's primitive
`NewTarget.prototype` fallback and preserves the exact representation tag for
explicit Object, Function and Array prototypes through allocation. Date value
calculation, `TimeClip`, Date branding and the call-without-`new` clock path are
otherwise outside this seam.
