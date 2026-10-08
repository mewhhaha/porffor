# Plain async Array-pattern native ownership

Plain async functions and ordinary generators consume one private native
Array-pattern pipeline. Its closed owner accepts only the two checked IR
carriers and requires the function's actual execution kind to match. Async
generators have no admission through this owner.

The complete async source carrier owns acquisition, every original Await pair,
the ordered StepValue/Elision/RestArray statement tape, and the complete body
range. The native entry acquires outside the pattern's own close scope, publishes
the existing `IteratorRecord` in its unique invocation `BindingCell`, and advances
to body entry. A resumed invocation reloads that same record from the original
`InvocationFrame.INVOCATION_ENVIRONMENT`; nested lexical environments never
change the retained cell's identity. No new GC field or JavaScript value encoding
is introduced.

Fresh and resumed body execution rebuild the entire close destination before
an Await rejection is injected. A pending Await returns a Normal suspension and
retains both the iterator and any captured Identifier Reference. Successful
resumption uses the original cached iterator/next and the original selected
Reference; target selection and earlier operand effects do not replay.

The existing iterator helpers own observation and Done. Step/protocol/done/value
failure marks Done and reaches the close scope without a second Close. Target,
default, Put, and rejected-Await failures leave Done false and invoke the existing
IteratorClose helper with the whole pending completion. The helper materializes
the whole result, including return getter/call failure, before this scope retires
its exact iterator edge and dispatches outward. Nested scopes close from inner
to outer; original Throw precedence and normal-close replacement remain the
existing IteratorClose behavior.

Captured Identifier Reference admission and committed Return/Throw retirement
accept Generator or Async invocation owners. Normal Await never retires these
edges. Reference cleanup clears only its own private Reference field; it cannot
retire an array iterator before that pattern's close owner has run. Internal
step/rest result cells use the existing original-invocation writer and preserve
the enclosing StatementList value. Original lexical bindings instantiate through
the complete async body before acquisition and retain their TDZ on resumption.

`async_array_destructuring_structure.rs` authors actual compiler-to-Wasm
validation for both async semantic fixtures and follows the existing BindingCell
edges to the native IteratorRecord and captured Reference records. The Engine
fixtures separately cover normal/rejected Await, targets/defaults, nested Close,
cached next, and lexical/Reference semantics. These controls are source-only in
this batch; no compilation, tests, or runtime validation has been executed.
