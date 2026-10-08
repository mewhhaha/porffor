# Computed RegExp Unicode named references

The emitted Pattern compiler now admits named references in its currently supported
u/v grammar. The private validated CompilerCharacterMode owner emits the actual
NamedCaptureGroups selection: either Unicode mode enables it unconditionally;
Legacy enables it when the complete capture census found a named group. Both the
atom parser and Pattern/Class IdentityEscape exclusion borrow that projection.
A Unicode reference without any declared captures therefore reaches named parsing
and the existing UnknownGroupName syntax return, rather than becoming literal k.

The Unicode prescan retains ordinary opener, closer, source-bound and named
angle-delimiter checks. The later class owner retires the ClassString taint and
lexically hands complete v class bodies to the strict grammar parser. Code-point properties now use a delimiter
scan without taint and the complete mode-owned property decoder. Actual name
syntax and resolution still belong to the canonical parser/inventory. The
subsequent [computed set owner](runtime-regexp-computed-unicode-sets.md) now
retains complete q/property keys through class and capture/name validation,
then lowers finite strings through existing matcher instructions. Unknown names
still reject before instruction publication. Nested code-point and finite-key
algebra share the same completed-pattern boundary.

The unchanged name decoder canonicalizes direct, fixed-pair and braced identifier
spellings to UTF-8. Start/continuation membership uses the existing pinned authority;
empty, malformed, surrogate and invalid identifier spellings reject. Name equality
is exact and independent of input ignoreCase. The completed inventory validates
participating duplicate paths and resolves every reference, including forward
references, before CompletedRegExpPattern permits lowering. Publication consumes that
owner and its retained inventory, then writes the existing relative named table; no unresolved name pointer
can be published.

Named and numbered references share the existing directional matcher. It selects
the single participating capture (or matches empty), compares full code points
in either Unicode mode, and applies the retained Unicode fold table only when
the reference site's scoped i operand requests it. Capture indices and lastIndex
remain UTF-16 offsets. Existing mode-driven input dispatch and both retained fold
tables are unchanged. No matcher, data service, descriptor, opcode, ABI or alternate
capture representation is added.

[ECMAScript 2026 ParsePattern](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-parsepattern)
selects NamedCaptureGroups for both Unicode flags.
[Pattern early errors and matching](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-patterns)
require a matching group specifier and use the ordinary directional backreference
operation over that capture's input. The Legacy census distinction comes from
[Annex B ParsePattern](https://tc39.es/ecma262/2026/multipage/additional-ecmascript-features-for-web-browsers.html#sec-regular-expressions-patterns).

Two finite Engine sources build patterns from actual UTF-16 unit arrays and composed
grammar fragments, then invoke normal constructors, compile, exec and test. Controls
cover supplementary quantified references and indices, unmatched/forward references,
disjoint duplicate candidates, Unicode/scoped folding, reverse lookbehind, canonical
name spellings, exact name identity, sticky/global state, retained programs, unknown
names without captures, malformed/invalid/class syntax and transactional recompile.
Legacy identity/class/folding controls remain. Mixed named/code-point-property patterns now match normally. q, nested q
class and q-intersection controls now require normal matching. Existing
sloppy/strict fixture dispatch is reused, with no mirrored structural test.

Two obsolete Legacy success patterns move from the raw failure table to its existing
success descriptor-roundtrip table. The failure-only zero-handle/heap rollback
assertions and all remaining syntax/capability/resource rows stay exact.

All changes are authored source only. Rustfmt, exact preimages/hashes and ordinary
forward/inverse patch checks do not prove typechecking, Wasm validity or semantics.
No compiler, tests, runtime, validator, oracle, stress/reproducer, generator or status
refresh ran. Independent source review precedes integration; focused raw/Engine,
static name/backreference/folding and capability controls plus combined verification
remain mandatory at the later checkpoint.

The subsequent [code-point property batch](runtime-regexp-codepoint-property.md)
admits complete property aliases and ranges through this same parser. The
finite-string extension adds q and all seven string properties, with nested
algebra and operand-local folding. These source extensions remain unverified.
