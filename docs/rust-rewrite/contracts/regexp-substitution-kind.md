# RegExp substitution kind

Status: implemented for the shared RegExp `GetSubstitution` emitter.

## Boundary

The private `builtins/string/regexp_substitution.rs` child recognizes exactly
six substitutions through its non-copyable `RegExpSubstitutionKind` domain:
literal dollar, matched substring, prefix, suffix, numbered capture and named
capture. `ALL` fixes that semantic order. One borrowed exhaustive
`runtime_code` projection owns the stable internal codes 1 through 6. The
String parent retains only the semantic `emit_regexp_get_substitution` call; it
cannot name a kind, encode a runtime code or invoke a partial raw consumer.

The four two-byte spellings and the numbered/named recognizers store only named
variants' runtime codes. The handler walks `ALL`, compares the same projection
and matches the enum exhaustively to emit each existing semantic instruction
sequence. Raw zero remains only the explicit no-recognized-substitution
sentinel. Consumed widths, replacement-index updates and literal-start updates
remain unchanged.

## Durable evidence

`regexp_substitution_kind_structure.rs` pins the exact six variants, `ALL`
order and runtime-code mapping; every recognition write; the exhaustive handler
order and semantic anchors; the zero sentinel; all consumed/index updates; and
a recursive source census of the closed authority and its projections.

The exact 30-line domain/authority and 448-line GetSubstitution algorithm
retain visibility-normalized SHA-256
`0f852520992bfe2689f1ba08c1351c8accc5921373cbb32c2ac1f493b56ab453`
and
`d11dd555a3b82a43496296de74b04367c50ffa0fc2b148f8c2b1eb2453ee0d8d`;
their combined 478-line selection retains SHA-256
`c8deaa00580f7d7a74e684273325a4e7b496c3aa39f69d66a7b7da8cfb02f2dd`.
The resulting 20,970-line parent and 483-line child have SHA-256
`62caf68bd5a9bc02354c8fdc31b1d73d467a374d68a82717561adcf810a2dd3f`
and
`5163b1c56b48ee90a6f3ee5ea6f5c19ad013ea1462090c526e3e951bee43a473`.
The unchanged 14-line parent call retains SHA-256
`fcecb3ddcc9b61f06b276734b76b5c04211dffb75d962f0f01c0e2f43a862b8a`.
The recursive guard and module policy pin zero parent raw-policy names, all 15
domain mentions and four runtime-code projections in the child, and exactly
one parent semantic call.

## Verification

The bounded structure target passes all `4/4` tests. Six exact Test262 leaves,
one for each substitution kind, pass both variants (`12/12`) with every failure
bucket at zero. Workspace formatting and the diff check are green. No new
fixture, broad Test262 run, semantic golden or status refresh belongs to this
source-equivalent batch. At the Batch AB checkpoint, `cargo xc` is green, the
owner target passes `4/4`, the neighboring flag-getter and literal-replacement
targets pass `3/3` each, and the same six leaves pass all `12/12` Wasm-AOT
variants with every failure bucket at zero. No CLI fixture or emitted-Wasm
golden was run for the owner move.

## Deferrals

This contract does not change substitution grammar, capture lookup, coercion,
Unicode indexing, RegExp matching, or complete String, RegExp, T18 or T19.
