# T19 — Complete ECMAScript RegExp semantics

## Folding inventory and native checkpoint — 2026-10-08

Unicode folding enumerates the pinned regress mapping ranges directly; legacy
folding scans only its UTF-16 domain. An exhaustive comparison with the previous
full-scalar algorithm proves identical mapping values and ordering. All seven
native backreference-folding controls pass. RegExp publication has a private
expression-family owner and retains compiled-program admission. These focused
results do not establish full pinned RegExp or performance acceptance; see the
[current checkpoint](README.md#closure-audit-and-verification--2026-10-08).

## Current source position — 2026-10-07

The reviewed static/runtime grammar owners and class-character domain repairs
are authored. The bounded audit below found no remaining reachable grammar
capability rejection; explicit resource limits and corruption checks remain.
This does not close task-wide native semantics, full pinned RegExp/Test262 or
resource/performance acceptance. Earlier unsupported-grammar notes retain their
historical scope.

## Current compiler capability audit — 2026-10-07 dry source

A bounded audit of the current static producer and emitted compiler found no
remaining reachable grammar-capability rejection in those owners. This is a
source finding, not a conformance result. The runtime parser owns ordinary and
Unicode classes, Unicode-set algebra and finite strings, named and numbered
references, forward/reverse assertions, scoped modifiers and exact decimal
quantifiers. The older dynamic-grammar and runtime-Unsupported checkpoints below
describe predecessors.

The closed runtime outcomes are `Compiled`, `SyntaxError`, `ResourceExhausted`
and `CorruptProgram` in
[`runtime_helpers.rs`](../crates/lila-aot-wasm/src/runtime_helpers.rs).
[`compiler/contracts.rs`](../crates/lila-aot-wasm/src/builtins/regexp/compiler/contracts.rs)
admits only `CompileFailure::{Syntax, Resource, Corrupt}`; its resource reasons
are address space, memory growth, nodes, tasks, instructions and ranges.
[`program.rs`](../crates/lila-aot-wasm/src/builtins/regexp/program.rs) validates
descriptor extents and ownership; its corruption paths do not reject language
features. The retained source limits are 32,768 instructions and 65,536 pooled
ranges in [`regexp.rs`](../crates/lila-ir/src/regexp.rs). The emitted workspace
uses source-sized capacities (`units = source bytes + 1`, `nodes = 4 * units + 1`,
`tasks = 4 * nodes + 8 * instruction limit + 32`) and checks its aligned end
against the Wasm32 address domain before growth or writes. These remain resource
bounds, not missing grammar implementations.

Static `UnsupportedFeature` still preserves runtime compilation through
[`lowering/static_literals.rs`](../crates/lila-ir/src/lowering/static_literals.rs).
The audit classified its remaining sites as actual range/instruction/capture
limits or parser-precondition guards: the lookbehind guard accepts every
currently constructible atom, group/alternative parsing consumes the remaining
syntax-character fallthrough, and non-ASCII classes select the full range owner.
Those guards were not expanded into a separate cleanup batch.

The audit did find and fix two static class representation defects: `\s` includes
non-ASCII whitespace, and Annex B octal escapes `\200` through `\377` exceed the
128-bit ASCII bitmap. Both now select the existing full range parser before
bitmap construction. Two static controls and the paired literal/computed
[`class_character_domains.js`](../crates/lila-engine/tests/fixtures/regexp_runtime_gap/class_character_domains.js)
fixture cover whitespace, complements, reverse matching, octal endpoints,
nonmembers and preserved syntax errors. Only narrow formatting and diff checks
were run; compilation, focused execution, resource/performance acceptance and
the full pinned RegExp/Test262 gates remain pending for the combined batch.

## Current counted source closure — 2026-10-06 dry source

The current matcher consumes linked choices, recursive required-run templates,
live progress playback and actual failed-group exhaustion traces. The producers
also share the pure-empty proof. A separate proof at the actual counted Begin
now admits complete iteration-independent bodies: the entry Clear resets every
written capture, nested counters and progress attempts have fresh source-owned
lifecycles, and assertions restore the original cursor and direction. Ordered
continuation results stabilize at the actual UTF-16 input length plus one, or
one for an all-zero-width body. Exact source-sized subtraction retains the
finite maximum-minus-minimum gap; unbounded maxima remain unbounded. This closes
successful empty alternatives that reach End and would recreate same-row runs,
without weakening Run observation or relying on cursor/template equality.

Two new literal/computed Engine controls contain 26 semantic cases covering
priority, last-iteration captures, assertions, references, reverse and astral
matching, nested cycles and finite optional gaps. The independent source
challenge is clear. Unproved reset/owner shapes retain the original matcher
path; descriptor semantics and resource ceilings are unchanged. The older
sections below record their narrower source epochs. The current combined source
is uncompiled, unformatted and unrun; full RegExp, performance and pinned-suite
acceptance remain pending. See the [independent iteration contract](../docs/rust-rewrite/contracts/regexp-independent-counted-repetition.md).

## Completed child and capture-assertion templates — 2026-10-06 dry source

The original required-run proof now scans complete nested counted pairs and
retains optional, zero-bound and choice-owning child continuations. Actual v3
pair admission requires a distinct inactive child row; the complete ordered
templates compare all its counters. Original optional Guard snapshots may supply
the fixed-point witness. Discharged capture-writing assertions and references
share the closed assertion scanner only under the full snapshot proof; the
required-empty replay remains input-only. Paired literal/computed controls cover
huge bounds, child fallback order, captures, reverse matching and finite exhaustion.

These changes are uncompiled and unrun. Live progress entries, nested Run
templates, nested assertions, nonuniform iterations and huge exhaustive fallback
traversal remain source work. No resource limit or failure outcome was relaxed.

## Linked choices and required runs — 2026-10-06 source

The original matcher now stores choices in one demand-grown linked arena with
complete capture/repetition snapshots. Its consumed required-run proof compares
two actual ordered templates before replacing mandatory iterations with exact
source-sized counters and immutable fallback templates. Restoring a virtual
choice preserves LIFO order and the original affine minimum/maximum state.
Shared exact-child admission covers completed positive exact nested lifecycles.

The original input-only assertion scanner is now shared by required-empty
replay and required-run body admission. A fully discharged assertion can appear
inside an outer surviving choice without retaining an assertion sentinel in the
compressed template. The original linked matcher still owns assertions,
progress, backtracking and full state restoration. Meaningful literal/computed
cohorts are authored and unrun. Live assertion/progress templates, optional or
choice-owning nested lifecycles, nonuniform iterations and huge exhaustive
fallback traversal remain open; resource ceilings are unchanged.
See the [run contract](../docs/rust-rewrite/contracts/regexp-required-choice-runs.md).

## Required empty replay source successor — 2026-10-06

The native matcher now consumes one paired Begin/End admission and its retained
counter workspace. After a genuine required iteration ends at the unchanged
UTF-16 cursor, it can prove capture effects, input-only assertions, untouched
canonical numbered/named references and completed positive exact nested counters
idempotent. The consuming proof updates minimum/maximum through the original
typed natural-counter operations; guard, backtracking and restoration owners
remain unchanged. The old capture-only child is replaced by this shared proof.

Independent source review is clear and actual producer/descriptor controls are
authored but unrun. A dry source successor now simulates each reference's ordered
prefix from the actual post-body fixed point, permitting balanced body-written
empty captures and checking named alias participation again. The real forward/
reverse capture rules and temporary open-capture nonparticipation are retained.
The same successor admits input-only lookarounds only through their exact closed
sentinel/End/Failure shape and bounded internal branches. Their original matcher
removes assertion choices and restores cursor/repeat state before returning.
No new controls or executions were run for this successor. General escaping choices,
nonempty capture-dependent references,
lookarounds, optional nested repeats and performance acceptance remain open.
Original source schemas and resource ceilings are unchanged.

## Exact natural bounds and native counters — 2026-10-06 source

Both actual producers retain canonical decimal minimum/finite maximum bounds at
and beyond u64 without numeric expansion or an Unbounded approximation. The v3
immutable descriptor carries checked bound rows and exact source-sized state
extents. Dense Begin slots, proper regions, canonical digit storage and successor
ownership are admitted before the matcher consumes the layout.

Native minimum/maximum counters have distinct sealed types and exact base-10^9
limbs. Choices/assertions restore the entire live slab. A scoped capture-only proof
admits mandatory empty acceleration after one real successful iteration and checks
CaptureEnd's actual replay precondition. General empty choices, assertions,
references and nested repeats remain outside that proof. Scratch ceilings are
unchanged. The transactional rejection control now exceeds genuine source-body
capacity; large numeric bounds are admitted.

Source peers and meaningful descriptor/native controls are written. No current
compilation or execution is claimed. See the [exact counter contract](../docs/rust-rewrite/contracts/regexp-exact-natural-counter.md).

## Non-ASCII identity atoms — 2026-10-06 source

The existing static atom decoder now shares scalar acquisition for ordinary and
legacy-escaped non-ASCII source characters. Its existing UTF-16 pair owner keeps
astral quantifier binding, grouped/reverse behavior and lone code units. Unicode
IdentityEscape retains its actual syntax-rule/backslash-offset rejection.
Two IR controls and a paired literal/computed Engine cohort are authored and
independently source-reviewed; all original RegExp controls are retained.
No compilation or execution is claimed. Exact oversized bounds now have the
source representation above; general execution-time acceleration remains open.

## Earlier counted compiler and demand-grown matcher — 2026-10-06 source

Static and emitted RegExp compilation now retain one body for admitted numeric
repetitions. The sole immutable program descriptor has reciprocal Begin/Guard/
End/Exit instructions and a dense repeat-slot count. Pair, nesting, active-region
and progress validation precede publication; certified End summaries still
expose cycles that reset counters by re-entering Begin.

The matcher saves capture and repetition state together in ordered choices.
Required empty successes decrement the exact minimum; optional empty attempts
backtrack before decrement. Scratch capacity grows on actual choice demand
under the unchanged 512 MiB ceiling, after immutable input materialization.
Positive/negative assertion completion restores the appropriate repeat state.
Tiny semantic controls compare literals and runtime character-loop constructors
for finite bounds, empty/captured alternatives, greedy/lazy order, reverse
matching and assertion restoration.

Braced syntax now orders the original decimal spans before backend admission;
an exact/overflow decimal enum prevents saturated upper bounds from being
re-sorted as mathematical counts. Existing controls cover ordered, equal and
reversed bounds around the u64 boundary. These controls remain unexecuted.

At this earlier source epoch, oversized nullable minima remained unsupported;
they do not expand instructions or become approximate counts. Compilation,
runtime regressions, broader RegExp coverage and pinned acceptance remain
unrun on this source successor. See the counted compiler and matcher workspace
contracts in `docs/rust-rewrite/contracts/`.

## Realm-owned legacy state — 2026-10-05 dry source

The constructor's legacy accessors now capture a closed slot and read actual
Realm-owned GC Strings instead of returning an unconditional empty value.
Successful builtin matches update all slots, including the final capture beyond
nine and exact UTF-16 contexts. Source literals and constructed objects retain
their original Realm and immutable legacy-enabled choice; successful subclass
matches invalidate their Realm's legacy state. Input ToString preserves receiver
validation order, reentrancy and abrupt identity. Foreign or disabled `compile`
receivers are rejected before mutation. Two paired Engine controls and the
[legacy-state contract](../docs/rust-rewrite/contracts/regexp-legacy-state.md) are
written for the complete batch. Compilation, execution and current-pin T19/T26
acceptance remain pending.


## String invocation results and effects — 2026-10-04 dry source

The complete String invocation-family successor keeps the actual acquired
callee, raw receiver and full real argument list. Both live result analysis and
the spread-aware signature retain arbitrary MatchAll hook returns, joining
Match, Replace, ReplaceAll, Search and Split. Generic String catalog effects
invalidate captured facts through original symbol GetMethod and Call. A const
rule requires the synchronous-user-code flag on every generic String row.

Meaningful IR and paired Engine controls retain all six hook roles, arbitrary
Number/Function/Symbol values, original getters/Proxy calls, spread, caller
mutations, borrowed Realm errors and exact abrupt cutoffs. This complete source
is authored after the passing ref105 checkpoint and remains type/runtime
unverified. Existing RegExp native algorithms, created-hook ownership and full
RegExpCreate/grammar/descriptor acceptance remain separate. No current pinned
result or full T19/T26 closure is claimed. See the
[invocation contract](../docs/rust-rewrite/contracts/string-invocation-family.md).

## Number borrowed String hooks — 2026-10-04 dry source

The successor removes Number primitive Match/Split gates and all copied native
method booleans. The actual acquired property, raw receiver and complete real
arguments use the existing indirect analysis/call owner; hook returns cannot
inherit a synthetic Array result. The static separator shortcut no longer
throws while discarding argument IR. Existing String symbol algorithms and
called-function Realm ownership remain independent.

Meaningful lowering and two paired Engine cohorts retain acquisition before
prototype replacement, spread/ignored operands, arbitrary hook returns and
caller mutation, original abrupt identity and both borrowed error Realms.
These controls and the coupled dead inline dispatch retirement remain
uncompiled and unexecuted. All remaining task source precedes capped
verification; full T19/T26 and pinned acceptance remain open. See the
[successor contract](../docs/rust-rewrite/contracts/number-string-hook-and-dispatch-retirement.md).

## String integration — 2026-10-04 dry source

A private consuming method owner joins original String hook GetMethod and the
created-RegExp required Invoke paths. An absent original hook enters fallback
once; the created receiver supplies its own observed hook, and its nullish or
non-callable result throws rather than selecting another literal iterator.
Match/search/matchAll share callable Proxy dispatch. The created receiver uses
the called String builtin's intrinsic RegExp prototype, with no mutable public
constructor/prototype lookup. Existing matcher/program initialization and
independently live iterator-from-start consumers retain their owners.

Finite existing Engine/CLI controls cover original/created getter identity,
Proxy/apply order, abrupt values, called Realm errors and prototype selection.
This complete String protocol batch passed the ref93 combined all-target Rust
type check and remains unexecuted. It does
not close all RegExpCreate initialization, pattern grammar, descriptor or
current-pin RegExp conformance work. See the
[contract](../docs/rust-rewrite/contracts/string-symbol-hook-operation.md).

## Current dry implementation — 2026-10-03

The emitted pattern compiler now has unverified source support for computed
named captures and named backreferences. Canonical names and duplicate-name
admission feed a completed capture inventory required by instruction lowering
and consumed by named-table publication. Forward references use that completed
inventory; existing matcher and groups/indices construction consume the table.
Workspace allocation preserves eight-byte descriptor alignment, including
padding in its capacity guard. Runtime and static class escapes retain the
complete named-capture context, including range endpoints.

Computed non-Unicode lookbehind now retains containing and child directions
through the existing parser workspace and reverse matcher. A closed child-order
choice keeps alternatives in source order and requires a direction for
sequences. Reverse capture boundaries and both nested assertion exits preserve
the containing state; quantified lookbehind remains a genuine SyntaxError.

The already accepted shared `u`/`v` route now retains one private validated
character-mode owner through workspace and parser consumers. It selects the
full code-point bitmap and existing Unicode fold table, closes scoped word
membership before nonword complement, and publishes those ranges for boundaries.
Raw/raw and fixed/fixed surrogate pairs retain their grammar-specific association;
decimal misses, quantified lookahead and ordinary `v` punctuation receive
mode-specific syntax checks. Existing shared lookbehind continues through the
reverse matcher. Both fold tables are rooted for computed backreferences without
requiring a static reference. Two finite Engine sources and one stale legacy
named-capture status-cell correction accompany the source change. They remain
unexecuted; see the [Unicode character contract](../docs/rust-rewrite/contracts/runtime-regexp-unicode-character-domain.md).

Computed braced Unicode escapes now consume a complete code-point atom through
that same mode owner. Only fixed escapes invoke fixed/fixed surrogate pairing;
braced null, maximum and lone-surrogate values remain separate characters.
Existing quantifier/class/range consumers and validated braced GroupSpecifier
names use the current representation. The sole braced pre-scan taint is
removed; the later code-point property owner closes that gate, while the later finite-string owner closes runtime string matching in unverified source.
Two finite semantic sources cover accepted values, cross-form separation,
syntax and recompile transaction. They remain uncompiled and unexecuted; see
the [braced escape contract](../docs/rust-rewrite/contracts/runtime-regexp-braced-unicode-escape.md).

Computed Unicode named references now reach the actual canonical name parser
and completed capture inventory. The validated character mode enables named
grammar for every Unicode pattern, including a pattern without declarations;
unknown names therefore produce a syntax error rather than Legacy identity text.
Existing directional comparisons, scoped Unicode folding, UTF-16 indices and
transactional publication remain shared. The ordinary prescan retains its named
escape delimiters; the later class owner consumes complete v bodies. The later code-point property owner closes its
property gate; the later class and finite-string owners close nested code-point
and finite-string algebra in unverified source. Two finite
matching and syntax sources and corrected raw success rows remain uncompiled
and unexecuted; see the [Unicode named reference contract](../docs/rust-rewrite/contracts/runtime-regexp-named-unicode-reference.md).

The native code-point property resolver now consumes one exact validated alias
owner. Its complete binary domain maps exhaustively to the existing pinned ICU
data, and the Unicode 17 delta accepts only validated values/new Script kinds.
Four ICU-only mixed-case names reject; strict GC/Script/scx aliases, Unicode 17
additions, string-property restrictions and Legacy grammar remain shared. Five
finite native semantic controls are authored and unexecuted; see the
[native alias contract](../docs/rust-rewrite/contracts/regexp-native-unicode-property-alias.md).

Computed code-point property escapes now consume a complete cached immutable
catalog from that validated native owner. Exact binary, GC/GC-family, Script
and Script_Extensions aliases derive from pinned provider data, including the
typed Unicode 17 additions. The module image deduplicates immutable range
payloads and checks every address before publishing its five-word rows.
The emitted parser consumes validated character-set operands through both
range-endpoint routes. Its separate property bitmap preserves `u` raw
complement followed by closure, `v` operand closure before complement/union,
and final negated-class closure before complement.

The complete property image now retains validated code-point keys for all seven
provider properties of strings, including singletons. Native static parsing and
immutable serialization consume that same catalog. String rows select checked
16-byte descriptors and u32 little-endian backing keys; code-point rows retain
their existing range bytes. No source-sized AST expansion is needed.

Computed `v` classes now consume a private closed grammar/frame owner for nested
union, homogeneous intersection and homogeneous subtraction. Character versus Set
operands enforce range endpoints; strict escapes and decoded `\q` cardinality
feed exact static MayContainStrings. Union uses OR, intersection uses AND and
subtraction retains the left static fact. Negation rejects a true fact even when
algebra eliminates every string. Actual finite keys and the separate empty key
participate in the same algebra as singleton bitmaps. Each operand is folded and
deduplicated before `/iv` algebra and nested complement.

The completed finite atom replaces the pending string-capability node. Its keys
are ordered longest first, followed by a singleton alternative and then empty;
checked lowering emits the existing literal/range and Split/Jump instructions.
Reverse code-point order, UTF-16 capture bounds and nullable quantifier progress
reuse the matcher. Full syntax and capture/name completion still precede checked
instruction expansion and descriptor publication. Checked compiler workspace
extensions retain rollback, zeroed released storage and receiver transactions.
Valid direct `\p` string properties and bracketed `\q`/property algebra share
this route; invalid aliases, `u` properties of strings, `\P` strings, true-MCS
negation and Set range endpoints retain SyntaxError.

Three paired finite-string Engine sources cover all seven complete properties,
algebra, longest-priority backtracking, empty progress, operand-local folding,
reverse bounds and recompile rollback. Existing string-gap controls become
positive admission/matching controls. Earlier code-point, named and property
controls remain. All new source is uncompiled and unexecuted. Normal Cargo lock
resolution for the direct pinned alias-iterator dependency remains part of the
deferred compile checkpoint; no lock file was hand-edited. See the
[computed property contract](../docs/rust-rewrite/contracts/runtime-regexp-codepoint-property.md),
[computed set contract](../docs/rust-rewrite/contracts/runtime-regexp-computed-unicode-sets.md)
and [finite catalog contract](../docs/rust-rewrite/contracts/runtime-regexp-finite-string-catalog.md).

All three emitted program-layout consumers now reserve a Pending owner without
decoded-section accessors. Only the consuming factory, which emits the existing
closed corrupt-program return, exposes a Validated owner. Raw fields, the
decoder and validity flag are private. The existing guards were already correct;
this source invariant prevents a future caller from omitting their handoff.
See the [program boundary](../docs/rust-rewrite/contracts/regexp-program-boundary.md).

The regression sources cover name spelling, forward references, alternative
duplicates, groups/indices identity, workspace parity and class grammar.
Computed lookbehind sources cover greediness, references, nested assertions,
anchors, nullable/huge bounds, sticky lastIndex and recompile publication.
Compilation and runtime verification are deferred until the coherent
implementation batch is ready. Full RegExp acceptance remains open; earlier results below apply to their original
source. See the
[named-capture contract](../docs/rust-rewrite/contracts/runtime-regexp-named-capture-inventory.md)
and [lookbehind contract](../docs/rust-rewrite/contracts/runtime-regexp-lookbehind-direction.md).

## Current candidate verification

Seven new operand-fold IR controls and nine paired direct Engine fixtures
pass. Ten selected adjacent RegExp files pass 20 modes, including both
circled-M property rows and the Unicode case-mapping neighbor. The 33-file/
66-mode direct class-string inventory is /v-only and was not replayed in
full by this selection. No direct /iv pins are selected. Runtime grammar
and full RegExp conformance remain open.

The candidate continuation revalidates 84 focused stages with 1,856 selected Rust test invocations on the exact same Source. Compilation,
one separate both-engine startup invocation and the default-features CLI
build belong to the original focused run. The continuation freezes that
CLI unchanged and executes all 151 selected pinned modes from 82 files.
The original pin-identity validation failure remains recorded.

This is candidate verification. MAIN installation and a fresh complete
MAIN broad checkpoint remain required. The earlier session 30300 is
INCOMPLETE without an owned terminal; its exit and cause remain unknown.
Full pinned Test262 conformance and task acceptance remain open. The
published status span is unchanged. The authentic continuation terminal is `bb22495c5187c67c269adeffd0e8efd91c3cbe97be18a79fb86c263945bc2798`; its Root-owned exit is `b399f9e5c16d0847ff4854a885e40c1b0b55f5428791d0186919db6dc64d0b1a`. The revalidated same-Source prefix is `7d0ce67ed3ce18e2646639461ba9eeb7f5d4e0793275b0937cf687ec96bf5bc9`; its original enclosing Root1 is `12a3deab22796d658bebdce50eaf263cf2a1443b1f03f0269dcdba951f9c77c1`.

The preparation and dated verification statements below retain their
original scope and failures. This checkpoint supersedes only the
unexecuted state of the named selected controls described above.


**Status:** In progress — reviewed static/runtime RegExp source and class-domain repairs are authored; task-wide native semantics, resource/performance acceptance and full pinned-suite verification remain incomplete.

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10, T18  
**Blocks:** String-RegExp integration and RegExp-related T26 closure

## Historical repository state — before current compiler closures

The current static compiler lowers all seven provider Unicode properties of
strings into finite sets. The historical keycap-only checkpoints below retain
their original scope; they do not describe the current provider projection.
Direct class-string `/iv` folding is now prepared in an isolated source-only
successor. It folds direct and finite-property operands before finite-set
algebra, normalizes singleton aliases through the established range representation
and matches each string position through existing scoped-fold instructions.
`Basic_Emoji` and `RGI_Emoji` include a casable circled-M sequence and require
this operand-local normalization. Seven new IR controls and nine paired Engine
fixtures are uncompiled and unexecuted. All 33 pinned
direct class-string files use `/v`; their 66 modes are adjacent coverage only.
Broader runtime grammar and full-tree gates remain open.

The runtime capability rejection follow-up routes emitted compiler Unsupported,
missing static literal programs and a defensive missing exec program through
`RuntimeSemanticGap::RegExpRuntimePatternCompilation` (T19, ABI 7). A compiler gap
cannot become a catchable TypeError or satisfy a runtime-negative expectation.
Compiled u/v programs, true SyntaxError, resource RangeError and failed compile
receiver preservation retain their separate paths. The staged Engine target defines
eight tests and 20 sloppy/strict observations; runner controls cover catching and
runtime-negative matching. Verification of this follow-up remains pending; no
conformance counts are changed. The contract and refresh commands are in
[`regexp-runtime-capability-rejection.md`](../docs/rust-rewrite/contracts/regexp-runtime-capability-rejection.md).

The same corrective batch removes 1,659 lines of unreachable zero-program exec
code: the simple matcher, final pattern fallback and handled-local handshake.
Exec retains its brand, input and single ActiveHandler ToLength ordering before
one real compiled matcher. The five compiled result-mode projections retain
their exact normalized bodies; the existing structure witnesses now bind two
consumers, five projections and a 16-mention ownership census. Genuine matcher
errors remain unchanged. Verification is pending integration with the capability
rejection and CLI controls. The subsequent ordinary `@@match` protocol retirement removes its compact
source catalogue and exact orphan closure. It preserves the prior generic
protocol body, including object acceptance, ToString, flags, custom exec,
Unicode advancement and lastIndex ordering. The three child owners and their
nine source-layout tests are deleted; semantic fixture coverage remains and now
includes UTF-16/sticky/flags/custom-exec/input-recompile controls. A subsequent
bounded repair passes the already-read exec value to one private abstract-operation
emitter: callable Function/Proxy values are called once and return Object/null;
noncallable values check the RegExp internal slots before using the current
compiled program. It fixes branded null/noncallable exec while retaining ordinary
custom-exec receivers and genuine non-RegExp TypeError. String.match always
invokes the created RegExp's observed `@@match`, fixing computed `a+` and `(a)+`;
its exact 301-line fallback closure is deleted. The expanded existing CLI fixture
checks getter throw/recompile order, proxy calls, primitive-result rejection and
created-method overrides. Node v24.12.0 passes both modes; Lila verification remains
pending integration. Separate String.matchAll, `@@search` and other String source
routes require their own audit.

The 2026-09-29 continuation repairs the emitted pattern compiler's unbounded
plus encoding. A shared must-advance predicate selects the same atom/loop-back
layout as static compilation, while nullable atoms retain their progress
handling. Descriptor round trips now cover greedy/lazy plus, nullable atoms,
backreferences and a program at the instruction cap. Resource probes derive
their sizes from `REGEXP_MAX_INSTRUCTIONS`; simple Unicode patterns are positive
controls, and unsupported property/set forms remain explicit capability
controls. All ten runtime-pattern compiler tests pass on 2026-09-30, including
the five failures found in the interrupted workspace run. Broad integration
verification remains pending.

Lila now has dedicated RegExp IR parsing and Wasm builtin support for a
growing syntax/behavior subset, plus String symbol-dispatch integration. The
selected engine, bytecode, Unicode, backtracking and deterministic resource
contracts are recorded in
[`docs/rust-rewrite/regexp-engine.md`](../docs/rust-rewrite/regexp-engine.md).
Arbitrary runtime pattern compilation, broad grammar coverage, the complete
typed resource policy and complete zero-timeout RegExp/String-regexp trees are
not yet present. The README explicitly records broader syntax combinations as
unsupported, and focused Test262 rewrites remain.

The `RegExp.prototype[Symbol.search]` descriptor, `length` and `name` metadata
cases now execute their unchanged sources and full `propertyHelper` harness for
6/6 sloppy/strict variants. Removing their stale rewrite authority retired four
T19-owned semantic shortcut observations. This records metadata conformance,
not broader RegExp matching or grammar closure.

The Unicode `String.prototype.matchAll` integration case now executes its
unchanged source and full `compareArray.js` harness for both sloppy and strict
Wasm-AOT variants. The adjacent null and undefined cases also execute unchanged
with the complete `compareArray.js`, `compareIterator.js` and `regExpUtils.js`
harness for all four sloppy/strict variants. The two RegExp match-indices cases
also execute unchanged with the complete `compareArray.js`, `propertyHelper.js`
and `deepEqual.js` harness for all four sloppy/strict variants. Removing their
compact harness leaves the generated inventory with zero T19 observations.
The shared workspace and repository policy gates pass after that retirement.
All 648 Wasm-golden artifacts remain present; the combined batch's RegExp dump
changes are emitted-size summaries from the shared unary-numeric lowering, not
a replacement rewrite or materializer path.

The exact generated UnicodeSets `\q{…}` class-string batch is implemented and
verified. At clean pre-batch commit `f580b424d`, the three exact representatives
`built-ins/RegExp/unicodeSets/generated/string-literal-union-string-literal.js`,
`built-ins/RegExp/unicodeSets/generated/string-literal-intersection-string-literal.js`
and
`built-ins/RegExp/unicodeSets/generated/string-literal-difference-string-literal.js`
each reported `0/2` sloppy/strict Wasm-AOT executions. All six measured
executions were `Runtime/NotImplemented` with `RegExp.prototype.exec unsupported
pattern`, with zero unsupported, crash or bug verdicts. None of the three has
an exact rewrite, materializer or known-failure entry. The source-coherent
27-file/54-execution inventory has nine generated combinations for each of
union, intersection and subtraction where at least one operand is a string
literal, excluding the six combinations that also depend on Unicode properties
of strings. The compiler now retains a canonical range-and-string set, applies
exact sequence algebra, normalizes one-code-point members into ranges, and
emits multi-code-point alternatives longest-first before the singleton class
and empty member in both directions. Central verification passed
workspace/all-target checking, `cargo xc`, focused IR `1/1`, bounded structure
`7/7`, the source-free Wasm lifecycle fixture `1/1`, and the exact unmasked
cohort `54/54`, with zero parser, early-error, lowering, runtime, Wasm-backend,
harness, unsupported, crash or bug outcomes. The fixture found one integration
gap after the IR implementation: reverse lookbehind rejected the existing
code-point-literal and Unicode-range instructions. The matcher now admits those
instructions in reverse and shares one canonical range-membership emitter
between forward and reverse paths. At that checkpoint, other Unicode properties
of strings and direct class-string `/iv` folding remained distinct capability
boundaries; this historical evidence records no broader UnicodeSets closure.

The adjacent Unicode-property-of-strings batch now gives Unicode 17
`Emoji_Keycap_Sequence` an exact finite representation: the twelve strings
`[#*0-9] FE0F 20E3`. Direct `\p{Emoji_Keycap_Sequence}` atoms and `v`-mode
union, intersection and subtraction all consume the existing canonical
`FiniteClassSet`; the direct `iv` form shares the same bytecode because every
member is simple-case-fold invariant. At that checkpoint, other properties of
strings retained typed unsupported capability. Negated classes that may contain
strings retain their required early error. At clean pre-batch commit `04e38f2ba`, exact direct-property file
`built-ins/RegExp/property-escapes/generated/strings/Emoji_Keycap_Sequence.js`
and generated algebra representative
`string-literal-union-property-of-strings-escape.js` each reported `0/2`
sloppy/strict Wasm-AOT executions, all `Runtime/NotImplemented`, with no exact
rewrite, materializer or known-failure mask. The source-derived inventory is
37 files/74 executions: 34 positive files/68 executions exercise the finite
property, while three negative syntax files/six executions must remain green.
Central verification passed workspace/all-target checking, focused IR `1/1`,
the retained UnicodeSets structure executable `7/7`, and the expanded Wasm
fixture `1/1` in `24.04s`. The exact raw inventory is `74/74`, with every
failure-kind and NotImplemented/Crash/Bug bucket at zero. This closes only the
finite keycap property. Basic_Emoji and the remaining RGI properties were still
open at that checkpoint; the current seven-property implementation supersedes
that boundary without extending this historical measurement.

The property-of-strings authority is now closed at the vendored provider
boundary without changing that behavior. The provider crate root narrowly
re-exports its strict parser, seven-variant `UnicodeStringProperty` and
read-only sequence accessor while keeping the generated table module private.
Lila parses once into that domain and projects every variant in an exhaustive,
catch-all-free match. At that checkpoint, only `EmojiKeycapSequence` consumed the
provider's exact twelve rows into `FiniteClassSet`, and the other six variants
were typed unsupported capabilities. The current projection consumes all seven. The duplicate handwritten keycap construction
is deleted. This invariant lane adds no RegExp syntax or conformance claim;
focused IR witnesses passed `1/1` and `1/1`, the dedicated provider-domain
structure target passed `3/3`, the retained finite-string structure target
passed `7/7`, and `cargo check -p lila-ir`, formatting and diff checks were
green. No Wasm fixture, golden or Test262 status was rerun; the detailed
boundary remains recorded in the finite-string-algebra contract.

The bounded matcher batch for RepeatMatcher's nullable unbounded quantifier
progress rule is now verified. At clean pre-batch commit `44247b836b`, the exact
unflagged `built-ins/RegExp/nullable-quantifier.js` witness reported `0/2`
sloppy/strict Wasm-AOT executions. Both were `Runtime/NotImplemented` with
`RegExp.prototype.exec unsupported pattern`; the file has no exact rewrite,
materializer or known-failure entry. The existing compiler rejects every
unbounded nullable atom instead of discarding only an optional iteration that
matched the empty string.

The durable CLI oracle covers the exact `(a?b??)*` result, suffix
backtracking after an empty-iteration rejection, greedy and lazy repeats,
required empty minima, a bounded-repeat control, captures, nested nullable
loops, reverse lookbehind compilation and overall/global empty-match progress.
The closed IR/bytecode progress authority and its bounded source witness passed
central verification: workspace/all-target `cargo check` and `cargo xc` were
green; the focused IR test passed `1/1` in `8.37s`; the structure executable
passed `5/5` in `22.36s`; the new CLI fixture passed `1/1` in `22.83s`; and the
retained quantifier CLI fixture passed `1/1` in `27.19s`. The exact Test262 file
now passes `2/2` with zero unsupported, crash or bug verdicts. No broader RegExp
or full-suite claim is made. Other Unicode properties of strings, arbitrary
runtime compilation, broad nullable-pattern closure and complete RegExp/String
coverage were outside that measured batch.

`OptionalAtomProgress` now makes each forward and reverse atom-nullability
classification a one-shot value: it derives no cloning or copying capability,
is created once after required iterations, and is consumed by the selected
finite or unbounded optional branch. The Rust-lexical guard owns the exact
12-mention, two-constructor and four-per-variant census together with both
complete ordered quantifier bodies, because an explicit recomputation cannot
be prohibited by the type alone. This source-equivalent hardening changes no
matcher-program instruction or evaluation order. The structure target remains
`5/5`, the exact IR unit passes `1/1`, and the nullable-progress and retained
quantifier CLI witnesses each pass `1/1`; formatting and scoped diff checks are
green. Independent review confirmed the exact route census and both complete
quantifier bodies. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings. Test262 was not rerun for this capability-only follow-up.

RegExp call and construction now have a bounded realm-correct allocation seam.
An undefined `NewTarget` becomes the exact entry- or created-realm active
RegExp constructor, explicit new targets receive one observable `prototype`
Get, primitive results fall back through the new target's required realm slot,
and tagged custom prototypes survive the sole result allocation. Direct
construct dispatch makes the RegExp body the owner of that Get and allocation.
The focused
[contract](../docs/rust-rewrite/contracts/regexp-constructor-realm-prototype.md)
and source-free cross-realm witness do not depend on dynamic Function source
generation. This does not implement `IsRegExp`/same-constructor early return,
cloning, flags override, general runtime pattern compilation or broader RegExp
protocol closure.

The emitted matcher now has a closed result-status ABI. All 45 result writers
must choose normal completion, corrupt-program failure or resource exhaustion;
the ordered-choice capacity guard is the sole current resource producer. The
wrapper uses the same typed resource route for its six scratch-arena preflight
failures, rewinds transient storage before routing a returned failure, and
returns a realm-correct `RangeError` before any post-match `lastIndex` write.
Corrupt artifacts retain their existing generic `Error`. One row source owns
the status words, constructors and messages, including string-pool interning.

The matcher failure route itself now carries no incidental clone, copy, debug,
equality or default capability. Its two status-row producers retain their exact
generic-`Error` and current-function-realm-`RangeError` mappings, the owner unit
observes both through an exhaustive projection, and the string wrapper remains
the sole exhaustive product consumer. A recursive structure regression pins
all eight source mentions and both throw bodies. This is source-equivalent and
changes no matcher status, message, Realm, rewind or `lastIndex` behavior; the
focused invariant is recorded in
`docs/rust-rewrite/contracts/regexp-matcher-failure-route.md`. Its structure
target passes `3/3`, the exact owner unit passes `1/1`, the neighboring runtime
entry-kind structure target passes `3/3`, and the package format check is
green. The hardened guard's independent dry review is clean, and `cargo xc`
plus repository checks are green; CLI, Test262, golden and broad-suite
verification remain deferred.

The ordered matcher result writer now consumes one private, non-capability
`RegExpMatcherResult::{Match, NoMatch, Failed(reason)}` authority instead of an
independent raw found word and status. Its sole exhaustive projection admits
only `(1, Complete)`, `(0, Complete)` and `(0, Failed(reason))`, so a found
failure or an arbitrary found ABI word cannot compile. The Rust-lexical guard
pins the exact 52 producers—one match, three normal misses, 46 corrupt-program
failures and two resource failures—across the matcher and its child modules,
together with the attribute-free domain and sole consuming writer. The original
source-equivalent ABI hardening added no runtime or conformance claim; its
focused structure target passed `4/4`, the neighboring nullable-quantifier
matcher-frame target passed `5/5`, and its CLI witness passed `1/1`. Test262,
golden and broad workspace verification were deferred for that invariant-only
batch. The current producer census must be rechecked when matcher source
changes, separately from behavior verification. The boundary is recorded in
[`regexp-matcher-result-domain.md`](../docs/rust-rewrite/contracts/regexp-matcher-result-domain.md).

This closes the raw status/current scratch-failure seam only. There is still no
deterministic execution-step budget or unified `RegExpResourceLimits`, and the
resource status is not expected to be reachable from a valid current program
under the exactly sized arena. No product hook or end-to-end exhaustion claim
is added ahead of that follow-up.

The runtime RegExp program-table kind now derives no incidental capability.
Its exact 0/1/2 wire projection and rejected-only SyntaxError policy borrow the
private three-row authority; the reader iterates `ALL` without copying and the
writer retains its three exhaustive encodings. The focused
[capability contract](../docs/rust-rewrite/contracts/runtime-regexp-entry-kind-capability.md)
and recursive lexical guard pin the ten source mentions, five direct word
calls, sole UFCS mapper and throw-policy route, exact writer arms, both Program
comparisons and borrowed reader pipeline. This is source-equivalent hardening,
not arbitrary runtime-pattern compilation or broader RegExp closure. The
structure target passes `3/3`, and the valid/invalid runtime-pattern CLI
witnesses pass `2/2`. Independent dry re-review is clean after the exact
constant authority, complete reader tail and no-overwrite writer tail were
pinned. The following shared workspace compile, formatter, module-boundary,
task-plan and diff gates all pass.

The IR carries the mutually exclusive legacy, `u` and `v` grammar modes as one
closed `RegExpUnicodeMode` from flag parsing through atom and character-class
dispatch. Compiled flags cannot represent both Unicode modes at once, and a new
mode must define its parser routing exhaustively.

Ordinary legacy/`u` classes now narrow that outer mode to a closed
`OrdinaryClassMode` before choosing an instruction representation. Both the
ASCII bitmap and code-point range parsers require that typed grammar mode and
enforce the same control, decimal/octal and identity-escape verdicts. Encoding
selection therefore cannot make Annex B escapes legal under `u` or change
`\cA` from U+0001 into literal class members. An incomplete legacy `\c`
preserves the standalone backslash and following `c` as two class members in
either representation. The focused
[contract](../docs/rust-rewrite/contracts/regexp-unicode-class-escape-grammar.md)
records the boundary and witnesses. This does not add arbitrary runtime
pattern compilation, close the dynamic-loop Test262 cases, or change the
UnicodeSets parser.

Named-group identifier classification now uses a closed start/continue domain
and the pinned ICU `ID_Start`/`ID_Continue` tables directly. The RegExp parser
no longer asks the third-party regex dependency to decide that product grammar
rule. The former shape-limited static generator membership fold has since been
removed with the obsolete lowering specializations. The remaining vendored
dependency supplies pinned Unicode property, string-sequence and fold data;
product matching goes through Lila's emitted program. This source census does
not replace the pending combined executable checkpoint.

Legacy direct astral source now has a typed term boundary. A validated UTF-16
surrogate pair cannot flow through the ordinary one-atom quantifier path: the
exhaustive term domain makes the lead mandatory and applies a following
quantifier only to the trail. The focused
[contract](../docs/rust-rewrite/contracts/regexp-legacy-direct-astral-quantifier.md)
and IR/Wasm witnesses distinguish that code-unit behavior from the whole-scalar
`u`/`v` rule. This does not close escaped-surrogate combinations, supplementary
case folding, the restricted lookbehind subset, or arbitrary runtime pattern
compilation.

Lookahead and lookbehind share the private, non-derived
`LookaroundPolarity::{Positive, Negative}` domain from `from_syntax_marker`
through typed `ParsedAtom` ownership and borrowed instruction construction.
The exhaustive `operand_bit` projection retains positive-zero and negative-one;
a separate closed matching direction records both the assertion and its caller.
Shared sentinels preserve captures, restore the input cursor, and discard private
alternatives after an assertion completes. Full lookahead Disjunction grammar
replaces the literal-byte shortcut, including nested assertions. The contract is
[`regexp-lookbehind-polarity.md`](../docs/rust-rewrite/contracts/regexp-lookbehind-polarity.md).
Reverse scalar/range/pair atoms now reach the existing matcher. The ASCII class
membership primitive rejects non-ASCII code points instead of aliasing their
low bits. Reverse backreferences and whitespace atoms remain explicit gaps.

The `v`-mode class parser now commits to one closed expression shape after its
first typed operand: union, homogeneous intersection, or homogeneous
subtraction. A private operator enum owns delimiter and range semantics, and
distinct tail parsers reject mixed operators, implicit operand unions and
missing operands with a cited `ClassSetExpression` syntax rule. The focused
[contract](../docs/rust-rewrite/contracts/regexp-unicode-set-expression-shape.md)
and IR/Wasm witnesses keep valid chained operations live. A private validated
`ClassSetCharacter` boundary rejects raw syntax characters and all reserved
double punctuators while preserving escaped operands such as `[a&&\&]`, and
enforces the decimal-digit lookahead after `\0`. A validated `\q{…}` stays a
typed operand through outer closure, range and operator validation plus the
exact §22.2.1.8 `MayContainStrings` negation early error. The typed capability
marker then survives the complete Pattern group, named-reference, and
nullable-group unbounded-quantifier checks; only a globally valid Pattern
remains an explicit unsupported capability. That validation boundary now feeds
finite class-string matching and all seven finite provider properties of strings.
The prepared direct class-string `/iv` successor removes the private deferred
capability only after operand folding and established matcher lowering are
connected. Fresh runtime verification and full UnicodeSets conformance remain
open.

The emitted range-pool reader now accepts one closed `RegExpRangeBound`
instead of an arbitrary byte offset. Its exhaustive projection preserves the
encoded `(start, end)` layout at offsets zero and four, while the two matcher
callers retain only the semantic range-mismatch operation. The private
`builtins/regexp/range_search.rs` child now owns the domain, exhaustive
projection, sole raw reader and complete binary search, so the parent cannot
construct a bound, project its offset or call the raw reader. The bounded
[range-search ownership contract](../docs/rust-rewrite/contracts/regexp-range-bound-domain.md)
and `regexp_range_bound_domain_structure` target record the projection, typed
reader and recursive ownership census. The exact 14-line domain/projection and
101-line search/reader selections retain visibility-normalized SHA-256
`7ac765b2195a8ad7e2935bbfb3da1b9e8e641a63906bb007eb67ea49e0da17b6`
and
`eb9ceaad299ab3277aa5bbf1228776d74098876f79f25bed106179e699489098`;
their combined 115-line hash is
`14e35ba4c6a910319e4e301ded5213315ba30fe537a3d92fcd2c3207b29b7801`.
The resulting 3,661-line parent and 120-line child have SHA-256
`60a443e0f39f719c28871815f5be6c7a7fd638e8389e6070167db05abc09b30b`
and
`c36626fb9c53468a49449012538a9ad32e80c37c9387ee7252d807864f9c9e8f`;
both unchanged parent calls retain SHA-256
`125fa46e9fab12f49c12f6280b95f618baa2d1cb88fad021fb7fc26029e63ab2`.
The existing
`wasm_regexp_exec_unicode_property_program` fixture is the direct behavior
witness for first-range start, a following gap, final-range end and the first
excluded code point after it. The structure target passes `3/3`, the focused
Unicode-property CLI fixture passes `1/1`, `cargo xc` is green, and the
neighboring matcher-result and Unicode-sets targets pass `4/4` and `7/7`. These
results were rerun after the source-equivalent Batch AA owner move. The earlier
647-artifact Wasm golden has an empty recursive pre/post diff, but it was not
rerun for Batch AA. This is a Rust decoder invariant only and makes no broader
RegExp conformance claim.

The complete RegExp GetSubstitution policy now lives in the private
`builtins/string/regexp_substitution.rs` child. Its non-copyable six-kind
domain, ordered runtime-code projection, every recognizer and the exhaustive
semantic handler moved with the sole algorithm; the String parent retains one
semantic call and cannot name or encode the raw policy. The exact 30-line
domain/authority and 448-line algorithm retain visibility-normalized SHA-256
`0f852520992bfe2689f1ba08c1351c8accc5921373cbb32c2ac1f493b56ab453`
and
`d11dd555a3b82a43496296de74b04367c50ffa0fc2b148f8c2b1eb2453ee0d8d`;
their combined 478-line hash is
`c8deaa00580f7d7a74e684273325a4e7b496c3aa39f69d66a7b7da8cfb02f2dd`.
The resulting 20,970-line parent and 483-line child have SHA-256
`62caf68bd5a9bc02354c8fdc31b1d73d467a374d68a82717561adcf810a2dd3f`
and
`5163b1c56b48ee90a6f3ee5ea6f5c19ad013ea1462090c526e3e951bee43a473`;
the unchanged parent call retains SHA-256
`fcecb3ddcc9b61f06b276734b76b5c04211dffb75d962f0f01c0e2f43a862b8a`.
The recursive guard and module policy pin zero parent raw-policy names, all 15
domain mentions, all four runtime-code projections and the one semantic call.
This source-equivalent Batch AB move changes no substitution behavior. At the
shared checkpoint, `cargo xc` is green, the owner target passes `4/4`, the
neighboring flag-getter and literal-replacement targets pass `3/3` each, and
the six substitution leaves pass all `12/12` Wasm-AOT variants with every
failure bucket at zero. No CLI fixture or emitted-Wasm golden was run. The
bounded contract remains
[`regexp-substitution-kind.md`](../docs/rust-rewrite/contracts/regexp-substitution-kind.md).

The following three child-owner extraction receipts are historical: the T19
ordinary `@@match` retirement deletes these unreachable owners and their nine
layout-only structure tests. They do not describe current product dispatch.

The complete duplicate-named-group pattern policy previously lived in the private
`builtins/string/duplicate_named_group_pattern.rs` child. Its capability-free
two-variant domain and sole raw pattern-parameterized emitter moved together;
the String parent retains only the alternative-captures and
iterated-backreference semantic calls. The moved four-line domain and 80-line
emitter retain SHA-256
`38391f8c3eaadf1cd997b13fffba38dccf8a017955d3bb75b48eb3e587af7280`
and
`bcd1693a0ff5292fa826e8449162eb85e7dedcec857aa0b76ef7b9d5c3bdd387`,
with combined hash
`3a5aa0f6afbd361cf6e88724d0c2e4a4bb1f559b5b0a81a15affd68c455063ee`.
The resulting 20,883-line parent and 125-line child have SHA-256
`6a8e1b8fb5d7f05b0bfaba1d8196dab577aac30a5a65cbde37c10291321cc984`
and
`9cb88aa5ee221e66911a1070062e7e15242aaa91585562dbfba51d4c709ee560`.
Recursive structure and module policies pin zero parent raw-policy names, six
child policy mentions, the one raw definition plus two child calls, and the two
parent semantic calls. At that historical checkpoint, the raw owner was byte-equivalent; only the
parent call spelling was narrowed. At the Batch AC shared checkpoint, `cargo xc` is
green, the structure target passes `3/3`, the exact CLI fixture passes `1/1`,
and the exact String match ordinary-groups and indices-groups leaves pass all
`4/4` variants with every failure bucket at zero. The semantic golden was not
rerun. The bounded contract remains
[`duplicate-named-group-pattern.md`](../docs/rust-rewrite/contracts/duplicate-named-group-pattern.md).

Internal RegExp execution accepts the private closed `RegExpExecResultMode`.
Its two variants bind intrinsic exec and the noncallable-`exec` fallback in
`@@match` to Array/null and the
intrinsic `test` fallback to Boolean. The wrapper owns one value and lends it
to the sole compiled-program matcher; five exhaustive projections choose
materialization and capture-carrier lifetime. The source guard pins both
signatures, unchanged projection bodies, the 16-mention ownership census,
nonreturning rejection and exact three-intrinsic-producer mapping. The
[contract](../docs/rust-rewrite/contracts/regexp-exec-result-mode.md) records
current verification commands and the remaining boundary.

Before runtime rejection and fallback retirement, the source-equivalent
result-mode closure retained three consumers, seven projections and a
21-mention census. Its structure target passed `3/3`, the result-shape CLI
fixture passed `1/1`, and the shared format, `cargo xc`, diff, module-boundary
and task-plan checkpoint was green with existing workspace warnings. Its
following semantic golden passed `2/2` in 707.16 seconds with 665 dumps,
preserving 663 of 664 retained non-accounting summaries; the sole structural
change was the independently expanded Promise Realm witness. These are
historical receipts for the previous boundary, not verification of the new
corrective batch.

The eight intrinsic RegExp Boolean flag getters now cross standard dispatch
through the closed, sibling-visible `RegExpFlagGetter` domain instead of
passing a broad builtin ID into their shared emitter. Eight exact producers
name the `hasIndices`, `global`, `ignoreCase`, `multiline`, `dotAll`, `unicode`,
`unicodeSets` and `sticky` rows; one borrowed exhaustive match projects only
those rows to `d/g/i/m/s/u/v/y`. The focused
[contract](../docs/rust-rewrite/contracts/regexp-flag-getter.md) and recursive
two-source guard record the boundary. The structure target passes all `3/3`
tests, the existing accessor CLI witness passes `1/1`, and eight exact pinned
leaves, one per getter, pass both variants (`16/16`) with every failure bucket
at zero. Workspace formatting and the diff check are green. This invariant adds
no flag syntax or broader RegExp conformance claim.

Batch AZ makes `RegExpFlagGetter` and its raw emitter private to
`builtins/string.rs`. Standard dispatch reaches them only through eight fixed RegExp flag-getter entries.
The frozen 93-line domain/emitter selection has SHA-256
`0bd635a1625364b6db7514af3ce13b96166d14614f9ec5ee5c6f7b25fbd76829`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The strengthened flag-getter and neighboring
symbol-hook structure targets pass `4/4` and `5/5`; the complete RegExp
prototype-accessor Wasm-AOT CLI fixture passes `1/1`. No Test262 leaf or Wasm
golden was required for this source-equivalent boundary, which claims no new RegExp behavior,
conformance result or published-count change.

String methods that dispatch through RegExp well-known-symbol hooks now cross
their shared emitter through T18's closed `StringSymbolHookOperation` domain.
That boundary exhaustively owns `matchAll`/`replaceAll` global validation and
the inherited `%RegExp.prototype%[@@matchAll]` path as well as ordinary custom
hook arity and literal fallback selection. `String.prototype.split` remains a
separate direct emitter. The focused
[contract](../docs/rust-rewrite/contracts/string-symbol-hook-operation.md)
records the integration seam; it does not add RegExp syntax or matching
coverage. Its structure target passes `4/4`, the shared symbol-hook CLI witness
passes `1/1`, and the six exact String entry leaves pass both variants
(`12/12`) with every failure bucket at zero.

RegExp modifier-group multiline and dotAll state now crosses the static IR and
matcher ABI through the non-copyable
`RegExpModifierOverride::{Inherit, ForceOn, ForceOff}` domain instead of two
`Option<bool>` fields and independently spelled numeric modes. Initial, added
and removed states name those variants; two inline exhaustive matches carry
unaffected state into a nested group, and the parser restores the moved outer
state before propagating a nested parse error. One exhaustive projection owns
the unchanged operand codes `0/1/2`; the inherent dot and start/end assertion
constructors explicitly name the inherited code, and the modifier application
rows project dotAll and multiline state. The Wasm matcher names the same
force-on and force-off codes when selecting effective multiline and dotAll
behavior. The focused
[contract](../docs/rust-rewrite/contracts/regexp-modifier-override.md), IR
tests and cross-source structure guard record the boundary. The structure
target passes `5/5`, both focused IR tests pass `1/1`, the Wasm-AOT modifier
fixture passes `1/1`, and `cargo xc` is green. The exact pinned
`add-dotAll.js` leaf remains `0/2` with `NotImplemented:Runtime` at a broader
unsupported `RegExp.prototype.exec` pattern, so no modifier-subtree conformance
claim is made. This source-equivalent invariant does not change modifier
syntax, top-level flags, case folding, malformed bytecode handling or broader
RegExp conformance.

## Objective

Implement the ECMAScript regular-expression grammar, matching model and observable object protocol for every feature in the pinned suite. Treat the current Rust regex dependency as an implementation component only where its behavior exactly matches ECMAScript; do not expose host-regex semantics as JavaScript semantics.

## Engine strategy

The selected design document evaluates:

- translating ECMAScript patterns into a compatible Rust engine plus Lila-managed semantics;
- extending/forking the current engine for missing features;
- implementing a dedicated bytecode/NFA/backtracking engine compiled into Wasm;
- hybrid specialized engines selected by pattern features.

The decision is one Lila-owned ordered-backtracking bytecode model: Rust compiles
static patterns, a RegExp-only compiler in emitted Wasm compiles arbitrary
runtime patterns, and both feed the same iterative Wasm matcher. A linear-time
specialization is deferred until the reference engine is complete and its
admitted feature set can prove observational equivalence. The design supports
lone-surrogate-aware UTF-16 matching, observable `lastIndex`, captures and all
pinned syntax without a host JavaScript engine, a JavaScript interpreter, or
known-pattern recognition.

## Pattern parsing and validation

Implement pattern parsing separately from JavaScript source parsing, with source spans and realm-correct `SyntaxError`s. Cover the pinned forms of:

- literals, alternatives, assertions, quantifiers and character classes;
- capturing/non-capturing/named groups and backreferences;
- lookahead and lookbehind;
- Unicode escapes, property escapes and script/category aliases;
- `u` and `v` Unicode modes, set operations, string properties and class-string disjunctions;
- duplicate named-group rules and early syntax validation;
- decimal/octal/legacy Annex B interpretation where required.

## Flags and matching state

Support all pinned flags and combinations: `d`, `g`, `i`, `m`, `s`, `u`, `v`, `y`, including duplicate/invalid flag errors, canonical `flags` ordering and accessors. Matching must correctly handle:

- UTF-16 code-unit positions and Unicode code-point advancement;
- empty matches and `AdvanceStringIndex`;
- sticky/global start behavior and `lastIndex` read/write/coercion;
- multiline anchors, dotAll, word boundaries and Unicode ignore-case folding;
- captures, unmatched captures, named groups and `d` indices;
- backtracking/capture restoration and lookaround semantics.

## RegExp object protocol

Implement:

- `RegExp` call/construct semantics, cloning, flags override and custom new target;
- exact prototype accessors/descriptors and source escaping;
- `RegExp.prototype.exec`, `test`, `compile` if present, and `toString`;
- `RegExpExec` abstract operation and custom `exec` dispatch;
- species construction and subclass/cross-realm behavior;
- `Symbol.match`, `matchAll`, `search`, `replace` and `split`;
- `IsRegExp` via `Symbol.match`, including proxies and abrupt getters.

## String integration

Coordinate with T18 so String methods first perform well-known-symbol dispatch. Implement replacement substitution tokens (`$$`, `$&`, ``$` ``, `$'`, `$n`, `$<name>`), functional replacements, captures/groups argument lists, split captures/limits and match-result array shapes/descriptors.

## Performance and safety

- Add deterministic step/resource limits that produce an explicit runtime failure during development rather than hanging the suite.
- Prevent Rust stack overflow on deeply nested patterns or adversarial input.
- Benchmark catastrophic-backtracking patterns and optimize without changing match order/capture semantics.
- Cache compiled patterns only when constructor/proxy/custom-exec behavior makes it unobservable.

## Acceptance criteria

- Full pinned `built-ins/RegExp`, RegExp literal and String regex-method filters are green.
- Every flag and syntax feature has positive, negative and interaction tests.
- Matching uses ECMAScript UTF-16/Unicode semantics, including lone surrogates.
- Custom `exec`, species, subclassing, proxies and `lastIndex` property effects are observable in correct order.
- No exact pattern/test-path materializations remain.
- Adversarial patterns cannot panic the Rust host or corrupt Wasm memory.
- Timeout counts for RegExp subtrees are zero at the normal publication timeout.

## Required tests

```sh
cargo test -p lila-ir regexp_ --quiet
cargo test -p lila-aot-wasm regexp_ --quiet
cargo test -p lila-cli wasm_regexp --quiet
./target/debug/lila test262 run built-ins/RegExp --execution-backend wasm --timeout-ms 180000 --threads 4
```

Also run RegExp literal grammar tests and the String `match`, `matchAll`, `search`, `replace`, `replaceAll` and `split` subtrees.
