# Runtime RegExp compiler

The AOT runtime compiles admitted computed Pattern strings directly in emitted
Wasm. This is a RegExp-only compiler feeding the existing ordered matcher; it
cannot parse or execute JavaScript source, and calls no host compiler or matcher.

## Grammar and outcomes

The runtime grammar composes sequence, ordered alternatives, numbered captures,
noncapturing groups, ordinary classes and ranges, dot and boundaries, greedy or
lazy finite/unbounded repetitions, numbered/forward/unmatched references and
positive/negative lookahead and lookbehind, named captures/references in Legacy
and admitted u/v grammar, and scoped i/m/s modifiers. The admitted shared u/v route also uses the existing
reverse matcher for ordinary lookbehind; this does not widen its Unicode grammar gates. Flags d/g/i/m/s/y and admitted shared u/v grammar
retain their validated mode. In Legacy, UTF-16 units include lone surrogates and
an ungrouped astral pair's quantifier applies only to its trail. In u/v, raw pairs
and fixed-escape pairs denote one code point; raw/escaped cross-form halves remain
separate grammar productions.
Nullable optional repetitions use the existing progress-choice/check protocol,
including capture clearing and rollback. Parsed numbered references permit an
unmatched capture to consume no input; a nonempty capture body does not prove
participation. The active character domain selects the same retained Legacy or
Unicode canonicalization table as static compilation and backreferences.

Admitted shared u/v grammar uses full-code-point bitmap classes, complements and
case closure. WordCharacters closes before nonword complement, including scoped
i, and boundaries publish that closed range set. Missing-capture decimal escapes
and quantified lookahead are SyntaxError in either Unicode mode; their Annex B
Legacy behavior remains. The ordinary v class decoder checks raw syntax and
reserved-double punctuation separately from its exact escaped alphabet.
Braced Unicode code-point escapes use the same validated mode and complete atom
decoder, including null, maximum and lone-surrogate values. Existing GroupSpecifier
name decoding validates the exposed braced spellings before capture publication.
Named-reference grammar is enabled by the validated Unicode mode or the complete
Legacy named-capture census. Canonical direct/fixed/braced names resolve against
the completed capture inventory before lowering and publication; an unknown name
rejects even when a Unicode pattern declares no captures. All code-point property families and exact aliases use a complete immutable image
from the validated native property authority. Operand-local P folding distinguishes
u from v before class union and outer negation. Complete positive v string
properties and q expressions now become finite atoms after exact set algebra,
operand-local iv folding and static MayContainStrings checks. The complete
property catalog retains all seven provider payloads, including singletons.
Nested classes and set operators use the strict class grammar and checked
workspace; direct string-property escapes use the same completed finite atom.
u strings, P strings, true-MCS negation and Set range endpoints retain SyntaxError.
All later ordinary syntax and capture/name validation precede instruction
expansion and descriptor publication. Checked lowering tries longest strings,
singleton characters and empty members with existing matcher instructions.
Reverse code-point order and nullable progress use the same matcher.
Each prepared finite-string position preserves an unchanged code-point literal,
uses the existing ASCII bitmap instruction when its changed fold class is wholly
ASCII, or publishes sorted, coalesced ranges for non-ASCII preimages. This follows
the static modifier owner without charging literal or bitmap positions against
the range budget. Kelvin sign and long s therefore retain their non-ASCII
preimages; adjacent sigma preimages become one range.
The old pending capability node, its compiler-local failure variant and sole
helper Unsupported status/decoder arm are retired. Static candidate misses still
invoke the runtime compiler; the separate static-literal rejection remains live.
This is not whole-RegExp or full-suite conformance.

The private helper ABI takes two non-null GC Strings: already-coerced source and
flags. Its four results are a nullable GC RegExpProgram, i32 closed status,
i64 UTF-16 source offset and i64 typed detail. Status words are Compiled0,
SyntaxError1, ResourceExhausted3 and CorruptProgram4; word 2 is retired. Only Compiled
carries a non-null program. The wrapper routes syntax errors to SyntaxError,
resource failures to RangeError, and internal invalid programs to Error, using
the current function's Realm. A missing statically compiled literal program
still uses the separate mandatory T19 semantic rejection (code 7), distinct from
JavaScript errors and no-match. Literal allocation
requires `&RegExpProgram`; a missing static literal program takes the same rejection
when evaluated. Exec has one compiled-program matcher and rejects a missing
program before layout validation. Its old zero-program simple matcher and final
pattern fallback are deleted. `@@match` also uses the ordinary exec protocol
for every object receiver, with no original-source dispatch override. Its
cached exec dispatch calls callable values once and validates Object/null, or
checks the intrinsic RegExp brand after a noncallable Get result. The String.match
RegExpCreate path always invokes the created receiver's observed `@@match`;
its former raw-pattern matcher is deleted.
See [the capability contract](regexp-runtime-capability-rejection.md).

## Ownership and bounds

The pure helper owns one checked region above its entry heap checkpoint. It
validates flags before minting the private non-Copy CompilerCharacterMode owner.
That owner selects actual bitmap domain and folding locals, is borrowed by workspace
and parser consumers, then is consumed in reverse local order after publication.
The parser decodes WTF-8 into UTF-16 source units, counts capture syntax before
decimal escapes, and builds a flat tree whose parents precede children. Parent
links form the parser group stack; lowering uses bounded three-word task records.
No source-dependent Wasm recursion or JS allocation occurs inside the compiler.

