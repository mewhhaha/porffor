# Runtime RegExp named-capture inventory

The emitted `RegExpCompiler` compiles computed pattern strings to the same
immutable descriptor format as the Rust static compiler. Named captures and
named backreferences in Legacy and admitted u/v grammar use one canonical name
inventory. Computed code-point properties and complete v finite-string/class
algebra use the current parser and descriptor consumers. Shared ordinary
lookbehind uses its existing reverse matcher. This contract establishes capture
and name ownership; it does not claim full RegExp conformance.

The parser decodes direct Unicode scalars and strict fixed/braced Unicode name
escapes to canonical UTF-8. RegExpIdentifierName start and continuation checks
project the static parser's pinned ICU property authority, including ECMAScript's
extra identifier characters. Fixed surrogate escapes must be paired; braced
surrogate escapes are invalid. Names are nonempty and position-valid.

Each named capture row retains its numbered capture ID and owning AST group.
After the complete pattern is parsed, equal names must diverge through different
Sequence arms of a common group. Every equal-name pair is checked; a capture
outside the shared alternative is not admitted. Unique names receive indices in
first-occurrence order. Named backreferences, including forward references, are
resolved only against this completed inventory; unknown names are SyntaxError.
Without named captures, the existing Legacy identity escape for `\k` remains.
Every Unicode pattern instead enables named-reference grammar through the borrowed
validated character-mode owner, so a reference with no matching declaration is
SyntaxError. Both the atom selector and IdentityEscape exclusion consume that same
mode/census projection. Capture names are compared exactly; input folding never
changes name identity.

`CompletedCaptureInventory` has a private constructor and validates named operands
against its completed group count. The parser retains it inside
CompletedRegExpPattern after complete parsing and name validation. Actual lowering
requires that complete pattern; publication consumes it and then consumes the
inventory to produce the canonical named-group table:
32-byte header, 24-byte unique-name records, grouped numbered-capture candidate
IDs, and contiguous canonical UTF-8 names. Name pointers in the descriptor are
relative to the named table, so final heap compaction preserves their meaning.
The existing matcher selects the participating candidate, and existing exec
result construction publishes null-prototype `groups` and `indices.groups`,
including undefined nonparticipating captures and shared indices-array identity.
Those consumers use the existing checked descriptor protocol unchanged.

The static Legacy OrdinaryClassMode also carries the complete census through
both ASCII and code-point class encoders and both range endpoints. Constructing
legacy class mode without selecting its named-capture context is no longer a
legal emitter/parser call. The existing static representation-verdict regression
includes this grammar boundary.

The complete census is carried by every Pattern/Class character context and
both class range endpoints. Under the Annex B named-capture grammar, escaped
`k` is not an identity escape in a class; it is rejected even when the named
group occurs later. Either Unicode mode excludes that identity meaning regardless
of declarations. Without named captures, the Legacy class identity meaning remains.
The grammar authority is [ECMA-262 Annex B.1.2](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-regular-expressions-patterns).

`CompilerWorkspace` privately owns the layout cursor. Every variable or fixed
region reservation rounds that cursor to eight bytes, including the four-byte
name storage stride. Its consumed finish includes named-output capacity and
padding in the addressability check before growth. The temporary descriptor
therefore satisfies checked program-layout alignment before final compaction,
regardless of source byte-length parity; subsequent heap publication also
retains alignment. Bare unrounded cursor arithmetic is not part of the real
region-reservation API.

All parser data and serialization capacity are included in the compiler's
checked workspace before memory growth. Name-byte writes are bounded; no name
or row pointer escapes publication. Failure rolls back the compiler-owned heap
tail before returning its existing syntax/resource/corrupt status.
There are no JavaScript callbacks or nested allocations in the compiler region.

These source ownership claims do not establish runtime acceptance. Rust source
checks do not verify emitted Wasm. Wasm validation, focused engine/CLI regressions
and pinned conformance verification remain pending in the combined batch. No
runtime or conformance result is claimed here. Required focused checks
include the computed-name fixture in `aot_regexp_runtime_gap`, the CLI computed
named fixture and independent lookbehind control, existing static named and
numbered backreference/folding suites, and RegExp recompile/result suites.

The [named Unicode reference source extension](runtime-regexp-named-unicode-reference.md)
retains this canonical inventory and descriptor unchanged. Its finite source
controls cover code-point comparison, Unicode folding, reverse direction, name
identity, syntax/transaction boundaries and retained Legacy grammar. They remain
unexecuted at this source checkpoint; Rust source checks do not establish their
semantic acceptance.

The [code-point property completion](runtime-regexp-codepoint-property.md) retains
this same inventory. The later completed finite-string owner admits actual string
properties and v algebra without the retired pending capability node. Complete
parsing and name validation still precede instruction expansion and publication.
There is no second capture/name representation.
