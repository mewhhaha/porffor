# Span-stable replacement failure

The module-syntax rewriter admits replacement text only when it can preserve
the erased source span's byte width and ordered ECMAScript line-terminator
sequences. Failure inside that private admission boundary has exactly three
meanings: an invalid source span, generated text containing a line terminator,
or replacement content that does not fit.

## Closed domain

`SpanStableReplacementError` is a private, non-derived three-row domain. Two
source slicing failures construct `InvalidSpan`; the generated-padding check
constructs `GeneratedLineTerminator`; and the three checked-width failures
construct `DoesNotFit`. The domain has no clone, copy, debug, equality or
default capability.

`rewrite_default_keywords` is the sole semantic consumer. Its exhaustive match
preserves the three existing `StripError` projections and their order:
`DoesNotFit`, `InvalidSpan`, then `GeneratedLineTerminator`. Adding a failure row
therefore requires the compiler-visible producer and consumer boundary to be
updated together.

## Durable regressions

The recursive structure guard fixes the attribute-free private declaration,
the 13 owner/source mentions, all six producer conditions in their original
order, the exact exhaustive error projections, and the existing focused unit
that rejects a generated line terminator. Run:

```sh
cargo test -p lila-ir --test span_stable_replacement_error_structure
cargo test -p lila-ir modules::source::tests::a_generated_replacement_cannot_add_a_line_terminator -- --exact
cargo fmt --all -- --check
```

The structure target passes `3/3`, and the exact owner witness passes `1/1`.
Independent review confirmed the private capability boundary, recursive census,
all six ordered producers and exact three-row diagnostic mapping. The
coordinated workspace checkpoint passes `cargo fmt --all -- --check`,
`cargo xc`, `git diff --check`, the module boundary check and the task-plan
check; the compile retains the repository's existing warnings.

## Nonclaims

This source-equivalent closure changes no module grammar, edit ordering,
replacement bytes, error text or emitted Wasm. It does not broaden module
loading or linking support, and it adds no public error surface.
