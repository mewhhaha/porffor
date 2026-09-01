# RegExp matcher result domain

Status: normative for the Wasm AOT ordered-bytecode matcher result writer.

## Boundary

The matcher helper returns four Wasm values: a found word, the match start,
the match end and a `RegExpMatcherStatus` ABI word. The first and fourth words
are not independent. A successful match must return `(1, Complete)`, a normal
miss must return `(0, Complete)`, and every matcher failure must return
`(0, Failed(reason))`.

`RegExpMatcherResult::{Match, NoMatch, Failed(RegExpMatcherFailure)}` is the
private authority for those three legal combinations. It derives no cloning,
copying, debugging, equality or default capability. All matcher exits pass one
owned result to `emit_regexp_match_result`; no exit can pass a raw found word or
an independently selected status.

The writer consumes the result in one exhaustive match:

- `Match` emits found word one and `Complete`;
- `NoMatch` emits found word zero and `Complete`; and
- `Failed(reason)` emits found word zero and preserves the typed failure in
  `RegExpMatcherStatus::Failed`.

There is no catch-all or unreachable arm. Adding another result state therefore
requires an explicit ABI projection before the crate compiles.

## Producer census

The current matcher has exactly 50 result producers: one match, three normal
misses, 44 corrupt-program failures and two resource-exhaustion failures. The
14 eager validation failures use parameter 3 as their preserved position; the
remaining exits use the candidate/match locals exactly as before. The private
writer is the sole consumer.

The Rust-lexical structure guard ignores comments and all Rust string,
byte-string, C-string, raw-string, character and byte-character literals. It
pins the attribute-free domain, capability absence, exact producer census,
writer signature and complete projection.

## Nonclaims and verification

This is source-equivalent ABI hardening. It changes no emitted status or found
word, matcher program, backtracking order, scratch rewind, error route, Realm or
`lastIndex` behavior. It adds no RegExp grammar, dynamic compilation or
conformance claim.

The focused structure target passes `4/4`. The neighboring nullable-quantifier
matcher-frame structure target passes `5/5`, and its nullable-quantifier CLI
witness passes `1/1`. No Test262, Wasm golden or broad workspace suite was run
for this invariant-only batch.
