# T18 — Strings, Unicode and the complete String API

**Status:** In progress — UTF-16-aware primitives exist; T18 materializers are retired and broad API closure remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10; regular-expression methods integrate with T19  
**Blocks:** T19, T22, T23 and String-related T26 closure

## Current repository state

Heap strings and many String methods operate on UTF-16 code units, with focused
coverage for surrogates, case conversion and symbol hooks. The Test262 harness
has no remaining T18-owned shortcut observations, while full Unicode
normalization/case data and RegExp/Intl integration are not proven complete.
The complete current-pin String tree and T18 acceptance gate remain open.

The metadata cases for `String.prototype.at`, `charAt`, `charCodeAt`,
`codePointAt`, `includes`, `indexOf`, `lastIndexOf`, `startsWith`, `endsWith`,
`match`, `matchAll`, `search`, `repeat`, `padStart`, `padEnd`, `trim`,
`trimStart`, `trimEnd`, `toString`, `valueOf`, `isWellFormed` and
`toWellFormed` now run their pinned sources through the shared Wasm-AOT
`propertyHelper.js` and general builtin metadata path; their path-specific
rewrites have been removed. The exact shortcut inventory now assigns zero
observations to T18.
The `Array.prototype.toString` non-callable-`join` case also runs its unchanged
pinned source with the full assertion harness. The shared Object fallback now
uses Proxy-aware `IsArray` and `IsCallable` decisions and complete builtin-brand
classification before `@@toStringTag`; both ordinary Wasm-AOT variants pass.
Revoked Proxy errors come from the borrowed builtin function's Realm. Removing
its exact materializer retired two T18 semantic observations.
The two unchanged `%TypedArray%.prototype.toString` identity/descriptor and
non-constructor cases now use ordinary materialization with the complete
vendored `propertyHelper.js`, `testTypedArray.js` and `isConstructor.js`
contracts. Their shared Array/TypedArray function identity and non-constructor
semantics no longer depend on a source fingerprint or reduced harness prelude;
removing that predicate retired one more T18 semantic observation. Both exact
files report `2/2` ordinary Wasm-AOT variants as of `2026-08-26`, with every
failure and unsupported bucket at zero.
The Unicode `String.prototype.matchAll` case now also executes its unchanged
pinned source with the full `compareArray.js` harness and ordinary Array
mapping. Both sloppy and strict Wasm-AOT executions pass. The adjacent null and
undefined cases also execute unchanged with the complete iterator and RegExp
helper harness for all four sloppy/strict variants.
The private `EcmaTrimMode::{Start, End, Both}` authority now derives no
incidental capabilities. Its first exhaustive scan borrows the owned mode and
its second consumes that same mode, retaining start-before-end behavior without
copying policy state. The recursive lexical guard pins the attribute-free
three-row declaration, eleven source mentions, four appearances per row, the
three exact wrappers, both complete scan bodies and all existing String and
BigInt caller/coercion order. This is source-equivalent lifecycle hardening;
it adds no runtime Wasm mode word and changes no trim behavior. The structure
target passes `2/2`, and the exact trim and arbitrary-precision BigInt CLI
witnesses pass `2/2` in aggregate. Independent dry review is clean, and the
following shared workspace compile, formatter, module-boundary, task-plan and
diff gates all pass.
The two non-`eval` Sputnik `charAt` receiver cases, the direct `charAt`
position-coercion/rounding cases and the plain Array `toString` conversion
matrix also run their pinned source now; existing product fixtures already
encode those same receiver, conversion and assertion contracts.

All thirteen non-dynamic Sputnik `String.prototype.slice` cases covered by the
legacy materializer (`S15.5.4.13_A1_T1`, `T2`, `T4`, `T6` through `T15`) now
run their pinned sources with only the shared `sta-preamble.js` definition
required by their `Test262Error` assertions.

