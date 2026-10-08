# Computed RegExp Unicode character domain

This source batch corrects currently admitted computed u/v character semantics;
it does not implement full computed Unicode or UnicodeSets grammar. The follow-on complete code-point property extension shares this authority;
The subsequent [computed set-algebra owner](runtime-regexp-computed-unicode-sets.md)
adds nested code-point and complete finite-string algebra, strict q/string
syntax and operand-local iv folding through the same validated domain. The matcher, opcode and validated descriptor
representation are unchanged.

The emitted flag-validation factory owns invalid/duplicate/mutually exclusive
flags and ordinary prescan syntax before minting the private,
non-Copy CompilerCharacterMode. This owner retains the actual closed Legacy/u/v
mode local and selected existing fold-table/count locals. Workspace reservation,
bitmap clearing/copying/member/set/scanning, ordinary character decoding, classes,
boundaries and mode-sensitive syntax admission borrow that owner. Publication
finishes before its three locals are consumed in reverse order; the
complete compiler local set is released afterwards. No caller chooses an independent
bitmap domain or folding-table pair.

Legacy bitmap cardinality is 0x10000; either Unicode mode uses 0x110000, including
surrogate code points and U+10FFFF. Every bitmap member/set validates against that
same domain before addressing. The scanner includes one non-reading sentinel to
flush the final range. Complements cover the active domain; all output still uses
canonical sorted disjoint range slices and the existing honest range resource cap.

Raw SourceCharacter lead/trail pairs and two adjacent fixed Unicode escapes
combine only in u/v. A raw half next to a fixed escaped half remains separate.
Lone surrogates remain legal characters; range endpoints use their decoded scalar
identity. In Legacy, direct pair quantifiers retain their trail-unit behavior.

Class and literal closure uses the selected existing Unicode or Legacy mappings.
WordCharacters derives its Unicode+i closure from that same mapping authority,
then W complements the closed word set. Boundary and nonboundary instructions
publish that same scoped-i range slice. Property P operands use a separate bitmap: u complements before closure and v
closes before complement, preserving each operand before containing-class union. Both fold tables are retained for runtime compiler and
backreference use even without any statically known case-insensitive reference.

The ordinary v decoder rejects raw ()[]{}/-| syntax characters (backslash begins
an escape) and the reserved raw pairs && !! ## $$ %% ** ++ ,, .. :: ;; << == >>
?? @@ ^^ `` ~~. The later computed set-algebra dispatcher owns nested classes
and &&/-- operations; this character decoder supplies their actual strict atoms. Escaped syntax
characters and the exact reserved-punctuator alphabet & - ! # % , : ; < = > @
` ~ are admitted along with Unicode CharacterEscape and backspace. Escaped
underscore is excluded. A pair with an escaped first or second character does not
form a reserved raw pair. Initial caret remains the negation marker.

The normative authority is [ECMAScript 2026 Patterns and RegExp matching](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-patterns),
particularly 22.2.1.1, 22.2.1.7, 22.2.2 and 22.2.2.9.3–6. The specification uses
code-point input for either Unicode flag, simple/common case folding for Unicode+i,
a closed WordCharacters set, mode-dependent escape grammar and capture-count
validation. Existing static parser authority supplies the same distinctions.

Two finite computed semantic sources use real constructor/recompile/exec paths
and code-unit arrays so static candidate rows cannot stand in for runtime parsing.
They cover full-domain complements, case and word closure, scoped modifiers,
raw/fixed/cross-form pairs, astral and lone-surrogate classes, maximum code point,
admitted shared Unicode lookbehind, reverse matching and capture indices, reference
table rooting, Legacy controls,
missing/forward references, lookahead quantification, range endpoint rejection,
receiver rollback and complete ordinary v raw/escaped punctuation distinctions.
They are authored without execution. Existing gap and compiled-literal controls
remain in the same Engine target; no new mirrored structural test is added.

Rustfmt, source inventories, hashes and ordinary forward/inverse patch checks are
source/transport checks. No compilation, tests, runtime, Wasm validation, oracle,
stress/reproducer, generator or status refresh ran. Focused compiler and Engine
regressions, existing static folding/class/boundary/backreference suites, capability
controls and broad verification remain mandatory at the combined checkpoint.

The follow-on [braced Unicode escape contract](runtime-regexp-braced-unicode-escape.md)
admits complete braced code-point atoms through this same mode owner and existing
character representation. It preserves the fixed/raw/braced grammar distinction,
uses the existing braced GroupSpecifier name validation. Code-point property
support and its follow-on finite-string extension are described in
[its follow-on contract](runtime-regexp-codepoint-property.md). That source extension is also awaiting compilation
and execution. The [named Unicode reference extension](runtime-regexp-named-unicode-reference.md)
now projects this same validated mode and complete capture census for both actual
named grammar consumers, retaining the existing Unicode comparison/folding path.
