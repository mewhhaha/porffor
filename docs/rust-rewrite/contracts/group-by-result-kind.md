# GroupBy result kind

Status: implemented and focused-verified for the shared Wasm-AOT
`Map.groupBy` / `Object.groupBy` emitter, 2026-08-26.

## Closed domain

`GroupByResult` has exactly two inhabitants: `Map` and `Object`. The two public
compiler wrappers construct only their matching result kind. The kind does not
implement `PartialEq` or `Eq`, and its shared emitter contains no equality,
inequality, Boolean or `is_*` projection.

The emitter projects the kind through exactly eleven direct exhaustive matches.
Seven select operation-specific diagnostics. The remaining four select:

1. whether the executing Realm's intrinsic Map prototype is loaded;
2. whether the result is a branded Map or a null-prototype ordinary object;
3. whether callback keys receive `-0` normalization or property-key conversion;
   and
4. whether groups use Map entries or enumerable own properties.

Adding another result representation therefore requires naming its behavior at
every semantic decision before the compiler builds. It cannot inherit Map or
Object policy from an `if` / `else` default.

## Durable regression

`group_by_result_structure.rs` bounds the shared emitter through the following
Map method. It owns the exact two-variant declaration, absence of equality
capability, two-producer census, eleven exhaustive projections, paired
diagnostic census and the distinct key-conversion, allocation, brand and
storage witnesses.

The finite CLI fixture distinguishes the two result representations. It checks
the Map brand and prototype, SameValueZero zero grouping and Map storage, then
checks the Object result's null prototype, safe `__proto__` property, Symbol
key and string-converted numeric key.

```sh
cargo test -p lila-aot-wasm --test group_by_result_structure --quiet
cargo test -p lila-cli --test cli iterator::run_wasm_backend_distinguishes_map_and_object_group_by_results -- --exact --test-threads=1
```

The shared semantic golden passes `2/2` in 717.58 seconds with 674 dumps. It
adds this witness plus the independent Promise combinator Realm and Temporal
overflow-options witnesses, removes none and leaves all 671 retained dumps
equal after accounting normalization. Broad Test262 verification remains
deferred.

This source-equivalent type closure changes no iterator protocol, callback
order, IteratorClose behavior, Realm selection, collection storage layout or
published conformance count. The complete Map and Object grouping Test262
leaves are not refreshed by this lane.
