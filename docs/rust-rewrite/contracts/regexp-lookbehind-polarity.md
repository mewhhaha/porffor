# RegExp lookaround polarity and direction

Both lookahead and lookbehind use `LookaroundPolarity::{Positive, Negative}`.
The private closed domain is produced by `from_syntax_marker`, retained by
`ParsedAtom::Lookaround`, and borrowed by the shared program lowerer.
`operand_bit` is its exhaustive wire projection. Matching direction is a
separate closed `RegExpMatchDirection::{Forward, Reverse}` domain.

The start instruction selects the assertion's direction. The end and failure
instructions carry the enclosing direction, so nested assertions resume their
caller without overwriting a shared success flag. End operand 1 stores polarity
in bit 63, enclosing direction in bit 62, and continuation in the remaining bits.
Failure operand 1 stores polarity in bit 0 and enclosing direction in bit 1.

A private choice sentinel snapshots the input cursor and captures. Successful
positive assertions retain captures but restore the cursor and discard their
private alternatives. Negative assertions restore captures as well. Exhausting
a negative body continues at its original cursor; matching that body rejects
the assertion. No outer continuation can backtrack into a successful assertion.
These are the [CompileAssertion semantics](https://tc39.es/ecma262/multipage/text-processing.html#sec-compileassertion).

The parser uses the full Disjunction grammar for lookaheads, including empty
bodies, escapes, character classes, scoped modifiers, captures, and nesting.
The old single-character shortcut and its two opcodes are removed. Legacy
quantified lookahead retains Annex B's zero-width repetition behavior; Unicode
lookahead and all lookbehind assertions reject a postfix quantifier.
Reverse scalar, range-set, and legacy surrogate-pair atoms now reach their
existing reverse matcher implementations. Reverse backreferences and whitespace
atoms remain separate matcher gaps; this change does not claim complete RegExp conformance.

Verification commands:

```sh
cargo test -p lila-ir --test regexp_lookbehind_polarity_structure --test regexp_lookbehind_polarity --test regexp_lookaround
cargo test -p lila-engine --test aot_regexp_lookaround --test aot_regexp_lookbehind_anchors
```
