# Legacy direct-astral RegExp quantifiers

Status: normative for the Rust RegExp parser's direct-source term seam.

## Semantic boundary

ECMA-262 parses a RegExp pattern as UTF-16 code units even when Lila receives
the static source as a Rust UTF-8 string. In legacy mode, a directly written
astral scalar therefore contributes two adjacent `PatternCharacter` atoms: its
lead surrogate and its trail surrogate. A postfix quantifier immediately after
that source applies only to the trail-surrogate atom.

For example, legacy `/𠮷?/` has the same atom boundary as
`/\uD842\uDFB7?/`; it does not mean `/(?:\uD842\uDFB7)?/`. The lead surrogate
is always emitted once, while `?`, `*`, `+`, or a braced quantifier controls
only the trail surrogate. In `u` or `v` mode the same direct source is one
code-point atom, so its postfix quantifier continues to apply to the whole
scalar.

This follows ECMA-262 `RegExpInitialize` (22.2.3.3): without `u` or `v`, each
16-bit pattern element becomes a BMP code point without UTF-16 decoding before
`ParsePattern` applies the `Pattern[~UnicodeMode, ~UnicodeSetsMode]` grammar in
22.2.1. It is not a matcher optimization.

## Closed parsed-term protocol

`LegacyUtf16Pair` can be constructed only from a validated astral Unicode
scalar and privately owns its lead and trail surrogate code units.
`ParsedTerm` then has two exhaustive cases:

- `Quantified` owns one ordinary `ParsedAtom` and its postfix quantifier; and
- `LegacyUtf16Pair` owns the validated pair and the quantifier that applies to
  its trail only.

The forward program lowerer must emit the pair's lead once and then lower the
trail through the ordinary quantifier path. Reverse lowering must lower the
quantified trail before the lead. Nullability is false regardless of the trail
quantifier because the lead remains mandatory. A new parsed-term consumer must
choose both cases before Rust will compile it; there is no generic instruction
sequence that can accidentally quantify the pair as one atom.

## Durable witnesses

The IR regression fixes exact instruction shapes for an unquantified direct
astral scalar and its optional, lazy-optional, zero-count, and repeated-trail
legacy forms. The Wasm CLI fixture fixes observable matching for:

- a full surrogate pair and a lone lead under `?`;
- zero and multiple trail repetitions;
- greedy versus lazy trail choice with a following literal; and
- the contrasting whole-code-point quantifier boundary in `u` mode.

## Nonclaims and deferred gates

This seam does not complete escaped-surrogate interpretation, supplementary
ignore-case folding, astral terms inside the currently restricted lookbehind
subset, arbitrary runtime pattern compilation, the full RegExp grammar, or the
RegExp object protocol. It changes no matcher opcode, bytecode encoding,
resource limit, or published conformance count.

Static freeze gates are `rustfmt --check` for the two touched Rust files,
`node --check` for the fixture, focused source searches, `git diff --check`, and
manual exhaustive-match review. Cargo, fixture execution, focused pinned
Test262 RegExp trees, and the broad batch ladder remain deferred until the
frozen patch is independently reviewed.
