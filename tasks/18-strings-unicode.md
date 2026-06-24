# T18 — Strings, Unicode and the complete String API

**Status:** Blocked on T04/T05/T10  
**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10; regular-expression methods integrate with T19  
**Blocks:** T19, T22, T23 and String-related T26 closure

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
cargo test -p porffor-ir string_ --quiet
cargo test -p porffor-aot-wasm string_ --quiet
cargo test -p porffor-cli wasm_string --quiet
./target/debug/porf test262 run built-ins/String --execution-backend wasm --timeout-ms 180000 --threads 8
```

Add focused representation tests for every surrogate boundary and rerun JSON, RegExp, URI, Date and Intl-adjacent filters that consume strings.