# Global numeric call coercion ownership

The coercing global `isFinite` and `isNaN` calls always retain their evaluated callee and argument list in the actual invocation IR. They use the existing emitted global-numeric builtin, including a missing first argument and all ignored extra-argument effects. Literal strings and fresh arrays carry no proof that would authorize replacing this invocation with a Boolean.

The consumed conversion owner is `builtins/global_numeric.rs::emit_global_numeric_builtin`. It calls the shared `emit_value_to_number_payload` path, preserving object ToPrimitive hooks and original abrupt values, then the existing StringToNumber helper when the primitive is a String. That helper owns ECMAScript Unicode whitespace, complete numeric prefixes and decimal grammar. The ordinary invocation and numeric Realm owners preserve the builtin's execution Realm policy.

The IR literal-fold owner returns None for these two globals. Its former Rust-only string parser and unproved array conversion helpers are retired. Both real call-lowering consumers therefore fall through to their existing emitted invocation. Type-only Number predicates, Annex B unescape and static RegExp specialization retain their existing owners; this batch introduces no new parser, conversion representation, dependency or evaluator.

The existing IR Number-coercion target gains controls for retained global callee/literal arguments, changed Array prototype hooks and observable ignored arguments through direct and global-object method calls. The existing Engine string-numeric target gains behavioral controls independently. Source implementation and controls are dry-authored; the ref77 combined all-target Rust type check passed. Emitted Wasm and runtime acceptance remain mandatory checks. Full T20 remains open.


The 2026-10-04 atomic T05 draft replaces this owner's former scalar/tag pair
with whole GC `ValueLocals` and `CompletionLocals`. It consumes the shared
conversion result only on Normal and publishes a whole Boolean or the original
Throw. The argument vector, conversion and cleanup all retain strong reference
identity. The added GC control is source only; no new type or runtime proof has
run. Ref77 remains historical predecessor evidence, and full T20 remains open.
