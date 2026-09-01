# BigInt prototype result policy

Status: normative for the Wasm AOT BigInt prototype result seam.

## Semantic boundary

`BigInt.prototype.valueOf`, `BigInt.prototype.toString`, and
`BigInt.prototype.toLocaleString` share `ThisBigIntValue` receiver validation,
including primitive, heap and boxed BigInt representations. They do not share
an argument or result policy:

- `valueOf` returns the extracted BigInt payload and tag unchanged;
- `toString` reads its optional radix, performs the required numeric coercion,
  rejects values outside 2 through 36, and formats in that radix; and
- the current core-language `toLocaleString` fallback formats the extracted
  BigInt in decimal and does not use either reserved argument position.

[ECMA-262 section 21.2.3.2](https://tc39.es/ecma262/#sec-bigint.prototype.tolocalestring)
permits an implementation without ECMA-402 to return the same result as the
parameterless `toString` operation, while requiring the two reserved parameter
positions not to be repurposed. Consequently, treating `locales` as a radix is
not an implementation-defined locale choice: it is a different operation and
can turn a valid locale string into a spurious `RangeError`.

[ECMA-402 section 20.3.1](https://tc39.es/ecma402/#sup-bigint.prototype.tolocalestring)
supersedes that fallback when the Internationalization API is present. Lila
does not yet have the `Intl.NumberFormat` integration owned by T23, so this
contract closes only the honest ECMA-262 fallback. It does not describe
locale-aware grouping, numbering systems, option processing or their abrupt
completions.

## Closed protocol

`BigIntBuiltin` carries prototype calls as one
`BigIntPrototypeResultPolicy` value with exactly three variants:

- `ExactValue(BigIntValueResult)`;
- `RadixString(BigIntRadixStringResult)`; and
- `LocaleStringFallback(BigIntLocaleStringFallbackResult)`.

The existing builtin producer names are associated constants that construct
the corresponding typed variant, so their callers remain narrow while the
semantic distinction is present at the emitter boundary. The emitter performs
receiver extraction once, then exhaustively matches the result policy without
a catch-all. Each arm calls a helper that accepts only its matching marker
type. A locale marker therefore cannot be passed to the radix reader, and a
new policy cannot omit a result arm without a Rust compile error. A focused
unit test pins the three public producer names to their policy projections.

Each variant carries a move-only result authority. Neither the policy nor its
three marker types implements `Clone` or `Copy`, and the consuming match moves
the selected marker into exactly one matching helper. In particular, the
radix marker moves onward into the sole preparation stage, so radix coercion
cannot be duplicated by retaining or copying that authority for a second
preparation call. The focused
`bigint_prototype_result_ownership_structure` guard uses a Rust-lexical
recursive census, pins the attribute-free declarations and exact producers,
and requires these one-way handoffs.

The radix path retains its required ordering: validate `this`, then read and
coerce radix, propagate an abrupt completion unchanged, range-check, and only
then inspect the receiver representation for formatting. One shared
realm-aware preparation stage returns a private `PreparedBigIntRadixLocal`.
The raw carrier, its constructor and projections, the preparation stage, both
representation formatters and the consuming release share the private
`bigint/radix_formatting.rs` owner. The parent can call only the sibling-visible
semantic result wrapper and cannot name, import, construct or project the raw
prepared local. Both immediate and heap formatters consume that same prepared
local. The stage
creates BigInt/Symbol conversion `TypeError`s and out-of-range `RangeError`s
from the calling builtin's defining/current-function Realm. The outlined
`ToNumber` helper receives a standard builtin's self-backed Realm environment
explicitly, while non-standard callers pass zero rather than expose an
incompatible lexical-environment layout. `FunctionBuilder` carries one closed
`NumericErrorRealmSource` body-domain value: main, user, host and ordinary
helper bodies select `GlobalFallback`; standard builtins select
`StandardBuiltinEnvironment`; and the exhaustive `begin_helper_body`
transition selects `NumericConversionHelperArgument` only for `ValueToNumber`
and `ValueToNumeric`. No `function_id == None` guess or mutable outline flag can
make a main lexical environment trusted Realm storage. One closed ABI
projection emits parameter 6 for both outlined `ToNumber` and `ToNumeric`, and
a unit witness pins the two trusted helpers plus all three source projections.

Created-Realm BigInt prototype method objects store both that Realm's
`TypeError.prototype` and `RangeError.prototype` in their function Realm slots.
BigInt/Symbol radix conversion errors and radix range failures therefore share
the same defining-Realm policy for immediate and heap receivers. An object
whose primitive conversion produces BigInt or Symbol cannot make a
foreign-Realm BigInt builtin fall back to the main-Realm constructors.
Representation-specific formatting cannot rerun, skip or replace the
coercion/range stage.

The locale fallback validates `this` but does not read, coerce or access any
property of `locales` or `options`. The exact-value path preserves the
extracted representation rather than converting it.

## Durable witness

`wasm_bigint_prototype_result_policy.js` covers the product path for:

- immediate and heap BigInt radix formatting;
- valid `en-US` locale calls that previously entered radix coercion, including
  an ungrouped heap-sized value whose fallback and future ECMA-402 spelling
  agree;
- throwing Proxy hooks in both reserved locale/options positions, proving that
  the core fallback observes neither argument;
- primitive and boxed exact `valueOf` results;
- invalid receiver rejection;
- radix abrupt-completion identity and range rejection; and
- a captured top-level `const Symbol` postfix update: immutable-binding update
  lowering first creates `SpecOperationIr::ToNumeric`, so this reaches
  `compile_expr_to_numeric_locals` and proves a main lexical environment is
  never interpreted as Realm prototype storage; and
- foreign-Realm immediate and heap radix failures, including BigInt, Symbol
  and object-to-BigInt conversion, whose errors use the builtin's Realm.

The ordinary locale text cases deliberately agree with likely future ECMA-402
output. The throwing reserved-argument hooks specifically pin the current
ECMA-262 fallback, however: T23 must replace those assertions when it replaces
the fallback helper with conforming locale/options processing.

## Nonclaims and deferred gates

This seam does not implement or claim `Intl.NumberFormat`, locale-aware BigInt
formatting, the ECMA-402 BigInt tree, `Number.prototype.toLocaleString`, full
BigInt formatting correctness, numeric closure, or changed conformance counts.

Static freeze gates are `rustfmt --check` for the touched Rust files,
`node --check` for the fixture, the focused result-ownership structure target,
focused source searches, `git diff --check`, and manual local-lifetime review.

## Batch AX dispatcher boundary

The builtin, fixed-width, prototype-result and three result-authority domains,
their associated prototype producers and the raw exhaustive emitter are now
private to the BigInt family. Standard dispatch reaches them only through six fixed BigInt entries.
The frozen 736-line domain/emitter selection has SHA-256
`5b61c6cfedaf3b988517eab492bb6c3dedb85a5eb9ac98992120ff39e7f30f18`;
restoring only the former visibilities reproduces that source exactly. Batch AX
`cargo xc` passes. The fixed-width, heap-slot, helper-operation, number-policy
and prototype-result structure targets pass `5/5`, `4/4`, `4/4`, `2/2` and
`4/4`. The constructor, arbitrary-width and wrapper-coercion Wasm-AOT CLI
controls pass `3/3`; a direct stdin control exercises `toString`,
`toLocaleString` and `valueOf` together and returns `number(3)`. The broader
prototype fixture remains red at its unrelated captured-main-lexical Symbol
Realm assertion. No Test262 leaf or Wasm golden was required for this
source-equivalent boundary, which claims no new BigInt behavior, conformance
result or published-count change.
`scripts/check-module-boundaries.sh` also pins the
closed BigInt result protocol, the child-private prepared-radix lifecycle and
sole semantic wrapper, the three-state builder Realm source, the two helper
ABI consumers, `new_main`'s exact `GlobalFallback` projection, and the
created-Realm BigInt TypeError/RangeError slots. The main-script witness does
not rely on a particular lexical-environment byte layout or poison value.
The result-ownership structure target passes `4/4`, the neighboring fixed-width
operation structure target passes `5/5`, and the exact producer unit witness
passes `1/1`. At the 2026-08-28 Batch V checkpoint, the ownership structure
target remained green at `4/4`, the shared `cargo xc` gate passed, and the
unchanged pinned `radix-2-to-36.js` leaf passed both ordinary Wasm-AOT variants
`2/2`. The existing product fixture was also executed, but the current shared
workspace fails it `0/1` before its radix assertions, with `main lexical Symbol
ToNumeric realm fallback`; that conversion/Realm path is outside this
source-equivalent extraction and is not reported as green. The full BigInt
prototype trees, ECMA-402 BigInt tree and broad batch ladder remain deferred.
