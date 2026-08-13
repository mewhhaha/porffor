# RegExp engine architecture: ordered bytecode in emitted Wasm

## Status and boundary

This document is the source of truth for T19's engine choice. It selects the
target architecture; it does not claim that the current RegExp grammar or
object protocol is complete.

Lila will use one Rust-owned ECMAScript RegExp grammar and one dedicated,
ordered-backtracking bytecode model. Static patterns are compiled while the
JavaScript program is compiled. Patterns constructed from runtime strings are
compiled by a RegExp-only compiler in the emitted Wasm and execute in the same
Wasm matcher. Neither path invokes a host JavaScript engine or a host regular-
expression matcher.

A RegExp compiler and matcher are language runtime components, not a JavaScript
interpreter: they accept UTF-16 pattern/input records and closed flags, and
cannot parse or execute JavaScript source.

## Current ground truth

`crates/lila-ir/src/regexp.rs` already parses a useful subset into
`RegExpProgram`, a fixed-width instruction stream plus range and named-group
metadata. `crates/lila-aot-wasm/src/builtins/regexp.rs` emits an iterative Wasm
matcher with ordered choice frames, UTF-16-visible cursors and capture
restoration. Literal and statically resolved constructor calls can attach an
immutable program to a RegExp object.

This is a foundation, not the complete design:

- computed patterns are served by a finite table of strings found at compile
  time and a separate small-pattern fallback; arbitrary runtime patterns do not
  yet reach the program compiler;
- the parser and program lowerer recurse on the Rust stack for nested groups;
- legal constructs such as general lookahead, `v`-mode string properties and
  several nullable or astral forms outside the direct legacy term seam still
  return `UnsupportedFeature`;
- expanded programs are capped at 4096 instructions and range pools at 65536
  entries, but those limits are not yet one typed resource policy;
- the matcher has checked scratch-address calculations and one closed status
  ABI: normal completion, corrupt-program failure and scratch-resource
  exhaustion are distinct, every result writer takes that domain, and the
  wrapper routes the two failures only after rewinding transient storage;
  however, there is still no deterministic execution-step budget; and
- the `regress` crate still decides membership inside one shape-limited static
  generator-fold optimization, so an accepted result can influence emitted IR.
  That fold must eventually prove equivalence with the Lila engine or decline;
  a third-party engine cannot decide product RegExp semantics.

The pattern parser no longer asks `regress` to classify named-group identifier
characters. A closed start/continue domain now selects the pinned ICU
`ID_Start` or `ID_Continue` property directly. This is the first code invariant
landed from the architecture below.

Legacy direct astral source now has its own closed parsed-term case. It stores a
validated UTF-16 surrogate pair, emits the lead once, and applies any following
quantifier only to the trail, as required by the non-Unicode grammar's code-unit
atom boundary. See the focused
[legacy direct-astral quantifier contract](contracts/regexp-legacy-direct-astral-quantifier.md).

UnicodeSets class expressions now choose one closed grammar shape after their
first typed operand: union, a homogeneous intersection chain, or a homogeneous
subtraction chain. The private operator domain owns both delimiters and range
semantics, so mixed operators, implicit unions inside operation operands and
missing operands are syntax errors rather than silently compiled range sets.
A private validated `ClassSetCharacter` boundary also rejects raw set-syntax
characters, reserved double punctuators, and `\0` followed by a decimal digit
while preserving escaped operands. A validated `\q{…}` remains typed while the
entire enclosing expression, closing bracket, range rules, and exact
§22.2.1.8 `MayContainStrings` negation early error are checked. Its typed marker
then survives the complete Pattern parse, including group, named-reference,
and nullable-group unbounded-quantifier checks; only a globally valid Pattern
records the still-unimplemented string semantics as a capability gap.
See the focused
[class-expression shape contract](contracts/regexp-unicode-set-expression-shape.md).
Class-string disjunctions and properties of strings remain capability gaps.

## Decision and rejected alternatives

The selected engine is an extension of the current dedicated bytecode path.

| Approach | Decision | Reason |
| --- | --- | --- |
| Translate into a Rust regex engine | Rejected as semantic authority | Rust strings cannot represent lone UTF-16 surrogates, host engines differ on captures, backreferences, sets and case folding, and computed patterns would require host matching rather than emitted-Wasm semantics. |
| Fork the current Rust dependency | Rejected as the product engine | It would still need a second implementation or a host dependency for runtime pattern compilation and would not remove the JavaScript object/wrapper work. Reviewed algorithms may be reused, but its API and data are not the contract. |
| Thompson NFA/DFA only | Rejected as the complete engine | Backreferences and ECMAScript's ordered captures/lookarounds are not a regular-language problem. A pure NFA cannot be the semantic fallback for all patterns. |
| Feature-selected hybrid engines | Deferred | A proven linear-time fast path may be added later, but introducing two semantic matchers before the reference bytecode engine is complete multiplies capture and Unicode drift. |
| Lila parser + ordered bytecode VM | Selected | It matches ECMAScript's leftmost, depth-first choice order, supports non-regular features, works for static and dynamic patterns, and can run without native recursion inside emitted Wasm. |