Nine non-`eval` Sputnik `String.prototype.match` cases (`S15.5.4.10_A2_T1`,
`T6` through `T11`, `T17` and `T18`) likewise run their pinned bodies rather
than materializations that cached a single match result. The final five
dynamic-source materializers are now retired too:
`charAt/S15.5.4.4_A1.1.js`, `charCodeAt/S15.5.4.5_A1.1.js`,
`indexOf/S15.5.4.7_A3_T2.js`, `match/S15.5.4.10_A1_T3.js` and
`slice/S15.5.4.13_A1_T5.js`. An exact invariant pins their vendored bodies,
empty metadata, sloppy/strict modes, both prelude profiles and typed compiler
diagnostics. The spec-exec oracle passes all ten executions. Wasm-AOT passes
`0/10` and reports all ten as typed `Unsupported`: the four direct-`eval`
sources require the caller-environment seam and the ordinary-`Function`
source requires the target-Realm-environment seam owned by T13. These visible
capability gaps replace the former fake-green rewrites. The six adjacent
non-dynamic controls pass all `12/12` sloppy/strict Wasm-AOT executions.

The `isWellFormed` and `toWellFormed` primitive-coercion leaves now also run
their exact pinned sources. Normal materialization supplies the shared
`sta-preamble.js` definition required by `Test262Error` and the general
`assert.js` contract; the former destructuring-free path rewrites are gone.
Their exact source/harness contract is dry-written, while the focused Rust, CLI
and two one-file Test262 execution gates remain queued behind that matrix.

The non-generic cross-realm `String.prototype.toString` and `valueOf` leaves
now run their exact pinned bodies as well. General materialization activates
the realm-aware `sta.js` boundary for `$262.createRealm()`, supplies the shared
`assert.js` contract and preserves the original `assert.throws` checks against
the other realm's `TypeError`. The former `instanceof`-only handwritten
rewrites are gone. Their exact source, active-realm and prelude-order contract
is dry-written; focused Rust, CLI and two one-file Test262 execution gates are
explicitly pending while the current-pin matrix owns runtime verification.

The direct `String.fromCharCode` lowering now consumes the shared exact
`ToUint32` residue emitter before selecting the low 16 bits. It no longer
narrows to a saturating signed `i64` first, so infinities and finite magnitudes
outside the signed-64-bit interval obey `ToUint16` rather than becoming
`0xffff` or `0x0000` by accident.

The ordinary empty-string `String.prototype.split` path now walks the
receiver's UTF-16 code-unit domain rather than advancing one result per decoded
UTF-8/WTF-8 scalar. Its one-unit materialization boundary cannot accept a raw
byte index, and every result element uses the shared UTF-16 range materializer,
so an astral scalar splits into its independently observable high and low
surrogates while lone surrogates remain intact. The authoritative length and
range helpers still decode the storage representation internally. A focused
product fixture covers literal and escaped pairs, mixed BMP/astral and empty
input, lone and reversed surrogates, limits, boxed receivers, `charAt` parity
and join round-tripping. The code, fixture, static structural guard and contract
are dry-written; Cargo, CLI execution and focused pinned split leaves remain
queued behind the active current-pin matrix.
The coordinator's private index, length and one-unit local domains now derive
no incidental capabilities. Its one-unit materializer borrows the index and
width, leaving the loop owner responsible for index advancement and the final
LIFO release without copying either proof carrier. The Rust-lexical
ownership guard pins the attribute-free declarations, the borrowed boundary,
the exact constructions and the post-call uses; it passes `4/4`. This is a
source-equivalent ownership closure, so no runtime or conformance claim is
added.

