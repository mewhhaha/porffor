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

The matcher and its child modules have exactly 56 result producers: one match,
three normal misses, 50 corrupt-program failures and two resource-exhaustion
failures. These are source call sites: 53 in `regexp.rs`, one in
`regexp/backreference.rs`, and two in `regexp/word_boundary.rs`. The
14 eager validation failures use parameter 3 as their preserved position; the
remaining exits preserve the position supplied by the matcher. The private
writer is the sole consumer.

The Rust-lexical structure guard ignores comments and all Rust string,
byte-string, C-string, raw-string, character and byte-character literals. It
pins the attribute-free domain, capability absence, exact producer census,
writer signature and complete projection.

## Nonclaims and verification

The original change was source-equivalent ABI hardening. It changed no emitted
status or found word, matcher program, backtracking order, scratch rewind, error
route, Realm or `lastIndex` behavior. Subsequent matcher features retain this
result boundary; refreshing the producer census itself changes no runtime code.

The original invariant-only batch's focused structure target passed `4/4`. Its
neighboring nullable-quantifier matcher-frame target passed `5/5`, and its CLI
witness passed `1/1`; no Test262, Wasm golden or broad workspace suite was run
for that batch. After matcher changes, refresh this source census and rerun
`cargo test -p lila-aot-wasm --test regexp_matcher_result_domain_structure`.
The census is separate from native behavior and conformance verification.
