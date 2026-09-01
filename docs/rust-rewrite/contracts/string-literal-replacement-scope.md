# String literal replacement scope

Status: implemented for the literal fallback shared by
`String.prototype.replace` and `String.prototype.replaceAll`.

## Boundary

The literal replacement loop has one private
`builtins/string/string_literal_replacement_scope.rs` owner. Its non-copyable
`StringLiteralReplacementScope::{FirstOccurrence, AllOccurrences}` domain and
raw scope-parameterized emitter are child-private. The outer fallback can
request only the first-occurrence or all-occurrences semantic operation; it
cannot name, construct, import or project the raw scope, and it no longer
passes the broader `StandardBuiltinId` into the replacement loop.

After a successful substitution, one borrowed exhaustive match owns the only
scope difference. `FirstOccurrence` emits the existing `Br(2)` exit.
`AllOccurrences` emits the existing scan-index update and empty-search advance
before the shared loop continuation. No Boolean, equality, default or wildcard
selection remains, and the emitted instruction order within both paths is
unchanged.

## Durable evidence

`string_literal_replacement_scope_structure.rs` recursively pins the private
module and parent exclusion, the exact two variants and lack of convenience
capabilities, each semantic wrapper's sole parent caller, the typed private
helper, the single exhaustive projection, both instruction sequences and the
private definition-plus-two-wrapper-call census.

The moved four-line domain and 440-line raw emitter retain SHA-256
`db2e26fd031d6c5ab6f0ce99ab16f58928a202f29e9bee436069cb9368b882ba`
and `f6a34782a74376adfe9f7b622241e986c8c2bdace5f2fac4967ff4f07cf5170e`;
their combined 444-line semantic selection retains
`0d0392432a791efc6c208fd83b41ab0061fecb60c05b93417928c355a68f2d15`.
Only the two narrow semantic wrappers are new. The resulting 489-line child
has SHA-256
`a85f4aba490a7792db382749383f8e1a9cc17195a902d4821d3f6225e11a1a4f`.

## Verification

The bounded structure target passes all `3/3` tests. The adjacent symbol-hook
CLI fixture passes `1/1`, and the exact first-only `replace` and all-occurrences
`replaceAll` Test262 leaves pass both variants (`4/4`) with every failure bucket
at zero. Workspace formatting and the diff check are green. No broad Test262
run, semantic golden or status refresh belongs to this source-equivalent batch.

Batch AJ moved the complete raw owner source-equivalently and narrowed the two
parent calls without changing their argument order or emitted behavior. Shared
`cargo xc` passes; this structure target and the adjacent symbol-hook and RegExp
flag-getter targets pass `10/10`, the exact symbol-hook CLI fixture passes
`1/1`, and the two pinned replacement leaves pass all `4/4` sloppy/strict
Wasm-AOT executions with every failure bucket at zero. No semantic golden was
needed or run.

## Deferrals

This contract does not change replacement-string substitution, functional
replacement calls, RegExp replacement, Unicode indexing, or complete String,
RegExp, T18 or T19.
