# T19 — Complete ECMAScript RegExp semantics

**Status:** Blocked on T04/T05/T10/T18  
**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10, T18  
**Blocks:** String-RegExp integration and RegExp-related T26 closure

## Objective

Implement the ECMAScript regular-expression grammar, matching model and observable object protocol for every feature in the pinned suite. Treat the current Rust regex dependency as an implementation component only where its behavior exactly matches ECMAScript; do not expose host-regex semantics as JavaScript semantics.

## Engine strategy

Write a short design document before broad implementation that evaluates:

- translating ECMAScript patterns into a compatible Rust engine plus Porffor-managed semantics;
- extending/forking the current engine for missing features;
- implementing a dedicated bytecode/NFA/backtracking engine compiled into Wasm;
- hybrid specialized engines selected by pattern features.

The chosen design must support lone-surrogate-aware UTF-16 matching, observable `lastIndex`, captures and all pinned syntax. It must not invoke a host JavaScript engine, bundle a JavaScript interpreter, or recognize known Test262 patterns specially.

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
cargo test -p porffor-ir regexp_ --quiet
cargo test -p porffor-aot-wasm regexp_ --quiet
cargo test -p porffor-cli wasm_regexp --quiet
./target/debug/porf test262 run built-ins/RegExp --execution-backend wasm --timeout-ms 180000 --threads 4
```

Also run RegExp literal grammar tests and the String `match`, `matchAll`, `search`, `replace`, `replaceAll` and `split` subtrees.