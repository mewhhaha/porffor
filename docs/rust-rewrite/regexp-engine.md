# RegExp engine architecture: ordered bytecode in emitted Wasm

## Current candidate verification

The seven new operand-fold IR controls and nine paired direct Engine fixtures
pass. The selected ten-file/20-mode adjacent RegExp replay passes; it includes
two circled-M property files and the Unicode case-mapping neighbor. The
33-file/66-mode /v inventory remains separate from this selected replay and
from direct /iv evidence. Emitted runtime-pattern compilation remains open.

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

The static serialization boundary is now `ValidatedRegExpProgram` in
`crates/lila-ir/src/regexp/program.rs`. It rejects invalid instructions, pool
references, named-capture maps and non-consuming cycles before exposing owned
bytes. A versioned relative descriptor owns its instruction, range, candidate
and UTF-8 name sections. One packed allocation handle replaces the six
independent RegExp header slots, and the emitted matcher derives its bounds
from the descriptor. Clone, compile, split and matchAll paths share the
immutable allocation while preserving separate observable object state. See
the [program-boundary contract](contracts/regexp-program-boundary.md) for the
wire format, ownership inventory and corruption controls.

This is a foundation, not the complete design:

- computed patterns reach an iterative RegExp-only parser/compiler in emitted
  Wasm after a static-cache miss. Its current unverified source adds validated
  Unicode character modes, named groups/references, lookbehind, braced escapes
  and exact code-point property escapes to the ordinary grammar. Runtime
  finite-string properties and nested code-point/string algebra now share the
  complete catalog and existing matcher instructions in unverified source; see the [runtime compiler contract](contracts/regexp-runtime-compiler.md)
  and [computed property contract](contracts/runtime-regexp-codepoint-property.md);
- the static parser and program lowerer still recurse on the Rust stack;
  the emitted runtime compiler uses bounded arena and task stacks;
- static compilation retains `UnsupportedFeature` for compiler limitations,
  including expanded instruction/range limits; these do not prove invalid syntax.
  Computed finite strings no longer select an unsupported helper outcome;
- expanded programs are capped at 32768 instructions and range pools at 65536
  entries, but those limits are not yet one typed resource policy;
- the matcher has checked scratch-address calculations and one closed status
  ABI: normal completion, corrupt-program failure and scratch-resource
  exhaustion are distinct, every result writer takes that domain, and the
  wrapper routes the two failures only after rewinding transient storage;
  however, there is still no deterministic execution-step budget; and
- the former third-party generator membership fold has been removed. The
  remaining vendored `regress` dependency supplies pinned Unicode property,
  string-sequence and fold data; product parsing and matching consume Lila's
  own program representation. Third-party matching cannot decide product
  RegExp semantics.

The pattern parser no longer asks `regress` to classify named-group identifier
characters. A closed start/continue domain now selects the pinned ICU
`ID_Start` or `ID_Continue` property directly. This is the first code invariant
landed from the architecture below.

Ordinary character classes now select a closed `Legacy` or `Unicode` grammar
mode before choosing their bitmap or range instruction representation. Every
ordinary-class representation is entered through that mode, so an ASCII fast
path cannot admit Annex B control/octal or unrestricted identity escapes under
`u`, and accepted escapes such as `\cA` have the same value in either
representation. An incomplete legacy `\c` likewise preserves Annex B's
standalone-backslash atom boundary through either encoder. See the
focused [Unicode ordinary-class escape contract](contracts/regexp-unicode-class-escape-grammar.md).
This does not complete the separate UnicodeSets class grammar. The current
unverified runtime compiler retains a validated character mode through class
parsing, complement and folding. Its computed code-point properties share the
complete exact native alias catalog. The emitted `v` grammar now handles nested
code-point algebra through operand-local closure and checked lazy depth bitmaps.
The finite-string extension adds complete q/property keys, empty members and
operand-local folding to that same algebra and checked checkpoint. Complete
pattern/capture validation precedes lowering and publication; see the
[computed set-algebra contract](contracts/runtime-regexp-computed-unicode-sets.md).

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
§22.2.1.8 `MayContainStrings` negation early error are checked. Only a globally
valid Pattern may turn that value into the exact finite matcher atom described
by the
[finite-string algebra contract](contracts/regexp-unicode-set-finite-string-algebra.md).
The same finite domain now owns all seven pinned Unicode 17 string properties.
The prepared `/iv` successor folds direct and finite-property operands before
set algebra and lowers folded positions through existing range instructions.
`Basic_Emoji` and `RGI_Emoji` require the same folding because their circled-M
sequence has a simple-case alias. Its seven new IR controls and nine paired
Engine fixtures are unexecuted. The later computed finite-string source batch
uses the same property catalog and matching priority in the emitted compiler,
with three additional paired semantic sources; broad conformance remains open. See also the focused
[class-expression shape contract](contracts/regexp-unicode-set-expression-shape.md).

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

The current 24-byte instruction representation remains behind the validated
serialization boundary. Its raw opcode and operand constructors still require
a future closed `RegExpOp` domain; this descriptor migration does not replace
the parser or instruction builder. A versioned program header now carries
instruction, capture, range and named-group section extents. Only validation
can construct `ValidatedRegExpProgram`, and the backend stores only its private
immutable bytes. The matcher accepts one allocation handle, validates the
header against that allocation, and never accepts independent object-owned
pointer/count pairs or a global heap limit as section authority.

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
  properties come from one pinned Unicode data identity. The RegExp projection
  combines the pinned ICU4X 2.0 Unicode 16 data with committed Unicode 17
  additions and fold/string tables. Changes to that authority are a coordinated
  T18/T19/T23 conformance event and invalidate compiled-program caches; this
  projection does not imply an upgrade of the Intl provider.
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
