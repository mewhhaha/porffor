# Runtime RegExp entry-kind capability

Status: the entry-kind authority is implemented; the typed T19 rejection follow-up
and two additional structure witnesses are staged for integration verification.

## Boundary

`RuntimeRegExpEntryKind::{Program, Rejected, Unsupported}` is the private
authority for the three words stored in each runtime RegExp program-table row.
It derives no cloning, copying, debugging, equality or default capability. Its
borrowed exhaustive `word` projection preserves the existing wire values 0, 1
and 2, while its borrowed exhaustive `throws_syntax_error` policy keeps
`Rejected` as the sole syntax-throwing row. `Unsupported` enters the emitted
pattern compiler. If that compiler also reports Unsupported, the product reports
the typed T19 semantic gap outside JavaScript completion.

The table writer uses the projection in all three exhaustive entry arms. The
reader uses it for its two `Program` comparisons and builds the throwing-word
list by borrowing `ALL`, filtering through `throws_syntax_error`, and mapping
through `word`. No copied enum value, raw enum cast, equality/default policy or
wildcard arm participates in that route.

The original entry-kind hardening changed no table word, emitted Wasm
instruction, comparison order, branch depth, local lifetime or error behavior.
The follow-up preserves those wire words and changes the Unsupported compiler
outcome to a mandatory semantic rejection before publication.
`ALL` remains a handwritten list whose two exhaustive projections force a new
variant to choose both a wire word and a throwing policy before the crate can
build.

## Durable evidence

`runtime_regexp_entry_kind_structure.rs` lexically excludes comments and every
Rust string/character literal form from its recursive census. It pins the exact
attribute-free declaration and policies, ten source mentions, five direct word
calls and the one UFCS mapper, the sole throw-policy call and `ALL.iter` route,
the exact 0/1/2 constant declarations and authority-only constant census, all
three writer arms with no later raw kind overwrite, both `Program` comparison
instruction sequences, and the complete borrowed throwing pipeline through its
equality/OR aggregation, SyntaxError emission and reverse local release tail.

The existing valid and invalid runtime-pattern CLI fixtures exercise the
program-row and rejected-row paths. They are focused witnesses, not arbitrary
runtime-pattern compilation or complete RegExp/Test262 conformance. The
original structure checkpoint passed `3/3`, and its two CLI witnesses passed
`2/2`. Those historical results do not verify the staged rejection policy or the
new literal/allocation proof witness.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test runtime_link --quiet -- runtime_regexp_entry_kind_structure::
cargo test -p lila-cli --test cli regexp::run_wasm_backend_succeeds_for_regexp_runtime_pattern_valid_fixture -- --exact --test-threads=1
cargo test -p lila-cli --test cli regexp::run_wasm_backend_succeeds_for_regexp_runtime_pattern_invalid_fixture -- --exact --test-threads=1
cargo fmt --all -- --check
git diff --check
```

No broad RegExp suite, Test262 cohort, semantic golden or README status refresh
is claimed by this lane.
Independent dry re-review is clean after the exact constant authority, complete
reader tail and no-overwrite writer tail were pinned. The following shared
workspace checkpoint passes `cargo fmt --all -- --check`, `cargo xc`, the
recursive module-boundary check, the task-plan check and `git diff --check`.