## Semantic layering

The object protocol and matcher remain separate:

```text
observable JS reads/calls/coercions
    -> validated PatternCodeUnits + RegExpFlags
    -> ValidatedRegExpProgram
    -> pure Wasm match over UTF-16 input
    -> captures as UTF-16 spans
    -> observable lastIndex writes and result construction
```

The outer builtins own `RegExpExec`, custom `exec` dispatch, species and
subclass construction, realm selection, `lastIndex`, result arrays, indices,
groups and String well-known-symbol methods. The matcher receives no JavaScript
object and performs no property access. It accepts an immutable program, input,
start position and a closed search mode (`Anchored` for sticky matching or
`LeftmostAtOrAfter` otherwise).

This boundary makes compiled-program caching unobservable. A cache key contains
the exact pattern code units, canonical flags, bytecode schema version, Unicode
data identity and resource-policy identity. It caches only immutable programs,
never RegExp objects, `lastIndex`, realms, species decisions or custom methods.
All observable coercions happen before lookup.

Static and runtime pattern compilers must consume the same generated grammar,
opcode and Unicode-property tables and produce the same validated program
format. The static compiler is still required for literal early errors. The
runtime compiler is required for arbitrary `new RegExp(value, flags)`; once it
exists, the finite candidate table and simple-pattern matcher are retired
rather than preserved as alternate semantics.

## Program and backtracking invariants

The current 24-byte instruction representation may remain during migration,
but its Rust construction becomes a closed `RegExpOp` domain with typed
operands and a private encoder. A versioned program header carries instruction,
capture, range, named-group and string-pool bounds. Only the compiler or a
validator can construct `ValidatedRegExpProgram`; the matcher never consumes an
unchecked pointer/count tuple.

The VM preserves these invariants:

1. `Split` pushes the later alternative and enters the earlier alternative.
   Greedy and lazy quantifiers differ only in that source-ordered branch choice.
2. The VM is iterative. Pattern parsing, program validation, matching,
   lookaround and rollback do not recurse on a Rust, host or Wasm call stack.
3. The live capture vector stores `Unmatched` or a half-open pair of UTF-16
   indexes. A capture mutation appends its prior value to an undo journal. Each
   choice frame stores the journal checkpoint, so rollback restores captures
   exactly without copying the full vector into every frame.
4. Assertion frames carry their input direction, return PC, choice depth and
   capture checkpoint. Positive assertions commit the captures required by
   ECMAScript; failed and negative assertions restore them.
5. Backreferences compare the captured UTF-16 code-unit sequence. An unmatched
   capture follows the ECMAScript empty-match rule; it is not represented by a
   magic zero span.
6. No memoization or NFA merge may discard capture, assertion or ordering state.
   A future fast path must prove that those states are observationally
   irrelevant for its admitted closed feature set.
7. Every opcode and pool reference is bounds checked by validation. An unknown
   opcode or invalid target is an internal corrupt-artifact fault, not a
   JavaScript no-match result.

## UTF-16 and Unicode invariants

`PatternCodeUnits`, `Utf16Index` and `InputCursor` are distinct types.
`PatternCodeUnits` is not a Rust `String`: computed patterns can contain lone
surrogates. `InputCursor` may cache a private byte position for the current
string representation, but all matching positions, captures, `lastIndex` and
reported indices are UTF-16 code-unit indexes.

- Legacy mode consumes one code unit and can match either half of a surrogate
  pair independently.
- `u` and `v` modes use `CodePointAt`/`AdvanceStringIndex` behavior, combining a
  valid surrogate pair but preserving a lone surrogate as its own value.
- `v` class strings consume an ordered sequence of code points; set operations
  are compiled from normalized immutable sets and string tries, not delegated
  to a host regex parser.
- Property names/aliases, `ID_Start`, `ID_Continue`, case closure and string
  properties come from one pinned Unicode data identity. The current lock's
  ICU4X 2.0 data line is Unicode 16.0.0; an upgrade is an atomic T18/T19/T23
  conformance event and invalidates compiled-program caches.
- Ignore-case matching implements ECMA-262 `Canonicalize` for the selected
  Unicode mode. General ICU lowercasing or full case folding is not a
  substitute, because it can expand strings or apply mappings ECMAScript does
  not use.

Source and flag errors carry code-unit offsets. Duplicate flags point at the
second occurrence. `u` and `v` remain mutually exclusive through the existing
closed `RegExpUnicodeMode`.