The `String.prototype.charAt` and `at` paths now share a private typed
code-unit-access coordinator. Both ordinary standard-builtin dispatch and the
optimized direct `charAt` call evaluate receiver and argument expressions
before the coordinator performs receiver/index coercion. The coordinator owns
opaque UTF-16 index, length and one-unit locals; its one-unit materializer can
call only the authoritative UTF-16 range operation, so an astral scalar's high
and low surrogates are independently observable rather than becoming a whole
scalar and an empty byte slice. The two named entry points fix `charAt`'s empty
String miss and `at`'s `undefined` miss without a caller-supplied policy. The
code, astral and ordering fixtures, structural guard and normative contract are
dry-written; Cargo, CLI and focused pinned execution remain queued behind the
active current-pin matrix. General `slice`/`substring` range extraction remains
an explicit adjacent seam and is not closed by that change.

The adjacent `String.prototype.slice` and `substring` paths now share a private
typed code-unit-range coordinator. It derives the UTF-16 String length,
normalizes both already-evaluated arguments through a closed method policy and
constructs a non-`Copy` materializable range token. The token's sole consuming
boundary uses the authoritative UTF-16 range operation; neither raw byte
slicing nor code-unit-to-byte boundary conversion is admitted. This preserves
the independently observable halves of an astral scalar and also fixes the
standard `substring` body, which previously treated every normalized UTF-16
index as a byte offset. The optimized direct `substring` call now delegates to
the standard builtin after the complete argument vector is evaluated, so
receiver coercion no longer runs before argument expressions and surrounding
`try` structure cannot select a parallel algorithm. Nullish errors use the
executing builtin function's Realm. Both shared index normalizers now saturate
finite values outside signed-64 range before their String-length clamp, so a
large finite index cannot trap during Wasm integer conversion. Annex B `substr`
already used the same
authoritative materializer and is structurally pinned as a non-regression. The
code, astral/BMP/lone-surrogate and ordering/Realm fixture, structural guard and
normative contract are dry-written; Cargo, CLI and focused pinned execution
remain queued behind the active current-pin matrix. Static-name method
misdispatch, the dynamic-`Function` slice materializer and the broader String
API remain explicit nonclaims.

The `String.prototype.repeat` count path now feeds `ToNumber` through the
shared `ToIntegerOrInfinity` operation before applying repeat's negative and
positive-infinity rejection. Negative fractions therefore normalize to zero
instead of being rejected from their raw Number value. Accepted finite counts
use a saturating unsigned Wasm conversion, so magnitudes outside the `u64`
domain cannot trap before the existing empty-string fast path or unsigned
implementation-limit comparison selects the ECMAScript-visible result. Both
the invalid-count and result-too-large `RangeError` paths now use the executing
repeat function's Realm. The fixture covers negative fractions, enormous
finite counts on empty and nonempty receivers, and both errors through a
created Realm's borrowed repeat method. On 2026-08-24,
`cargo check -p lila-aot-wasm` and `cargo xc` passed, the hardened structural
target passed `4/4`, and the exact CLI fixture passed `1/1`. At current Test262 pin
`e9d582d6b8b13afc5ba9a676664741592b5c7f69`, all 16 unrewritten repeat files
materialized as 32 ordinary sloppy/strict Wasm-AOT executions and passed
`32/32` with every failure bucket at zero. The direct files do not contain the
negative-fraction, finite-above-`u64` or created-Realm cases; the CLI fixture
owns those observations. This does not change the maximum String size, general
numeric conversion, complete String-tree status or published conformance
counts, and emitted Wasm was not byte-compared.

Computed property reads on primitive Strings now cross lowering through one
private closed `CanonicalIndex` / `OrdinaryPropertyKey` classification. Static
proof may select the indexed IR path; every other value is preserved for the
general `ToPropertyKey` path instead of being rejected merely because its
lowered kind is not Number. The backend performs dynamic canonical numeric
index recognition across the full non-negative integral `u64` domain, rejects
`-0`, fractions and non-finite values as String exotic indices, and falls
through to `%String.prototype%` for non-index and out-of-bounds keys. This
removes the general `"string index must be number"` unsupported boundary that
owned the two pinned `15.5.5.5.2-{1,3}-2.js` failures. The structural contract
test passes, both exact files now report `2/2` execution variants, and the
adjacent `built-ins/String/15.5.5.5.2` family reports `28/28` under Wasm-AOT at
the harness-declared `aa55200d1310384c5cf69ea95b2a2ecba457007b` pin. This is
focused leaf evidence, not complete String-tree or current publication proof.

