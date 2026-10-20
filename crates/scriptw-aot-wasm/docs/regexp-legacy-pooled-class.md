# Legacy RegExp pooled character classes

Pooled classes such as `[\S]`, `[^é]`, surrogate ranges, and non-ASCII literal
classes use the same RegExp range-pool instruction in legacy and Unicode modes.
The pool's representation does not determine the character width: legacy matching
classifies one UTF-16 unit, while `u` and `v` classify a full Unicode character.

The forward matcher projects the decoded scalar to the selected surrogate unit
for legacy membership checks. It retains the original scalar for the shared
character-advance logic, which can leave the byte cursor between a pair or finish
the pair. Pooled classes can therefore match at a trailing-unit cursor, including
sticky `lastIndex` and backtracking positions. Membership failure leaves the
cursor unchanged until the existing choice rollback path runs. Reverse matching
already selects the preceding unit or scalar before checking the range pool.

The ordinary-class parser also treats a raw astral source character as two grammar
atoms in legacy mode. For `[a-😀]`, the leading surrogate ends the range and the
trailing surrogate is the next atom. For `[😀-\uFFFF]`, the leading surrogate is
an independent member and the trailing surrogate starts the range. Retaining that
next atom preserves reversed-range early errors; expanding an astral endpoint
into an unordered union would not. The existing private `LegacyUtf16Pair` domain
owns this conversion for both ordinary terms and classes. Unicode parsing still
uses scalar endpoints. Original pattern source and capture text are preserved.

Range normalization, complement flags, and case-fold closure stay unchanged.
Legacy classes continue to use legacy Canonicalize closure; Unicode classes retain
simple-fold closure and the existing `u`/`v` complement ordering. The matcher ABI,
range-pool format, and admission of runtime patterns do not change.

The frozen batch4 reproducer checked `[\S]` and `[^é]` with one and two atoms on
`😀`. It printed `true, false, true, false`; the required result is
`false, true, false, true`. Focused verification comprises the
`regexp_legacy_pooled_class` IR target, the updated `legacy_utf16_pair_structure`
target, `aot_regexp_legacy_pooled_class`, and the existing forward-whitespace,
backreference and case-folding regressions. Compilation and native results must
be recorded after integration; staging is not a passing-test claim.
