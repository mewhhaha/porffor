# DisposableStack construction and distinct brand

> **Lifecycle status:** the construction boundary in this document remains
> normative. The former method-absence and deferred-disposal statements are
> superseded by
> [`disposable-stack-synchronous-lifecycle.md`](disposable-stack-synchronous-lifecycle.md),
> which owns the complete synchronous prototype surface.

## Specification boundary

This contract covers only `%DisposableStack%` construction and the distinct
brand shared by its lifecycle operations. The methods and synchronous
`DisposeResources` algorithm are specified by the lifecycle contract rather
than duplicated here.

`DisposableStack()` requires a `NewTarget`. Construction performs
`OrdinaryCreateFromConstructor(NewTarget, "%DisposableStack.prototype%",
« [[DisposableState]], [[DisposeCapability]] »)`, then initializes
`[[DisposableState]]` to `pending` and the capability's
`[[DisposableResourceStack]]` to an empty List.

The supported source-free Wasm-AOT boundary therefore requires:

1. exactly one observable `Get(NewTarget, "prototype")`;
2. propagation of an abrupt prototype Get before an instance is published;
3. preservation of an Object-, Function-, Array- or Arguments-valued custom
   prototype and its representation tag;
4. `%DisposableStack.prototype%` fallback for a primitive prototype result;
5. an extensible ordinary result with a distinct `[[DisposableState]]` brand;
   and
6. an initialized pending record with an empty disposal-resource stack.

The cross-realm fallback test whose only new target is constructed by
`new other.Function()` remains a dynamic-source Wasm-AOT exclusion. This seam
uses the current intrinsic global for primitive fallback, matching the already
supported `%AsyncDisposableStack%` boundary without claiming dynamic Function
construction.

## Closed pending-record witness

The backend represents the initialized sync record as a private,
non-`Copy`, `#[must_use]` `PendingDisposableStackRecordLocal`. Its constructor
allocates only the record and writes all four initial fields: `pending`, a null
entry pointer, zero length, and zero capacity. Lifecycle emitters add the typed
entry layout and `disposed` transition without accepting a bare Wasm local as
an initialized sync record.

A single consuming finalizer accepts that witness and the freshly allocated
ordinary object. It installs `OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK`, stores
the record pointer, and publishes the Object result. The brand and record
writes cannot be split between call sites, and an initialized record cannot be
silently dropped without a compiler warning.

The synchronous and asynchronous brands are separate wire values. In
particular, constructing a sync stack must not make it pass
`RequireInternalSlot(..., [[AsyncDisposableState]])`; the existing async
receiver checks must reject a real `%DisposableStack%` instance.

## Catalog and intrinsic shell

The standard-builtin catalog is the sole authority for constructability,
global installation, function identity, arity, and the family installer.
Bootstrap creates `%DisposableStack.prototype%` with `%Object.prototype%` and
links its `constructor`. The resource-management installer defines the complete
prototype surface, including `Symbol.toStringTag`; the common installer owns
the global property and the constructor's `name`, `length`, and non-writable
`prototype` property.

The constructor is direct-returning: its body owns the sole prototype Get,
allocation, record initialization, branding, and result. The generic construct
dispatcher must not preallocate a discarded receiver or perform a second
prototype Get.

## Durable evidence

The construction fixture pins the constructor/global/prototype descriptors,
call-without-`new`, extensibility, custom and primitive new-target prototypes,
one-Get abrupt propagation, and separation from every existing
`%AsyncDisposableStack.prototype%` receiver brand check. The lifecycle fixture
separately owns the synchronous method contract.

Structural Rust tests pin the private witness, its non-`Copy` consuming
finalizer, the distinct brand words, direct-returning construct classification,
and the lifecycle catalog/install boundary that consumes the constructed brand.

The pinned Test262 witnesses are the twelve non-dynamic top-level
`built-ins/DisposableStack/*.js` constructor cases, four prototype-shell cases
(`constructor`, `prop-desc`, `proto`, and `Symbol.toStringTag`), and the five
`AsyncDisposableStack` wrong-brand cases that construct a sync stack before
calling an async method.

## Deferred verification

Central verification owns compilation and Test262 execution. The focused
commands are:

```sh
cargo fmt --all -- --check
cargo check -p lila-aot-wasm --lib
cargo test -p lila-ir disposable_stack_ --quiet
cargo test -p lila-aot-wasm disposable_stack_ --quiet
cargo test -p lila-cli --test cli wasm_disposable_stack_constructor_surface --quiet
./target/debug/lila test262 run built-ins/DisposableStack --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run built-ins/AsyncDisposableStack --execution-backend wasm --timeout-ms 180000 --threads 1
```

## Non-claims

This construction seam does not by itself claim `using` lowering, disposal on
abrupt language completion, dynamic Function construction, or complete
T15/Test262 closure. Synchronous registration, move, disposal, suppression, the
`disposed` accessor, and `Symbol.dispose` are claims of the lifecycle contract
and its verification gates, not of constructor evidence alone.
