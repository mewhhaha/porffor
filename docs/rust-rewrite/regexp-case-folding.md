# RegExp case folding and reverse backreferences

Character-set compilation distinguishes case-sensitive, legacy UTF-16 and
Unicode simple-folding modes. Scoped modifier groups select that mode before
literal or class instructions are emitted. Legacy Canonicalize uppercases one
code unit, rejects multi-character mappings, and prevents a non-ASCII input
from becoming ASCII. Unicode mode uses the C/S simple mappings, excluding full
and Turkic mappings. These rules follow
[ECMA-262 Canonicalize](https://tc39.es/ecma262/multipage/text-processing.html#sec-runtime-semantics-canonicalize-ch).

The existing vendored `regress` Unicode table exposes only its pure simple-fold
lookup to this compiler path. This does not add a runtime regex evaluator or a
new dependency. Its 1,512 non-identity mappings were compared exactly with
[Unicode 17.0 CaseFolding.txt](https://www.unicode.org/Public/17.0.0/ucd/CaseFolding.txt),
whose SHA-256 is
`ff8d8fefbf123574205085d6714c36149eb946d717a0c585c27f0f4ef58c4183`.
The comparison evidence is retained under
`target/failure-review/completed-baseline-20260914/unicode-dependency-review`.
A compile-time Unicode-version guard protects the Rust uppercase data used by
legacy matching. This change does not upgrade the separate ICU property data.

Reverse named and numbered references share the forward comparison operation.
The matcher traverses capture and candidate characters in the active direction,
then commits the candidate cursor after a complete comparison. A mismatch or input
underflow follows ordinary backtracking without changing the authoritative
cursor. Undefined captures match empty; Unicode matching rejects candidate
starts inside a surrogate pair while legacy matching retains code-unit
boundaries. Decimal references consume the entire decimal escape before the
parser decides whether legacy octal grammar applies.

The focused regressions are `regexp_case_folding`, `aot_regexp_case_folding`
and `aot_regexp_backreference`. The next checkpoint adds
[case-insensitive backreference comparison](../../crates/lila-aot-wasm/docs/regexp-backreference-folding.md)
and [word-boundary assertions](../../crates/lila-aot-wasm/docs/regexp-word-boundary.md).
UnicodeSets string folding remains unfinished.
Focused native verification and the pinned-suite replay are required before
reporting repaired execution counts.
