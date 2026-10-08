# Non-ASCII RegExp identity atoms

The Pattern atom decoder admits a non-ASCII identity escape in a BMP pattern.
It retains the original source and uses the same scalar acquisition owner as
an unescaped source character. A BMP scalar produces one ordinary instruction;
an astral scalar produces the existing `LegacyUtf16Pair` term. Quantifiers
therefore attach to the trailing code unit. A group around the pair owns both
units before repetition. Forward and reverse lowerers consume that same term.

In either Unicode mode, a non-ASCII identity escape returns `InvalidSyntax`
through `SyntaxRule::IdentityEscape`, with the original backslash offset. The
private ASCII escape decoder has one caller after this complete acquisition
decision. Named-capture `k`, control `c`, class identities and ordinary Unicode
characters retain their existing grammar owners.

The references are the ECMA-262 [Annex B Pattern grammar](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-regular-expressions-patterns),
[IdentityEscape](https://tc39.es/ecma262/multipage/text-processing.html#prod-IdentityEscape)
and [Pattern Semantics](https://tc39.es/ecma262/multipage/text-processing.html#sec-pattern-semantics),
read on 2026-10-06. Legacy source-character identities allow these characters;
Unicode identities allow syntax characters and solidus. BMP Pattern semantics
use individual UTF-16 units for non-BMP source characters.

Two IR controls cover independent instruction expectations, descriptor
admission, greedy/lazy and zero/exact trailing-unit repetitions, grouped whole
pair repetitions, and the Unicode rule/offset. One Engine cohort runs strict
and sloppy sources, each pairing real literals with character-loop constructor
patterns against independent UTF-16 match/capture ranges. It includes case
folding, partial surrogates, reverse matching and catchable Unicode syntax.

The emitted constructor compiler, immutable descriptor, counted bounds and
matcher are unchanged. Oversized nullable minima remain explicit source debt.
This source batch is authored and isolated-formatted; compilation, runtime
controls and pinned RegExp acceptance have not run.
