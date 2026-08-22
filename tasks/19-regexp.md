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
between forward and reverse paths. Unicode properties of strings and direct
class-string `/iv` folding remain distinct typed capability boundaries; this
records no broader UnicodeSets or RegExp completion claim.

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
or full-suite claim is made. Class-string matching,
properties of strings, arbitrary runtime pattern compilation, broad
nullable-pattern closure and the complete RegExp/String trees remain outside
this batch.

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

This closes the raw status/current scratch-failure seam only. There is still no
deterministic execution-step budget or unified `RegExpResourceLimits`, and the
resource status is not expected to be reachable from a valid current program
under the exactly sized arena. No product hook or end-to-end exhaustion claim
is added ahead of that follow-up.

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
remains an explicit unsupported capability. This does not implement
class-string matching, properties of strings or full UnicodeSets conformance.

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