Each group owns its inherited modifier state in the arena. The parser restores
that state from the group owner before reading each term or alternative; captures,
noncapturing groups and lookaheads preserve the same lexical boundary. Scoped i
controls literal/class case closure and reference operands; scoped m/s use the
existing Inherit/ForceOn/ForceOff anchor/dot operands. No matcher flag stack or
public flag mutation is involved. Prefixes admit only i/m/s, at most one dash and
no duplicate or overlapping flags. An empty remove list after added flags is
valid; an entirely empty modifier list is SyntaxError. Both parsers share the
closed modifier alphabet and masks. Changing global i/m/s flags still recompiles
a clone because atoms outside local scopes inherit those global flags.

A postorder width pass rejects expansion before iteration. Full decimal spans
are compared before narrowing, so reversed huge bounds are SyntaxError. Finite
oversized bounds remain distinct from unbounded repetitions; state-free atoms
with zero emitted width can discard repetition without iterating the count.
Programs retain the existing 32768-instruction/65536-range limits. Instruction
operands, successors, progress and non-consuming cycles use shared closed opcode
facts. Exact repeatable-split counts are derived with bounded graph traversal.

Workspace address overflow and failed memory.grow return typed resource outcomes;
the generic trapping HeapAlloc helper is not used. Every failure clears the
owned region before restoring the entry checkpoint and allowing the wrapper to
allocate an Error. A failure before workspace allocation clears an empty range.
Successful instructions and ranges are serialized into the existing RGPB relative
descriptor, copied into an owned immutable GC byte array, then checked through
the GC RegExpProgram. The compiler clears and releases all private workspace
above the entry checkpoint. The published GC descriptor and all pre-checkpoint
allocations remain intact.
This zero-memory contract matters because ordinary allocation leaves some record
slots at their initial zero value; rewinding dirty parser storage could make a
fresh capture array appear non-extensible. No published descriptor points into
parser, task, source, folding or scratch storage. Later matcher scratch checkpoints occur
after publication; clones can share the GC program with independent lastIndex.

A failed compile preserves the receiver's old program/source/flags/lastIndex.
Successful installation precedes the final strict lastIndex Set, which may throw
after the new program is installed. Constructor source/flags/program snapshots
remain coherent across coercion callbacks. Constructor/split/matchAll share an
existing program only when compilation flags agree; changing only g/d/y retains
supported static Unicode/named programs without requiring runtime recompilation.

## Verification boundary

`runtime_regexp_compiler_tests` discovers helper indices from the physical runtime
R's name section and calls its existing exports directly. Host imports trap if
called. Returned owned GC bytes
must roundtrip through ValidatedRegExpProgram and match static descriptors for
selected common grammar. Separate controls cover resource rollback including
physical memory-growth refusal, zeroed released memory and allocator reuse,
descriptor persistence, computed pattern matches,
syntax/throw identity, recompile state and changed clone flags. Computed matching
fixtures use code-unit arrays and composed grammar fragments; literal regression
controls compile in a separate artifact so candidate rows cannot mask the runtime
parser. Scoped-modifier controls separately cover lexical restoration, captures,
lookahead, nullable repetitions, class closure, reference-site folding, rejected
prefixes and changed-global-flag clones. Descriptor comparison includes nested
m/s overrides and restored sensitive/reference operands. The original immutable
descriptor lifetime fixture remains unchanged. Additional exact comparisons
cover finite-string literals, ASCII bitmaps, non-ASCII fold preimages, normalized
sigma ranges, lone surrogates, scoped i restoration and reverse string lowering.
All five emitted-compiler functions pass in the 2026-10-09 cloud checkpoint,
including these nine additional comparisons and the unchanged original parity,
resource, empty-composition and allocator-reuse controls. Broad verification is
recorded separately in `CONTINUE.md`.

Computed capture-array controls also allocate and mutate fresh arrays and objects
after successful, syntax-failing and resource-failing compilation; retained
programs must continue matching after later compiler invocations.

These new checks are staged for the next verification checkpoint. No full-suite
status is inferred from them. Named-capture and shared lookbehind source implementations and
these admitted Unicode-route corrections remain unexecuted in the current batch;
complete computed finite-string algebra is also written but unverified. See
[the computed Unicode character contract](runtime-regexp-unicode-character-domain.md).

The narrow braced escape source extension is described in
[its contract](runtime-regexp-braced-unicode-escape.md); its semantic sources and
existing braced GroupSpecifier controls remain unexecuted in this source batch.

The [named Unicode reference contract](runtime-regexp-named-unicode-reference.md)
uses the existing completed name inventory and directional code-point matcher.
Finite normal/syntax controls remain unexecuted. Two obsolete Legacy
named/lookbehind raw failure rows now belong to the existing success descriptor
roundtrip table; all failure-only publication and rollback checks are preserved.

The [computed code-point property contract](runtime-regexp-codepoint-property.md)
describes the complete pinned alias/range image and consumed parser completion.
Lowering borrows CompletedRegExpPattern, and publication consumes it; the capture
inventory alone cannot bypass complete pattern validation. Its finite
source controls and updated old gate/raw expectations remain unexecuted.
The [computed set contract](runtime-regexp-computed-unicode-sets.md) and
[finite catalog contract](runtime-regexp-finite-string-catalog.md) describe
complete q/property payloads, exact finite algebra and instruction publication.
Existing raw descriptor controls now admit q strings, while failure-only
zero-handle/rollback checks retain genuine syntax and resource cases.