## Deterministic resource policy

Resource failure is a third answer, distinct from invalid syntax and a normal
no-match. During the migration, `UnsupportedFeature` remains a fourth explicit
compiler capability gap; completion deletes it rather than mapping it to
`SyntaxError`.

The target domains are equivalent to:

```rust
enum RegExpCompileOutcome {
    Program(ValidatedRegExpProgram),
    SyntaxError(RegExpSyntaxError),
    ResourceExhausted(RegExpResourceError),
}

enum RegExpMatchOutcome {
    Match(RegExpMatch),
    NoMatch,
    ResourceExhausted(RegExpResourceError),
}
```

`RegExpResourceLimits` is one validated, versioned record covering pattern code
units, nesting depth, captures, instructions, ranges/string-pool data, choice
frames, capture-journal entries, scratch bytes and match steps. Its values are
embedded in the artifact or its runtime policy and participate in cache
identity. Scattered constants and wall-clock timeouts are not resource policy.

Every opcode dispatch consumes a step. Data-dependent decoding, class-string
comparison and rollback loops also consume named units, so no control-flow
cycle is free. Checked arithmetic derives all arena sizes before allocation;
memory growth failure, an unaddressable arena or exhausted steps returns
`ResourceExhausted` without wrapping, trapping or corrupting captures.

A dynamic constructor or match resource failure becomes a catchable
`RangeError` with a stable message. It is never `SyntaxError`, `NoMatch`, a
timeout counted as a pass, or the current generic “matcher failed” error. A
static literal that exceeds compiler resources is a compiler resource error,
not a claim that its grammar is invalid. Harness/development profiles may use a
lower deterministic step budget to expose catastrophic patterns quickly; they
may not disable address and structural checks.

The wrapper performs no success/failure `lastIndex` write after a matcher
resource error. Reads and coercions already required before matching remain
observable in specification order.

### Landed matcher-status slice

The current helper ABI closes the first, deliberately bounded part of that
target. `RegExpMatcherStatus` has exactly `Complete` and
`Failed(RegExpMatcherFailure)`; the failure domain has exactly
`CorruptProgram` and `ResourceExhausted`. One macro row source owns each
failure's ABI word, error route and stable message, and derives both `ALL`
views. The helper result writer accepts the typed status rather than an `i64`,
so a new exit cannot invent or silently reuse a status word.

All 45 current result writers are classified: four complete, forty corrupt
program and one resource-exhausted. The resource row is the ordered-choice
capacity guard. Earlier metadata/arithmetic failures remain corrupt-program
outcomes because the wrapper has already validated and allocated the matching
arena; reaching one means the helper and its trusted caller disagree. The six
wrapper preflight exits use the same typed `ResourceExhausted` route rather
than spelling their own error identity.

The wrapper rewinds the scratch arena and any speculative result carrier
before examining the returned status. `CorruptProgram` preserves the existing
generic `Error`; `ResourceExhausted` becomes a catchable current-function-realm
`RangeError`. Either route returns before the global/sticky `lastIndex` write.
The two messages are interned by walking the failure domain, not repeated in a
parallel string list.

All helper writers are private and typed, so the emitted helper cannot produce
an unknown status word. The dynamic ABI consumer nevertheless treats every
nonzero word left after the known-failure comparisons as `CorruptProgram`.
This final guard is deliberate defense at the Wasm boundary: a future helper
generation or call-wiring defect cannot fall through as a normal result.

This slice does **not** implement `RegExpMatchOutcome`, a step counter, the
versioned `RegExpResourceLimits` record, iterative parsing, arbitrary runtime
pattern compilation or a product-only reachability hook. With the present
validated program limits and exactly sized arena, the resource status is not
expected to be reachable from a valid program; the deterministic step-policy
work is what must supply an honest end-to-end resource-exhaustion fixture.

## Completion sequence

1. Close and validate the opcode/program/status/resource domains. The matcher
   status words and current scratch-failure routing are closed; unchecked
   metadata tuples and the complete resource policy remain.
2. Make the Rust parser/lowerer iterative and accept UTF-16 pattern code units.
3. Complete grammar and bytecode semantics, including general assertions,
   UnicodeSets string members and all capture restoration.
4. Emit the runtime pattern compiler and retire the candidate-table and simple
   matcher paths.
5. Add the deterministic step policy, adversarial benchmarks and exact
   resource-error fixtures.
6. Remove `UnsupportedFeature` only after the complete pinned RegExp and String
   integration trees have no capability gaps, timeouts or materializations.

Cheap structural gates precede those suites: exhaustive matches over every
closed domain, bytecode encode/validate round trips, static/runtime compiler
differential corpora and a scan proving product matcher decisions no longer
call `regress`. Broad Test262 runs remain the final semantic gate, not the
architecture proof.