Normalization form selection now crosses the Wasm emitter through the closed
`StringNormalizationForm::{Nfc,Nfd,Nfkc,Nfkd}` domain instead of paired
compatibility/composition booleans. Exhaustive projections own the accepted
spelling, runtime branch code, decomposition table and composition policy;
`normalize` retains its existing argument coercion, validation and dispatch
order, `localeCompare` names NFC explicitly, and the String pool interns the
four spellings by walking the same `ALL` domain. The product fixture now
executes NFKD directly. On 2026-08-26, the focused structural witness passed
`4/4`, and the normalization and locale-compare product filters each passed
`1/1`. Broad workspace, Test262 and golden verification were intentionally
deferred; this invariant does not close locale/options-aware collation or
complete Unicode conformance.
The form authority now derives no incidental capabilities. The normalization
emitter borrows its single owned form for both decomposition passes and then
consumes it for the composition decision; spelling borrows before runtime-code
selection consumes each validation-loop form. This is source-equivalent
lifecycle hardening and adds no runtime Wasm form word or conformance claim.

The static global ASCII class matcher uses the private, non-copyable
`GlobalAsciiClassQuantifier::{DigitOnce, DigitTwice, NonDigitTwice}` domain
instead of an independent digit-polarity Boolean and arbitrary integer width.
Batch AD moved that domain and the parameterized emitter into one private
child; the parent now exposes only the `digit once`, `digit twice` and
`non-digit twice` semantic calls. Two direct exhaustive matches own width and
predicate polarity while the existing scalar decode, non-overlapping scan
advancement and final mismatch logic remain unchanged. The moved five-line
domain and 203-line emitter retain SHA-256
`2c70e7cfdceb62904b990196833997be1cfb643595987e38f8871942bfc49860`
and
`6a7fce3d1705ae08dbd92d96b2046445a07c2740bb314f882d7cf4f6a4320211`.
Batch AD focused structure and CLI verification is deferred to the combined
checkpoint; the three exact pinned leaves and semantic golden were not rerun.
It does not complete String, RegExp, T18 or T19.

The specialized postal-code match-result shape and raw emitter now have one
private `builtins/string/postal_code_match_result_shape.rs` owner. The parent
can request only the global or exec semantic operation and cannot name,
construct, import or project the private, non-copyable
`PostalCodeMatchResultShape::{GlobalMatchArray, ExecMatchArray}` domain. Two
exhaustive matches still own the one-versus-three-element Array length and the
none-versus-full capture, UTF-16 `index` and `input` publication. The moved
four-line domain and 357-line raw emitter retain SHA-256
`2c218b01e482cf283729f52db2c171b9dddd0d6fbe1d4eac5bf2fb79fdc0ac71`
and `06fe70a126949e33e1cba69b6f349cf83d960a8e9961eecf65bbf5fc33c540d8`.
The resulting 398-line private child has SHA-256
`fc2d538c93855feb1e1f011af9d2851d42f9b6c8db6f59a15387ea93e89088b4`.
Shared match discovery, optional-capture `undefined`, the full-match element
and no-match `null` remain unchanged. At the Batch AI shared checkpoint,
`cargo xc` exits zero; `postal_code_match_result_shape_structure`,
`string_literal_replacement_scope_structure` and
`global_ascii_class_quantifier_structure` each pass `3/3`, for `9/9` total.
The exact
`string::run_wasm_backend_succeeds_for_string_match_postal_code_fixture` CLI
witness passes `1/1`, and
exact `S15.5.4.10_A2_T6.js`, `S15.5.4.10_A2_T7.js` and
`S15.5.4.10_A2_T8.js` pass sloppy and strict execution (`6/6`) with every
failure bucket at zero. No semantic golden was needed or run. Final formatter,
diff, module-boundary, task-plan and 240-entry shortcut-inventory gates are
green. The source-equivalent owner move adds no fixture or conformance claim.
It does not complete String, RegExp, T18 or T19.

