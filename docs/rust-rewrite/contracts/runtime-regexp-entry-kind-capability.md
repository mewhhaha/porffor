# Runtime RegExp entry-kind capability

Status: implemented as a source-equivalent Wasm-AOT emitter invariant.

## Boundary

`RuntimeRegExpEntryKind::{Program, Rejected, Unsupported}` is the private
authority for the three words stored in each runtime RegExp program-table row.
It derives no cloning, copying, debugging, equality or default capability. Its
borrowed exhaustive `word` projection preserves the existing wire values 0, 1
and 2, while its borrowed exhaustive `throws_syntax_error` policy keeps
`Rejected` as the sole throwing row. `Unsupported` remains a legal pattern that
falls through to the runtime matcher, not a syntax error.

The table writer uses the projection in all three exhaustive entry arms. The
reader uses it for its two `Program` comparisons and builds the throwing-word
list by borrowing `ALL`, filtering through `throws_syntax_error`, and mapping
through `word`. No copied enum value, raw enum cast, equality/default policy or
wildcard arm participates in that route.

This is Rust-time capability hardening. It changes no table word, emitted Wasm
instruction, comparison order, branch depth, local lifetime or error behavior.
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
structure target passes `3/3`, and these two CLI witnesses pass `2/2`.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test runtime_regexp_entry_kind_structure --quiet
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
