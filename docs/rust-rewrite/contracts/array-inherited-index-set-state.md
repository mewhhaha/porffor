# Array inherited-index Set state authority

## Closed domain

`ArrayInheritedIndexSetState` is the sole compiler authority for the result of
walking an Array receiver's prototype chain before an indexed write. Its five
states are `Unhandled`, `Setter`, `OrdinaryRejected`, `Handled` and
`ProxyRejected`.

The capability-free `ArrayInheritedIndexSetState` implements no clone, copy,
debug, default, comparison, ordering or hashing capability. It has no Rust
numeric representation or discriminants. Its only semantic projection borrows
the state and exhaustively maps those five variants to the existing Wasm-local
codes 0 through 4. Adding a sixth state therefore fails to compile until the
wire mapping is reviewed; a state cannot be copied or compared into a second
decision authority.

The raw Wasm local remains necessary because the generated program chooses its
state at run time. The Rust compiler, rather than an implicit enum discriminant,
now owns every value that may be written to that local.

## Producer and consumer census

Product Rust source contains 19 exact type-name occurrences: the declaration,
the projection impl, the read-only standard dispatcher import and 16 code
producers. The producer distribution is:

- `Unhandled`: 2;
- `Setter`: 3;
- `OrdinaryRejected`: 6;
- `Handled`: 2; and
- `ProxyRejected`: 3.

There are exactly two calls to `emit_array_inherited_index_set_state`: ordinary
Array index assignment and the canonical dense-Array Push branch. Assignment
retains strict-sensitive `OrdinaryRejected` and `ProxyRejected` handling. Push
retains unconditional built-in failure semantics. `standard.rs` remains a
read-only consumer of the closed projection.

## Source-equivalent preservation

Only the Rust declaration and numeric projection change. The assignment body,
the prototype-chain walker and canonical Push compiler remain byte-identical.
Their frozen raw-body SHA-256 values are:

- assignment: `e91057b92ce1a9491c657ab22936fdebe348d3ead110abf9cacef79c347b23d3`;
- inherited-index state emitter: `c4806b5db178523368c877c3a663d0fe77ca4963022ac46f1a2602a083efb089`;
- canonical Push branch: `c954cac5f939488f2cb6d07e5b9d70fba3224d33ec57c708570e6759856cf6c8`.

The exhaustive match returns the same 0, 1, 2, 3 and 4 constants in the same
variant order. It emits no new Wasm instruction and changes no observable Set,
setter, Proxy, strict-mode or current-realm error behavior.

## Durable evidence

The existing `object_write_proxy_realm_structure.rs` target now pins the exact
capability-free declaration, borrowed exhaustive mapping, absent wildcard and
capabilities, 19-name/16-producer census, per-variant producer counts and the
two consumer calls. Its existing tests continue to distinguish assignment's
strict-sensitive Proxy rejection from Push's unconditional rejection and to
pin current-function-realm TypeErrors.

The existing exact CLI controls are
`object::proxy_set_errors_use_the_borrowed_builtin_realm`, which reaches
inherited Proxy rejection through Array fill and Push, and
`array::run_wasm_backend_expands_all_array_push_arguments_before_appending`,
which controls ordinary Push. The most direct vendored inherited-setter leaf is
`staging/sm/Array/set-with-indexed-property-on-prototype-chain.js`.

At the shared Batch AD checkpoint, `cargo xc` is green, the
`object_write_proxy_realm_structure` target passes `5/5`, and the two exact CLI
controls each pass `1/1`. The exact inherited-setter Test262 leaf passes both
sloppy and strict Wasm-AOT executions (`2/2`). No semantic golden was run for
Batch AD.

## Nonclaims

This closure does not type Wasm locals, change the prototype walk, add a new
Array optimization, alter `standard.rs`, or establish complete Array Set
conformance.
