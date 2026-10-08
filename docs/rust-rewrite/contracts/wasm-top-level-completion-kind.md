# Wasm top-level completion kind

## GC completion draft — 2026-10-04

The atomic T05 draft changes the host boundary to a five-result Main tuple:
I32 tag, I64 scalar, nullable rooted EqRef, I32 kind and I32 target. Main is the
sole result authority; duplicate exported result-tag/kind globals are retired.
A `RootScope` begins before the call and remains live through module status,
GC value decoding and legacy or structured publication.

The compiler's consumed host schema checks Normal/Throw and a zero target once.
The Engine's private `GcObservedCompletion` then checks the tag/scalar/reference
contract once and owns the decoded value and non-derived
`WasmTopLevelCompletionKind::{Normal, Throw}`. Its private fields and consuming
projection prevent bypassing the checked observation constructor. The same
three exhaustive consumers retain thrown-text access, legacy result/error and
structured normal/throw publication. Diagnostic String globals are nullable GC
references and remain separate from Main's result authority.

String decoding uses the actual schema field/array storage and retains every
UTF-16 code unit. BigInt decoding retains unsigned I64 limb bits and emits
canonical signed decimal. Object/Symbol observation remains category-only;
legacy diagnostics no longer expose linear heap addresses. Meaningful native
controls cover rooted String/BigInt collection survival, canonical domains,
malformed completion rejection and release of native shared byte resources.
The maintained lexical guard follows the actual rooted call/check/consumer
order. These controls and the complete atomic cutover remain unverified; no
compilation or runtime result follows from this draft.

## Historical scalar boundary

The following source-equivalent checkpoint describes the earlier ABI. Its
passes and nine-mention census do not accept the GC draft.

The engine validates the raw exported `completion_kind` once, at the Wasm
trust boundary, and converts it into the private
`WasmTopLevelCompletionKind::{Normal, Throw}` domain. The domain derives no
cloning, copying, debugging, equality, ordering, hashing or default capability.

Three exhaustive consumers own every consequence of that parsed kind. Legacy
execution first decides whether thrown Error text may be read, then decides
whether to return a successful legacy outcome or an uncaught-throw engine
error. Structured execution consumes the kind to construct either
`ObservedCompletion::Normal` or `ObservedCompletion::Throw`. No consumer
projects the domain back to an unlabeled Boolean.

The focused structure guard lexically excludes Rust comments and literals,
pins the private declaration and nine production mentions, proves the single
raw-code parser precedes the three exhaustive consumers, and binds each variant
to its exact legacy and structured consequence. The existing
`observed_wasm_completion_is_typed_and_captures_print_once` and
`observed_wasm_throw_stays_distinct_from_engine_error_and_legacy_adapter`
tests witness normal structured observation and throw behavior across both
public execution modes.

This source-equivalent ownership closure changes no completion ABI or runtime
behavior. It does not replace the tuple completion convention, add completion
kinds, or complete the planned `exnref` migration.
