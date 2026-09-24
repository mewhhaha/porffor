# Error builtin dispatch ownership

Status: source-equivalent T24 ownership invariant implemented with focused
structural verification, 2026-08-27.

## Scope

This contract owns the Wasm-AOT dispatch boundary for `Error.isError`, the nine
Error-family constructors, `Error.prototype.toString`, and the two functions of
the `Error.prototype.stack` accessor (proposal-error-stack-accessor). It does
not own the algorithms inside those branches, Error prototype selection,
native-error metadata, or intrinsic publication.

## Rust invariant

The private, non-derived `ErrorBuiltin` domain is created only by thirteen fixed
entries inside `builtins/errors.rs` and consumed by the sole raw error-family
emitter. Standard dispatch cannot import, construct or pass it. The authority
has no clone, copy, debug, equality, default, wildcard or Boolean projection. A
caller cannot transfer the same dispatch authority twice, and a future row
cannot inherit an existing branch through equality plus a default.

The domain has five rows: `IsError`, `Constructor(NativeErrorKind)`,
`PrototypeToString`, `PrototypeStackGetter` and `PrototypeStackSetter`. There
are nine exact constructor producers. The standard
dispatcher also has one exact producer for each non-constructor row. The
consumer's outer match is exhaustive. Its constructor arm immediately exhausts
all nine `NativeErrorKind` rows, retaining the existing distinct AggregateError
and SuppressedError paths and the shared seven-family message-error path.

Removing the unused derived capabilities changes no Wasm instruction, local,
branch, diagnostic, or observable behavior. The invariant matters because the
dispatch value selects algorithms with different argument orders, allocation
phases, and observable receiver semantics; emitting it twice is not a valid
operation.

## Verification and non-claims

The Rust-lexical structure guard ignores comments and every normal, raw, byte,
C-string, character and raw-identifier form. It pins the exact declaration,
the 20-mention recursive source census, the sole two-level exhaustive consumer,
all thirteen fixed entries and their exact standard calls. The focused target
passes `4/4`.

This is a source-equivalent ownership closure. Runtime witnesses remain owned
by the existing Error, AggregateError and SuppressedError fixtures. It does not
claim the full Error or NativeErrors trees, broader T24 completion, a Wasm
golden result, or a published conformance-count change.

Batch AQ makes the raw `ErrorBuiltin`, its `NativeErrorKind` constructor
selection and `emit_error_builtin` consumer private to `builtins/errors.rs`.
Eleven fixed sibling-visible entries expose only exact Error-family semantics.
The former 216-line raw policy/consumer selection and 50-line dispatcher
selection have SHA-256
`c8eed6033e7a9f0b7f942ff36a02d35eff1be4c7312dde21c240b68ec45bc8f6`
and `61d7263402899afaea30eb5d4d445698c9427945dbcdc8824946840d25c50e8a`;
the eleven fixed entries occupy 101 lines with SHA-256
`7354f5926ebfb194728e7dda61085162dc9323d3b42f71e804c333ed3c851c0e`.
This source-equivalent boundary tightening claims no new Error behavior. At
the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the strengthened
ownership structure target passes `4/4`, and the exact constructor-properties,
cross-realm `Error.isError` and `Error.prototype.toString` CLI controls pass
`3/3`. No Batch AQ Test262, semantic-golden or published conformance-count
result is claimed.

The 2026-09-24 `Error.prototype.stack` accessor adds the `PrototypeStackGetter`
and `PrototypeStackSetter` rows, each with one fixed entry and one exact
standard dispatch mapping. The getter body is `errors/prototype_stack.rs`; the
setter row forwards to the shared `SetterThatIgnoresPrototypeProperties`
emitter (`builtins/object/setter_ignoring_prototype_properties.rs`) with the
`ErrorStack` home. The historical Batch AQ hashes above describe the
eleven-entry source and are not re-pinned.