The duplicate-named-group matcher uses the private, non-copyable
`DuplicateNamedGroupPattern::{AlternativeCaptures, IteratedBackreference}`
domain instead of an `iterated` Boolean. Batch AC moved that domain and the
pattern-parameterized emitter into one private child; the parent now exposes
only the alternative-captures and iterated-backreference semantic calls. One
borrowed exhaustive match owns the unchanged `abc`/`ad` and `aac`
candidate/result tables. The shared `has_indices` flow and initial no-match
`null` remain outside that selection. At the Batch AC shared checkpoint,
`cargo xc` is green, its bounded structure target passes `3/3`, the exact CLI
witness passes `1/1`, and the exact String match ordinary-groups and
indices-groups leaves pass all `4/4` variants with every failure bucket at
zero. The semantic golden was not rerun. It does not complete String, RegExp,
T18 or T19.

The literal replacement scope and raw loop now have one private
`builtins/string/string_literal_replacement_scope.rs` owner. The parent can
request only the first-occurrence or all-occurrences semantic operation and
cannot name, construct, import or project the private, non-copyable
`StringLiteralReplacementScope::{FirstOccurrence, AllOccurrences}` domain.
The moved four-line domain and 440-line raw emitter retain SHA-256
`db2e26fd031d6c5ab6f0ce99ab16f58928a202f29e9bee436069cb9368b882ba`
and `f6a34782a74376adfe9f7b622241e986c8c2bdace5f2fac4967ff4f07cf5170e`.
One borrowed exhaustive match still owns the unchanged first-match `Br(2)` exit
versus all-matches scan continuation without reordering emitted instructions.
Batch AJ shared `cargo xc` passes; the literal-scope, symbol-hook and RegExp
flag-getter structure targets pass `10/10`, the exact symbol-hook CLI fixture
passes `1/1`, and the first-only and all-occurrences pinned leaves pass all
`4/4` sloppy/strict Wasm-AOT executions with every failure bucket at zero. No
semantic golden was needed or run. The source-equivalent owner move adds no
fixture or conformance claim and does not complete String, RegExp, T18 or T19.

The shared `match`, `matchAll`, `replace`, `replaceAll` and `search` symbol-hook
emitter now accepts the sibling-visible, non-copyable
`StringSymbolHookOperation` domain instead of a broad `StandardBuiltinId` and
a projected second-argument Boolean. Six borrowed exhaustive matches own its
symbol key, argument, global-validation and `matchAll` policies; the private
fallback uses a seventh exhaustive match to select the five unchanged
algorithms. Standard dispatch names five exact producers, while `split` now
routes directly to its already separate emitter and cannot become an
impossible domain row. The focused
[contract](../docs/rust-rewrite/contracts/string-symbol-hook-operation.md) and
recursive two-source guard record this source-equivalent boundary. It adds no
fixture or broader String/RegExp conformance claim. The structure target passes
`4/4`, the adjacent literal-replacement guard passes `3/3`, and the existing
symbol-hook CLI fixture passes `1/1`. One exact pinned leaf per operation plus
the direct `split` path passes both variants (`12/12`) with every failure bucket
at zero; `cargo xc` is green.

