# Pending ToPrimitive operation identity

## Current typed boundary

The named operation boundaries now own identity. The public pending ToPrimitive
emitter accepts the closed `ToPrimitiveHint` and exhaustively selects the
registered Default, Number or String helper's typed argument owner. Those
declarations accept the actual `ValueLocals` and caller Environment and return a
`HelperCompletion`; they cannot be invoked with a pure helper's result ABI.

`CompletionLocals` owns the complete value's tag, scalar and GC reference, plus
completion kind and target. Its whole-result copy and the consuming helper
store preserve all five parts. The routed ToPrimitive wrapper then passes that
same completion to its private exhaustive `ToPrimitiveAbruptRoute` finisher.
It does not infer throws from a value tag or discard the original reference.
The other abstract operations select their actual `SpecOperationIr` branches
and preserve the same whole-result contract, including GetV's original receiver.

The ignored `MayThrowOperation` marker remains absent. The retired
`PendingToPrimitiveCompletion` payload/tag pair is absent too; restoring it
would lose the current GC reference and full completion ownership. Operation
identity resides in the typed declaration and consumed call boundary. Arbitrary
raw scalar helper tuples are unrepresentable through that boundary.

Each hint-specific compiler obtains its actual typed parameter owner and emits
the original ToPrimitive inner algorithm. Public callers emit typed helper
calls. The compiler does not call its own facade, so sharing the body cannot
produce an emitter recursion or direct self-call. PropertyKey and the neighboring
value conversions use these shared boundaries as described in
[shared conversion bodies](shared-coercion-helpers.md).

## Controls and verification

The existing three structure-control names now check current abstract-operation
dispatch, whole-completion copy/store/routing, exact hint-to-argument/parameter
mapping and the sole original helper-body compilation route. The emitted
scheduling control separately checks actual non-self direct-call consumers and
retains its original size ceilings. Existing coercion, error-Realm and
abrupt-identity runtime fixtures remain semantic acceptance obligations.

This guard migration is source-authored and unrun. Earlier scalar-representation
verification counts do not verify the current typed GC boundary. It does not add
an operation, completion route, ABI slot or fallback representation.
