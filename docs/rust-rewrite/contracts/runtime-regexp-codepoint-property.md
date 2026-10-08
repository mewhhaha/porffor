# Computed RegExp code-point properties

Computed p/P now consumes all normative code-point property families and exact
aliases from the same validated native constructor as static RegExp compilation.
The complete binary row macro retains all 53 variants and aliases. Strict ICU
GeneralCategoryGroup and Script tries enumerate all GC values, lone GC aliases
and Script/Script_Extensions values; only the six normative family spellings
select named families. The committed four Unicode 17 Script additions supply
their exact eight aliases. No partial property list or loose ICU lookup enters
the emitted parser. One OnceLock catalog validates each name and normalized
range set once. Seven provider string names share an exhaustive closed enum.

Every StringPool serializes that complete immutable catalog with shared raw
range payloads, checked 32-bit addresses and five private row words. The image
is unconditional and independent of source candidates. A native image owner
and mode-owned emitted operand factory retain the actual data and character
domain; callers cannot construct properties from an arbitrary pointer or choose
a separate Unicode domain. The locked zerotrie 0.2.2 alloc feature enables the
actual strict-key iterator. Cargo.lock resolution is pending; no dependency
version/provenance is hand-edited.

The opaque character-set operand distinguishes single characters, built-in
escapes, code-point properties and positive v string properties. Classes keep
those identities through both endpoint decoders: only two single characters
form a range, with the existing Legacy Annex B built-in fallback preserved.
A property operand has its own full-code-point bitmap. In u, P complements raw
membership before containing-class case closure. In v, the operand closes using
the current scoped i before complement and union. Class negation complements
the completed union through the existing range opcode. UTF-16 source decoding,
surrogate code points, U+10FFFF, folded backreferences and matcher representation
remain the existing authorities. Workspace reservation includes the third
bitmap; the existing range/instruction bounds and rollback/publication rules
remain enforced. No image pointer escapes into a published descriptor.

The subsequent finite-string owner replaces the pending string-capability AST
node with a completed finite atom. All seven positive v string properties retain
their complete validated singleton and multi-code-point payloads; the property
image's closed kind selects ranges or sequence descriptors. Native parsing and
the emitted parser consume the same immutable catalog. See the
[finite catalog contract](runtime-regexp-finite-string-catalog.md).

The complete ordinary parser still checks delimiters, quantifiers, groups,
property aliases and static MayContainStrings. Existing capture completion
resolves and validates every name before minting CompletedRegExpPattern. The
lowerer borrows that private owner and publication consumes it; an unresolved
capture inventory alone cannot permit publication. Finite-key instruction
expansion follows this complete validation boundary. Range and workspace budgets
are checked during private preparation, with the existing typed rollback.

u string properties, P string properties, true-MayContainStrings negation and
Set range endpoints reject SyntaxError. A recognized v string followed by an
unknown property, malformed grammar or unknown named reference also rejects
SyntaxError. Valid positive v strings now lower to the existing matcher. The
[computed set-algebra owner](runtime-regexp-computed-unicode-sets.md) handles
complete q/property keys, code-point and finite-key algebra, operand-local iv
folding, nested complements and longest/singleton/empty priority. Legacy p/P
identity and brace/quantifier grammar remain unchanged.

The normative authority is [ECMA-262 2026 Patterns](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-patterns),
especially 22.2.1.1 and 22.2.2.9.7–8. P and class case/complement ordering use
the same static matching authority and pinned Unicode data.

Finite code-unit-composed Engine sources observe complete family/alias domains,
case-sensitive spelling, Unicode 17 membership, full-domain and surrogate
membership, scoped folding, u/v P versus class negation, captures/direction,
receiver rollback and string syntax/publication ordering. Existing ASCII-p raw
status cells move to success/descriptor and allocator controls; later q/v
classification cells become positive matching controls with finite-string support. These semantic sources are authored, not executed.
No compiler, Cargo, runtime, tests, validator, JS parser, oracle, stress/resource
probe, generator or status refresh ran. Formatting, hashes and ordinary patch
roundtrips check source transport only. Compilation, focused native/compiler and
Engine checks, finite-string/algebra controls and broad verification remain
mandatory before runtime or conformance acceptance.
