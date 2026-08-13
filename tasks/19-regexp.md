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

Named-group identifier classification now uses a closed start/continue domain
and the pinned ICU `ID_Start`/`ID_Continue` tables directly. The RegExp parser
no longer asks the third-party regex dependency to decide that product grammar
rule. That dependency remains in a separate shape-limited static generator fold
whose accepted results can influence emitted IR; it must be proven against the
Lila engine or removed.

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
