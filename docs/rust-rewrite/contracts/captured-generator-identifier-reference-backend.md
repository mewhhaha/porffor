# Captured generator Identifier References

Ordinary generator plain, compound and logical Identifier assignments acquire
the real Reference before any selected RHS evaluation or suspension. Closed
ReadBeforeRhs capture performs GetValue for compound/logical assignments;
WriteOnly capture never reads a plain-assignment target. Plain assignment checks
TDZ, immutability and strict unresolvable failure only at the later PutValue,
after the whole RHS. HasBinding and unscopables selection still run before it.
The old whole JavaScript value remains in its existing activation binding. A
separate compiler-private BindingCell field retains the immutable native
EnvironmentIdentifierReferenceRecord, containing the exact key, Reference kind,
selected Environment Record or BindingCell, named entry and whole object base.
It has no JavaScript value tag, manual heap carrier or second resolver.

Only the lowering-owned CapturedIdentifierReferenceIr can request this transport.
The private backend constructor checks its actual owned activation slot and
ordinary resumable frame. Capture storage comes from InvocationFrame's stable
INVOCATION_ENVIRONMENT, even when the current environment is a suspended block
or per-iteration record. Declarative capture roots the actual selected cell;
runtime-visible capture uses the existing ResolveBinding algorithm. Ordered with
selection runs once, then capture uses the selected object or actual declarative
fallback. A global fallback starts the same resolver at the actual global record.

Put restores that saved record and clears its continuation edge before evaluating
the final RHS value or invoking PutValue. It never repeats environment-chain
resolution or with HasBinding/unscopables selection. Existing named-record and object writes keep
their own required property checks, strictness, Proxy hooks and abrupt identity.
Direct-cell GetValue follows the existing import target and TDZ check; PutValue
checks the original cell's initialization and mutability, including the sloppy
named-function-self no-op rule, before writing its whole stored value. A skipped
logical assignment clears the native edge without PutValue.

A resolved global Reference retains the Global Environment Record across RHS
execution and suspension. The shared Get/Put owner rechecks that record's
declarative name table when the operation runs, as required by
[Global Environment Record SetMutableBinding](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-global-environment-records-setmutablebinding-n-v-s)
and GetBindingValue. A fresh Script can introduce a lexical shadow after the
original HasBinding; the later operation then delegates to that lexical binding,
including its TDZ or immutability failure. An originally unresolvable Reference
stays unresolvable. Ordinary global identifier reads and writes use the same
resolver and operation owners. The global object's Realm, complete key and
selected record remain GC references; no new environment-chain search occurs.

Ordinary plain assignments captured from a `with` scope also finish the global
fallback's ResolveBinding before evaluating the RHS. The closed
`AssignWithGlobalFallback` operation owns ordered object selection and the single
RHS; its emitter retains either the selected object Reference or the complete
global/unresolvable Reference. It performs no target GetValue. Strict missing
References therefore stay missing even if the RHS creates a global property,
and global HasBinding throws prevent RHS effects. PutValue retains the existing
live-record checks. New strict/sloppy controls cover property creation/deletion,
unscopables changes, RHS throws and observable Proxy Has/Set ordering. Joined
validation is recorded in [CONTINUE.md](../../../CONTINUE.md).

The ordinary and resumed global assignment controls cover fresh Script `let`,
`const` and uninitialized lexical shadows, strict unresolved References, and a
Proxy HasBinding trap that installs a lexical binding before GetValue. Additional
read controls retain the second Object Environment HasProperty check, its
strict/sloppy missing-binding result and the identity of a thrown trap value.
These added controls remain authored but unexecuted.

An abandoned expression retires its saved Reference before a source Throw reaches
a catch/finalizer and before a committed Return enters a finalizer. One typed
BindingCellTable loop clears only CAPTURED_IDENTIFIER_REFERENCE in that invocation.
It preserves all JavaScript values and other continuation fields. Called-function
Throws caught inside that callee never clear the caller's activation. Entry failure
before invocation-environment publication safely finds no cells. Normal Yield
keeps the native edge; delegated return with done:false also keeps it until the
delegate actually completes or propagates an abrupt completion.

The existing two compound/logical Engine cohorts exercise real JS -> IR -> Wasm execution: strict
and sloppy global/declarative/logical cases, GC while suspended, exact Proxy with
selection order, TDZ precedence, immutable writes, and injected Return/Throw.
Their fixture successor adds caught RHS abrupts, yielding handlers/finalizers,
fresh assignments after abrupt transfer and delegated return(done:false).
The separate [plain assignment source contract](generator-plain-identifier-assignment.md)
and two Engine cohorts cover the consumed write-only successor, including
reference selection changes during suspension and delayed assignment failures.
IR admission controls and the shared native GC schema belong to coordinated
successor packets. These controls are authored but unrun; no compilation, runtime,
guard execution or conformance result is claimed by this source-only packet.
