# String symbol-hook operation

Status: implemented for `String.prototype.match`, `matchAll`, `replace`,
`replaceAll` and `search`.

## Boundary

The shared symbol-hook emitter accepts only the sibling-visible, non-copyable
`StringSymbolHookOperation::{Match, MatchAll, Replace, ReplaceAll, Search}`
domain. Standard builtin dispatch maps those five exact methods to named rows.
`String.prototype.split` dispatches directly to its separate split emitter and
cannot enter this domain.

Six borrowed exhaustive matches in the shared emitter own the well-known
symbol key, second-argument read, global-RegExp validation, `matchAll` own-hook
probe and retry, and callable-hook argument vector. The private fallback takes
the operation by reference and uses a seventh exhaustive match to select its
five existing literal/RegExp algorithms. No broad builtin ID, projected
Boolean, wildcard, default or impossible split arm remains.

## Durable evidence

`string_symbol_hook_operation_structure.rs` pins the exact domain, all seven
policy matches, semantic anchors, the private exhaustive fallback, five typed
standard producers, direct split dispatch and recursive source censuses. The
adjacent literal-replacement structure guard continues to pin the replace and
replace-all operation arms to their fixed semantic scope wrappers while the
raw scope remains private to its child owner.

## Verification

The structure target passes `4/4`, the adjacent literal-replacement guard
passes `3/3`, and the complete symbol-hook CLI fixture passes `1/1`. One exact
pinned leaf for each of `match`, `matchAll`, `replace`, `replaceAll`, `search`
and the direct `split` path passes both variants (`12/12`) with every failure
and unsupported bucket at zero. `cargo xc` is green. No semantic golden was run
because all operation choices occur while Rust emits the unchanged instruction
sequences.

## Deferrals

This invariant changes no hook lookup order, `IsRegExp`, RegExp global-flags
validation, fallback matching/replacement semantics, split semantics, dynamic
pattern compilation or broader String/RegExp conformance.

## Batch AY dispatcher boundary

The operation domain and raw emitter are now private to `string.rs`. Standard
dispatch reaches them only through five fixed String symbol-hook entries. The
frozen 306-line domain/emitter selection has SHA-256
`06636af9cd91f1e237e7cb08d47132941a9976c712a818073d1c208ce1271c26`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The symbol-hook, literal-replacement and RegExp
result-mode structure targets pass `5/5`, `3/3` and `3/3`; the complete
symbol-hook Wasm-AOT CLI fixture passes `1/1`. No Test262 leaf or Wasm golden
was required for this source-equivalent boundary, which claims no new String behavior,
conformance result or published-count change.
