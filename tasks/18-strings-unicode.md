# T18 — Strings, Unicode and the complete String API

**Status:** In progress — UTF-16-aware primitives exist; broad APIs still use materializations

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10; regular-expression methods integrate with T19  
**Blocks:** T19, T22, T23 and String-related T26 closure

## Current repository state

Heap strings and many String methods operate on UTF-16 code units, with focused
coverage for surrogates, case conversion and symbol hooks. The Test262 harness
still contains a large family of String method metadata, legacy and coercion
rewrites, while full Unicode normalization/case data and RegExp/Intl
integration are not proven complete. The complete current-pin String tree and
materialization-removal gate remain open.

The metadata cases for `String.prototype.at`, `charAt`, `charCodeAt`,
`codePointAt`, `includes`, `indexOf`, `lastIndexOf`, `startsWith`, `endsWith`,
`match`, `matchAll`, `search`, `repeat`, `padStart`, `padEnd`, `trim`,
`trimStart`, `trimEnd`, `toString`, `valueOf`, `isWellFormed` and
`toWellFormed` now run their pinned sources through the shared Wasm-AOT
`propertyHelper.js` and general builtin metadata path; their path-specific
rewrites have been removed. The exact shortcut inventory now assigns 18
remaining observations to T18. Those are legacy, helper-reduction, coercion,
dynamic-source, RegExp-integrated and array-exotic semantic rewrites rather
than the metadata leaves above.
The two non-`eval` Sputnik `charAt` receiver cases, the direct `charAt`
position-coercion/rounding cases and the plain Array `toString` conversion
matrix also run their pinned source now; existing product fixtures already
encode those same receiver, conversion and assertion contracts. The remaining
`charAt`/`charCodeAt`/`indexOf` legacy rewrites contain dynamic `eval` and stay
explicit until the T13 policy permits a static-source replacement or the
pinned cases are classified unsupported.

All thirteen non-dynamic Sputnik `String.prototype.slice` cases covered by the
legacy materializer (`S15.5.4.13_A1_T1`, `T2`, `T4`, `T6` through `T15`) now
run their pinned sources with only the shared `sta-preamble.js` definition
required by their `Test262Error` assertions. The only remaining slice rewrite
is `T5`, whose pinned body constructs source dynamically through `Function()`.

Nine non-`eval` Sputnik `String.prototype.match` cases (`S15.5.4.10_A2_T1`,
`T6` through `T11`, `T17` and `T18`) likewise run their pinned bodies rather
than materializations that cached a single match result. The remaining
`S15.5.4.10_A1_T3` rewrite substitutes its dynamic `eval` input and stays
explicit until T13 permits the pinned source or classifies it unsupported. The
source and harness-preservation contract for these fifteen newly restored
leaves is dry-written; its focused Rust, CLI and exact Test262 execution gates
remain queued behind the active current-pin matrix.

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
created Realm's borrowed repeat method. The code, structural guard, fixture
and normative contract are dry-written; Cargo, CLI and focused pinned repeat
execution remain queued behind the active current-pin matrix. This does not
change the maximum String size, general numeric conversion or the published
repeat count.

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
