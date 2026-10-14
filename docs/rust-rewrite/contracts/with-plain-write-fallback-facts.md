# With plain-write fallback facts

A `with` Object Environment Record can intercept a plain identifier write.
Lowering the located declarative/global fallback is therefore not evidence
that the fallback executes. The shared plain-write owner joins a mutable
declarative binding's previous value with the assigned value, so a selected
Object Record preserves the old value domain while a rejected candidate can
write the new one. Const and uninitialized fallback references keep their
existing lazy failure semantics.

The selected Object Record's PutValue can run a setter after the complete RHS.
The lowerer records those possible source effects after assembling the write.
That invalidates configurable-global presence and mutable value facts created
by the RHS or by lowering a fallback that might never run. Definite ordinary
write proofs and non-configurable declaration presence remain under their
existing owners. Assignment result metadata still belongs to the completed
RHS rather than to either destination.

After leaving `with ({foo: 1}) { foo = 42; }`, bare `foo` must use the existing
GlobalIdentifierRead path and throw ReferenceError if it is unresolvable.
`globalThis.foo` remains an ordinary property Get and may return undefined;
`typeof foo` retains its unresolvable-reference exemption. No unconditional
reader-opcode replacement changes legitimate proven property paths.

Paired IR/native controls cover the original scope-exit trigger, inherited
Object Record properties, unscopables selecting a real global fallback,
abrupt setters, deletion of a global created by the RHS, old/new local value
domains and preservation of a definite ordinary global-write proof. Native
execution and replay of the original 12.10-0-7 ID remain required before this
prepared patch can be called accepted.
