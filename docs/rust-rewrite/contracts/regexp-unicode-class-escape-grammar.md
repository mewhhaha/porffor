# Unicode ordinary-class escape grammar

Status: normative for the Rust RegExp parser's ordinary character-class seam.

## Semantic boundary

An ordinary character class has the same ECMAScript grammar regardless of the
instruction representation selected for its members. Lila may encode an
ASCII-only class as two bitmap words and a wider class as code-point ranges,
but that optimization cannot change whether the Pattern is valid or the value
of an accepted escape.

The `u`-mode `ClassEscape` and `CharacterEscape` productions admit the common
control escapes, `\c` followed by an ASCII letter, `\0` only when the next
source character is not a decimal digit, hexadecimal and Unicode escapes,
character-class escapes, and the restricted Unicode identity escapes. Annex B
adds `\c` followed by a decimal digit or underscore and legacy octal escapes
only when Unicode mode is absent. Legacy identity escapes remain legacy
grammar; an ASCII bitmap must not make them legal under `u`.

An incomplete legacy `\c` has a different atom boundary from either control
escape. Annex B's standalone-backslash `ClassAtomNoDash` production consumes
only the backslash, after which `c` is parsed as the next class member. Thus
`[\c]` contains both `\` and `c`; it does not contain U+0003 and does not
collapse the two source characters into one identity escape. The same boundary
applies in the bitmap and range paths.

Consequently `[\q]` and `[\q\u0041]` have the same `u`-mode verdict even
though the latter spelling forces the range representation. The same invariant
applies to `\c`, `\c0`, `\c_`, `\1`, `\8`, and `\01`. Conversely, `\cA`
denotes U+0001 through either representation; it is not the two literal class
members `c` and `A` merely because the class fits in an ASCII bitmap.

This contract covers the ordinary legacy and `u` grammars only. UnicodeSets
classes have their own `ClassSetCharacter` and class-expression protocol.

## Closed parser protocol

`OrdinaryClassMode` has exactly `Legacy` and `Unicode`. It is selected by the
exhaustive top-level `RegExpUnicodeMode` dispatch and is required by the
ordinary class parser, the ASCII bitmap parser, and its atom parser. The range
path can reach the existing shared class-atom decoder only through an
exhaustive conversion of that closed ordinary mode. There is no Boolean mode
and no `UnicodeSets` value that an ordinary parser can silently reinterpret.

Both representations enforce the same escape decisions:

- `\c` followed by an ASCII letter is a control character in either mode;
- `\c` followed by a decimal digit or underscore is accepted only in Legacy;
- `\0` is NUL in Unicode only when no decimal digit follows;
- `\1` through `\7` are legacy octal only, while `\8` and `\9` are legacy
  identity escapes only; and
- the remaining identity fallback is unrestricted in Legacy but uses the
  closed Unicode class-identity predicate in Unicode.

The enumerated rejected control, octal, and identity forms return
`InvalidSyntax` carrying `SyntaxRule::ClassEscape`. Malformed hexadecimal,
Unicode, and property escapes retain their more specific syntax-rule
classifications; this seam does not collapse all invalid class escapes into one
rule. The bitmap and range parsing paths remain separate, but neither ordinary
representation is selected or decoded without the grammar mode that determines
these decisions.

## Durable witnesses

The IR representation-invariance regression asserts that each paired spelling
actually reaches a different parser before comparing its result. Negative
Unicode pairs cover `\q`, bare `\c`, `\c0`, `\1`, `\8`, and `\01`, with the
range sibling forced by `\u0041`. A prefixed `\c_` pair pins both the Unicode
`ClassEscape` verdict and its exact source offset, while legacy controls keep
the Annex B value U+001F live. Exact bare-`\c` values preserve both `\` and `c`
through both representations.

Value witnesses require `\cA` to match U+0001 through both encoders, preserve
`\0` under `u`, and preserve the legal Unicode class identity escapes. A
focused Wasm fixture covers both statically written patterns and patterns
selected from the finite runtime candidate table; legal siblings prevent a
blanket rejection from satisfying the fixture.

## Nonclaims and deferred gates

This seam does not implement arbitrary runtime pattern compilation, complete
UnicodeSets semantics, multi-digit decimal backreferences, new matcher opcodes,
resource limits, UTF-16 cursor changes, the RegExp object protocol, or String
integration. It removes no Test262 shortcut, changes no published conformance
count, and does not close the dynamic-loop Test262 witnesses or T19.

Static freeze gates are `rustfmt --check` for the touched Rust files,
`node --check` for the fixture, focused source scans proving every ordinary
class parser receives `OrdinaryClassMode`, `git diff --check`, and manual
exhaustive-match review. Cargo, fixture execution, focused pinned RegExp cases,
and the broad batch ladder remain deferred until the frozen patch is
independently reviewed.
