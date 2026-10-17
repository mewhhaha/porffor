# ABI8 constructor instance-elements hook

FunctionContext appends one immutable nullable typed funcref:
`INSTANCE_ELEMENTS_INITIALIZER: InitializeInstanceElementsHook?`.
Its exact ABI is `(defining P!, actual constructor FunctionContext!, receiver
Value3) -> Completion5`. Its helper signature shares the core GC recursion
group with FunctionContext; its executable body belongs to the defining P.

Complete callable allocation installs a nonnull hook exactly when the validated
instance plan contains fields, auto-accessor backing fields, or private method
and accessor brands. Truly empty plans and nonconstructor callable plans capture
null. Synthetic static contexts explicitly capture null. Class-element callables
are rebuilt through the same complete allocator using their own nonconstructor
metadata. Existing FunctionContext references retain the immutable slot directly.

`super()` selects `LocalFunction` and `ExternalCaller` explicitly. External eval
reads the actual constructor from the retained shared ACTIVE_FUNCTION_CELL, then
roots its exact FunctionContext and DEFINING_PROGRAM. Nested eval may have a
current or calling P different from that constructor's defining P. The typed
call carries the constructor's P and context together; throws preserve all five
Completion results. BindThisValue updates the existing this/status cells before
initialization; no route replaces any of the four derived binding cells.

One P-owned helper checks its supplied P equals the constructor context's
DEFINING_PROGRAM, reads the actual constructor's immutable ExecutableCode
SOURCE_ID, and dispatches only its own nonempty constructor plans. Publication
and dispatch use the same PlannedFunctionEntry source_id projection. Foreign
FunctionIds are never looked up in an eval program. Unknown SOURCE_ID or a
mismatched P/context pair traps as an internal linkage invariant failure.

Assembly delegates program-helper body selection to a private child while the
physical emission order remains RuntimeHelperId::ALL. The existing 1950-line
assembly cap remains unchanged. NativeDirectEvalFactory registration expects the
separate native-direct-eval component's typed row and compiler method.

Source controls: `python3 scripts/check-abi8-instance-elements.py`.
These source controls and rustfmt do not establish Rust type checking, typed-Wasm
validation, same-Store linkage, GC behavior, or ECMAScript conformance. Compilation
and focused native cases remain required: public fields, private-brand-only
classes, private methods/accessors, throwing initializers, duplicate super,
retained arrows, nested eval crossing P1/P2/P3, and foreign constructor defining-P
ownership. No compiler/native command ran while preparing this component.
