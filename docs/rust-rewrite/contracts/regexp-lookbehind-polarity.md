# RegExp lookbehind polarity authority

A parsed lookbehind has one polarity from syntax through matcher-program
lowering. `LookbehindPolarity::{Positive, Negative}` is the private closed
domain for that choice. It derives no cloning, copying, comparison, debugging
or default capability.

`LookbehindPolarity::from_syntax_marker` is the only syntax producer. It maps
`=` and `!` to the two legal variants and rejects every other byte. The
`ParsedAtom::Lookbehind` row owns that value; the program lowerer borrows the
same authority for both provisional instructions and their final patched
forms. It never projects the polarity back to a Boolean.

`LookbehindPolarity::operand_bit` is the sole wire projection. Its exhaustive
match preserves the existing matcher ABI: positive is zero and negative is
one. `RegExpInstruction::lookbehind_end` places that bit at operand 1 bit 63,
while `lookbehind_failure` writes it as operand 1. Adding a polarity therefore
requires a compile-visible decision at the one ABI boundary.

The focused IR witness compiles positive and negative patterns through
`RegExpProgram::compile` and observes distinct end and failure bits. The source
guard fixes the closed domain, sole syntax producer, typed ParsedAtom field,
four borrowed lowering uses and sole exhaustive wire projection:

```sh
cargo test -p lila-ir --test regexp_lookbehind_polarity_structure
cargo test -p lila-ir --test regexp_lookbehind_polarity positive_and_negative_lookbehind_preserve_distinct_matcher_polarity -- --exact
```

This is source-equivalent matcher-program hardening. It adds no lookbehind
grammar, reverse-matcher instruction, runtime pattern compilation or broader
RegExp conformance claim.