Batch AY makes `StringSymbolHookOperation` and its raw emitter private to
`builtins/string.rs`. Standard dispatch reaches them only through five fixed String symbol-hook entries.
The frozen 306-line domain/emitter selection has SHA-256
`06636af9cd91f1e237e7cb08d47132941a9976c712a818073d1c208ce1271c26`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The symbol-hook, literal-replacement and RegExp
result-mode structure targets pass `5/5`, `3/3` and `3/3`; the complete
symbol-hook Wasm-AOT CLI fixture passes `1/1`. No Test262 leaf or Wasm golden
was required for this source-equivalent boundary, which claims no new String behavior,
conformance result or published-count change.

RegExp substitution recognition and handling now share the private,
non-copyable `RegExpSubstitutionKind::{LiteralDollar, MatchedSubstring, Prefix,
Suffix, NumberedCapture, NamedCapture}` authority. Its exact-order `ALL` table
and borrowed exhaustive runtime-code projection own stable codes 1 through 6;
the spelling table and numbered/named recognizers store those named codes, and
the handler exhaustively emits the existing six semantics. Raw zero remains
only the no-recognized-substitution sentinel, while consumed widths and source
updates remain unchanged. This source-equivalent invariant adds no fixture. Its
bounded structure target passes all `4/4` tests, and six exact pinned leaves,
one per substitution kind, pass both variants (`12/12`) with every failure
bucket at zero. Workspace formatting and the diff check are green. It does not
complete String, RegExp, T18 or T19.

The well-formed String builtin seam now accepts the private, capability-free
`StringWellFormedOperation::{Check, Repair}` domain instead of inspecting the
broad standard-builtin identity beside an independently selected result tag.
The two named entry points are its only producers. One consuming exhaustive
match owns both the unchanged well-formedness-check versus surrogate-repair
algorithm and the Boolean versus String result tag, so callers cannot create a
crossed algorithm/result-shape state. Standard dispatch can neither supply the
mode nor retag the resulting payload. This source-equivalent invariant adds no
runtime mode word or fixture; its bounded structure witness and focused source
checks are dry-written. It does not complete Unicode well-formedness, the
String API, the pinned String tree or T18.

Created-Realm bootstrap now publishes Array, String, Map and Set iterator
`next` functions through four unit targets, one private publication context and
a non-`Copy`, one-shot token. Bootstrap constructs the context once from its
Realm-function authority, TypeError prototype local and exact four iterator
prototype locals. Materialization derives every raw input from that borrowed
context, records the function's own payload and the TypeError prototype, then
returns a token that owns the selected prototype and function locals.
Publication consumes the token and writes its Function tag local before
defining `next`. This makes a borrowed created-Realm String iterator `next`
allocate result objects from that Realm and create incompatible receiver errors
with that Realm's `%TypeError%`. The exact receiver message and String stepping
algorithm remain unchanged.

The context couples and localizes raw bootstrap locals; Rust cannot
independently prove those `u32` local indices all came from one Realm. The sole
constructor call remains the explicit trust boundary. The strengthened bounded
structure target passes `5/5`, the focused cross-Realm String fixture passes
`1/1`, the created-Realm materialization inventory passes `1/1`, the retained
collection receiver target passes `2/2`, and the Map/Set and Array iterator
controls pass `1/1` each. `cargo check -p lila-aot-wasm` is green. The
[contract](../docs/rust-rewrite/contracts/created-realm-iterator-next-publication.md)
records the invariant and evidence. `ArrayIteratorIdentity` remains outside
the typed `next` lane, and the existing Map/Set receiver fixture is only a
control. This does not complete created-Realm intrinsic publication, the
pinned String iterator tree or T18.

## Objective

Implement ECMAScript strings as sequences of UTF-16 code units, including lone surrogates, while retaining an efficient Wasm representation. Complete String primitives, wrapper exotics, iterators and all pinned String APIs without ASCII-only assumptions or exact-test materializations.

## String representation contract

