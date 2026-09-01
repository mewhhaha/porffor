# T19 — Complete ECMAScript RegExp semantics

**Status:** In progress — the ordered-bytecode engine architecture is fixed; dynamic compilation and broad grammar remain incomplete

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10, T18  
**Blocks:** String-RegExp integration and RegExp-related T26 closure

## Current repository state

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
between forward and reverse paths. Other Unicode properties of strings and
direct class-string `/iv` folding remain distinct typed capability boundaries;
this records no broader UnicodeSets or RegExp completion claim.

The adjacent Unicode-property-of-strings batch now gives Unicode 17
`Emoji_Keycap_Sequence` an exact finite representation: the twelve strings
`[#*0-9] FE0F 20E3`. Direct `\p{Emoji_Keycap_Sequence}` atoms and `v`-mode
union, intersection and subtraction all consume the existing canonical
`FiniteClassSet`; the direct `iv` form shares the same bytecode because every
member is simple-case-fold invariant. Other properties of strings retain their
typed unsupported capability, and negated classes that may contain strings
retain their required early error. At clean pre-batch commit `04e38f2ba`, exact direct-property file
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
finite keycap property; Basic_Emoji, the remaining RGI properties, general
Unicode property data and complete RegExp conformance remain open.

The property-of-strings authority is now closed at the vendored provider
boundary without changing that behavior. The provider crate root narrowly
re-exports its strict parser, seven-variant `UnicodeStringProperty` and
read-only sequence accessor while keeping the generated table module private.
Lila parses once into that domain and projects every variant in an exhaustive,
catch-all-free match: only `EmojiKeycapSequence` consumes the provider's exact
twelve rows into `FiniteClassSet`; each of the other six variants remains the
typed unsupported capability. The duplicate handwritten keycap construction
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
runtime pattern compilation, broad
nullable-pattern closure and the complete RegExp/String trees remain outside
this batch.

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
pins the exact 50 producers—one match, three normal misses, 44 corrupt-program
failures and two resource failures—together with the attribute-free domain and
sole consuming writer. This is source-equivalent ABI hardening and adds no
runtime or conformance claim. The focused structure target passes `4/4`, and
the neighboring nullable-quantifier matcher-frame target passes `5/5`; its
focused CLI witness passes `1/1`. Test262, golden and broad workspace
verification remain deferred. The boundary is recorded in
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
rule. That dependency remains in a separate shape-limited static generator fold
whose accepted results can influence emitted IR; it must be proven against the
Lila engine or removed.

Legacy direct astral source now has a typed term boundary. A validated UTF-16
surrogate pair cannot flow through the ordinary one-atom quantifier path: the
exhaustive term domain makes the lead mandatory and applies a following
quantifier only to the trail. The focused
[contract](../docs/rust-rewrite/contracts/regexp-legacy-direct-astral-quantifier.md)
and IR/Wasm witnesses distinguish that code-unit behavior from the whole-scalar
`u`/`v` rule. This does not close escaped-surrogate combinations, supplementary
case folding, the restricted lookbehind subset, or arbitrary runtime pattern
compilation.

Lookbehind polarity now remains the private, non-derived
`LookbehindPolarity::{Positive, Negative}` domain from its sole syntax-marker
producer, `from_syntax_marker`, through `ParsedAtom` ownership and the four
borrowed lowering uses. Only one exhaustive `operand_bit` projection emits the
unchanged positive-zero and negative-one matcher ABI, so end and failure
instructions cannot receive independently spelled Booleans. The focused invariant and evidence live in
[`regexp-lookbehind-polarity.md`](../docs/rust-rewrite/contracts/regexp-lookbehind-polarity.md).
This source-equivalent boundary adds no grammar, reverse matcher or broader
RegExp conformance claim.

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
finite class-string matching and the finite Emoji Keycap property; it does not
implement other Unicode properties of strings or full UnicodeSets conformance.

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

The complete duplicate-named-group pattern policy now lives in the private
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
parent semantic calls. The raw owner remains byte-equivalent; only the parent
call spelling is narrowed. At the Batch AC shared checkpoint, `cargo xc` is
green, the structure target passes `3/3`, the exact CLI fixture passes `1/1`,
and the exact String match ordinary-groups and indices-groups leaves pass all
`4/4` variants with every failure bucket at zero. The semantic golden was not
rerun. The bounded contract remains
[`duplicate-named-group-pattern.md`](../docs/rust-rewrite/contracts/duplicate-named-group-pattern.md).

Internal RegExp execution now accepts one private closed
`RegExpExecResultMode` instead of threading a `return_boolean` Boolean through
the wrapper, bytecode-program matcher and simple matcher. The two variants bind
non-global `@@match` and `exec` to Array/null results and bind the intrinsic
`test` fallback to Boolean results. Seven direct exhaustive matches also make
the bytecode matcher's capture-carrier allocation and rewind policy explicit,
without adding a runtime word or changing instruction and local ordering. The
bounded source target pins the two variants, exact three consumers, seven
projections and exact three-producer census and passes `3/3`. A finite Wasm CLI
witness selects the bytecode-program, simple and final legacy fallback paths,
distinguishes their Array/null and Boolean success/failure results plus
`lastIndex` effects, and passes `1/1`. The callable custom-`exec` protocol,
global `@@match`, arbitrary runtime compilation, broader grammar and matcher
coverage remain open; the focused
[contract](../docs/rust-rewrite/contracts/regexp-exec-result-mode.md) records
the boundary and explicit deferrals.

The result-mode authority now derives no cloning, copying, debugging, equality
or default capability. Its wrapper owns the single value, lends it in program-
then-simple matcher order, and consumes it only in the existing final
exhaustive projection; the borrowed matcher parameters retain their six
existing exhaustive projections. The strengthened structure target pins that
ownership, forwarding order and the recursive 21-mention capability census.
This capability hardening is source-equivalent and expected to leave
emitted Wasm byte-identical. Independent dry review is clean after strengthening
the guard's exact signatures, seven lexical body fingerprints and
borrow-borrow-consume order. The shared format, `cargo xc`, diff,
module-boundary and task-plan checkpoint is green with the workspace's existing
warnings.

The following workspace semantic golden passes `2/2` in 707.16 seconds and
contains 665 dumps. It adds only the result-mode fixture, removes none and
preserves 663 of 664 retained non-accounting summaries; the sole retained
structural change is the independently expanded Promise Realm witness.

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
