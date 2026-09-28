# Shared semantic invariants

These are the enduring requirements behind retired repair and recovery notes.
Implementation details live in the linked contracts, source and regression
fixtures. Current failing executions and cause confidence belong in the
[failure backlog](../../tasks/README.md), not dated progress narratives.

## References, coercion and completion

Evaluate operands once and retain their values across later effects. Numeric
addition evaluates both operands before ordered primitive conversions; string
concatenation takes precedence over numeric addition. Coercive arithmetic
performs both ToNumeric conversions before the mixed Number/BigInt check.
Discarding a result does not authorize skipping conversion effects or throws.
Prefix/postfix updates keep separate old and new tagged values across BigInt
representation changes and typed-array wrapping.

Property assignments retain the receiver and raw computed key across a
suspended RHS; the shared Reference operation owns conversion, strictness,
the setter receiver and the assignment result. Nullish receiver errors and
key-conversion order must match the ordinary path. Object-environment writes
retain the originally selected binding object, recheck HasProperty when the
algorithm requires it, and preserve Proxy effects. A missing binding must not
silently become a newly resolved Reference.

Declarations preserve Empty completion while retaining initializer effects
and throws. Captured-cell initialization must not publish its internal TDZ tag
as the StatementList result. Global function bindings resolve through their
installed properties, including when also declared with `var`.

See [operation descriptors](operation-descriptors.md),
[suspended References](aot-suspended-references.md), and the
[ordinary assignment contract](contracts/ordinary-property-plain-assignment-reference.md).

## Suspension and environments

Activation storage owns uncaptured values that survive suspension. Captured
Environment Records remain the sole owner of captured cells. Scope-depth
addressing must remain correct after a child environment is created.
Analysis and emission agree on activation environments before calculating
capture hops, including functions whose initial binding list is empty.

Suspended calls retain the selected callee and receiver before evaluating
arguments. Resumption cannot reread a method replaced while an argument yielded.
Class definitions retain their constructor, private context and field keys;
heritage validation and superclass prototype lookup precede computed keys.
Flow facts that observable getters or yields can invalidate must not survive
those boundaries. Abrupt resumption skips pending initialization or writes.

See [generator suspension](generator-expression-suspension.md),
[captured for-await heads](aot-captured-for-await.md), and
[control-flow emission](aot-control-flow-review.md).

## Realms, modules and prepared sources

Intrinsic results, prototypes, callbacks and errors use the Realm required by
their operation. Constructor fallback resolves the canonical intrinsic of
NewTarget's Realm, including bound or proxied constructors. Mutable global
properties and early bootstrap snapshots are not intrinsic identity authorities.

Anonymous default function declarations are initialized once during module
instantiation, so imports and cycles see the same callable before evaluation.
Default-exported expressions retain evaluation-time initialization and TDZ.
Anonymous exports receive the observable name `default`; internal linker names
must never become public function/class names. Establish class display names
before static initialization. Source extraction preserves UTF-16 parser spans
when indexing UTF-8 source text.

Prepared eval and Function bodies are compiled ahead of execution. Runtime
callable identity, converted arguments, exact source tuples, Realm and scope
still control dispatch. Unsupported runtime-generated source is a typed host
capability failure, not a manufactured JavaScript exception that a negative
test can treat as success. Non-string `%eval%` retains its ordinary behavior.

See [Realm intrinsics](realm-intrinsics.md),
[module instantiation](contracts/module-instantiation.md), and
[architecture invariants](architecture-invariants.md).

## Iterators and resource disposal

Iterator acquisition observes the live method and original receiver; a present
nullish or non-callable override cannot be replaced by an intrinsic fallback.
Consumers preserve the acquired Iterator Record through suspension. Closing
occurs exactly when the algorithm requires it, once, with the required completion
precedence. Rejection while unwrapping a synchronous iterator's value uses
synchronous IteratorClose and does not observe a return object's done/value.

Async wrapping of a synchronous disposer ignores its returned value and turns
throws into promise rejections. Realm ownership and suppression order remain
the responsibility of the shared disposal operation.

See [iterator protocol](contracts/iterator-protocol.md),
[IteratorClose](contracts/iterator-close-obligation.md), and
[arguments iteration](aot-arguments-iteration.md).

## Binary data and codecs

Canonical numeric typed-array keys use integer-indexed access. An invalid
canonical index does not fall through to a prototype property. Ordinary Array
reads still preserve inherited getter receivers and expose String keys to Proxy
prototypes. Atomics and view operations revalidate buffer state after observable
conversions at the points required by their algorithms.

ArrayBuffer allocation and byte access select the same memory. Resizable
transfers reserve the maximum capacity required for later growth.

Uint8Array codecs require genuine Uint8Array receivers and exact String
arguments/options. Preserve option-get order, abrupt identity, method-Realm
results, and post-option detached/out-of-bounds checks. Immutable writes reject
at the receiver boundary. Base64 decoding preserves last-chunk policy, padding,
unused-bit checks, exact read/written counts and completed-prefix writes on an
invalid suffix. Hex decoding checks full input length before bounded writes.

See [typed-array witness ownership](contracts/typed-array-witness-use-ownership.md)
and [immutable buffers](contracts/immutable-array-buffer.md).

## Temporal and conformance evidence

Temporal values retain exact epoch seconds/subseconds and validated range proofs.
Wall-clock rounding uses the selected field's origin; higher fields must not
change its half-even parity. Elapsed-duration rounding uses its separate scalar
value. Month-code syntax, calendar suitability, option spelling and option
admission each occur at their specified observable phase. Zone request encoding
derives the identifier from the closed zone value and validates extreme wire
integers without signed absolute-value overflow.

See [Duration fields](temporal-duration-number-fields.md),
[Instant methods](contracts/temporal-instant-methods.md),
[field replacement](temporal-zoned-field-replacement.md), and
[named zones](intl-named-time-zones.md).

Test262 retains root/agent failure provenance and `$DONE(error)` even if a later
engine error also occurs. Runtime-negative expectations compare the final thrown
value's constructor metadata without invoking getters or Proxy traps. A crash,
timeout, unsupported capability or failed async callback cannot satisfy an
expected JavaScript exception. Snapshot identity includes strict/sloppy/module
mode, compiler identity and suite pin; partial replays never stand in for a
complete aggregate. See [taxonomy](conformance-taxonomy.md) and
[snapshot comparison](test262-snapshot-comparison.md).
