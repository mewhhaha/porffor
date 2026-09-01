# Wasm top-level completion kind

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