- Document whether heap strings use UTF-16, WTF-8, ropes/slices or a hybrid representation.
- Expose authoritative operations for code-unit length, code-unit indexing, code-point decoding, slicing, concatenation, comparison, hashing and flattening.
- Preserve lone high/low surrogates exactly through construction, concatenation, slicing, property access and serialization paths that require them.
- Keep byte offsets private to the representation; ECMAScript-visible indexes and `length` are always UTF-16 code units.
- Add overflow checks and avoid repeated quadratic transcoding for loops over long strings.
- Ensure GC tracing, interning and deduplication do not merge observably distinct wrapper/object identities.

## Primitive and String exotic behavior

Implement:

- `String` call/construct semantics, including Symbol handling and boxed-string internal data;
- string wrapper indexed own properties, non-writable/non-configurable descriptors, `length`, own-key ordering and deletion/definition restrictions;
- primitive property lookup/boxing through the shared object model;
- `%StringIteratorPrototype%` and code-point iteration;
- cross-realm wrappers, prototypes and error objects.

## Constructor and static methods

Complete all APIs present in the pinned suite, including:

- `String.fromCharCode`, `String.fromCodePoint` and `String.raw`;
- constructor/prototype metadata, descriptors and species-independent behavior;
- well-known symbol interactions and non-constructable method behavior.

## Prototype method families

Implement from shared conversion/string primitives rather than method-local byte logic:

- access/extraction: `at`, `charAt`, `charCodeAt`, `codePointAt`, `slice`, `substring`, Annex B `substr` where required;
- search/prefix: `includes`, `indexOf`, `lastIndexOf`, `startsWith`, `endsWith`;
- construction: `concat`, `repeat`, `padStart`, `padEnd`;
- whitespace/case/normalization: `trim`, `trimStart`, `trimEnd`, `toLowerCase`, `toUpperCase`, locale-sensitive variants through T23, and `normalize`;
- Unicode well-formedness: `isWellFormed`, `toWellFormed`;
- RegExp-integrated methods: `match`, `matchAll`, `search`, `replace`, `replaceAll`, `split` through the symbol-dispatch protocol from T19;
- `localeCompare` through T23;
- `toString`, `valueOf`, iterator and pinned Annex B HTML wrapper methods through T24.

## Unicode data and algorithms

- Pin Unicode data versions used for case conversion and normalization.
- Implement full default case mappings, including expansions and context-sensitive mappings required by ECMAScript.
- Implement NFC/NFD/NFKC/NFKD normalization over arbitrary strings and lone surrogates.
- Distinguish code-unit, code-point and grapheme operations; core String APIs must not accidentally use grapheme boundaries.
- Keep locale-sensitive casing and collation behind Intl interfaces, not host process locale calls.

## Observable-order requirements

- Reject RegExp inputs for methods such as `startsWith` only after the specified `IsRegExp` and coercion steps.
- Preserve receiver, argument and replacement-function coercion order.
- Propagate getters, proxies, `Symbol.match`, `Symbol.replace`, `Symbol.split` and thrown conversion values through shared operations.
- Do not constant-fold strings when doing so would suppress observable coercion or property lookup.

## Acceptance criteria

- The complete pinned `built-ins/String` and String iterator trees are green.
- All visible lengths/indexes are correct for BMP, astral characters, combining sequences and lone surrogates.
- No String Test262 path/source-specific materialization remains for covered semantics.
- String wrapper descriptors and own-key ordering pass through the general exotic-object protocol.
- Case conversion and normalization pass exhaustive data-driven tests for the pinned Unicode version.
- RegExp-symbol delegation works with custom objects and proxies.
- Long-string benchmarks avoid accidental quadratic behavior.

## Required tests

```sh
cargo test -p lila-ir string_ --quiet
cargo test -p lila-aot-wasm string_ --quiet
cargo test -p lila-cli wasm_string --quiet
./target/debug/lila test262 run built-ins/String --execution-backend wasm --timeout-ms 180000 --threads 8
```

Add focused representation tests for every surrogate boundary and rerun JSON, RegExp, URI, Date and Intl-adjacent filters that consume strings.
