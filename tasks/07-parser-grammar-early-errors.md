# T07 — Parser boundary, grammar coverage and early errors

**Status:** In progress — parse-once boundary and duplicate-formal-parameter classification implemented; grammar and early-error closure remain

**Parallel group:** Core foundations  
**Depends on:** T01, T02  
**Blocks:** T08, T09, T12, T24 and parser-failure closure

## Current repository state

The front end now returns a closed `ParsedSource` with goal-typed
`ParsedScript` and `ParsedModule` variants. Each variant owns Boa's AST and the
exact interner that produced it, while `lila-ir` can only borrow the pair
through a controlled compiler session. Raw `SourceUnit` metadata is a distinct
type that the lowerer does not accept.

Loaded modules retain either that parsed product or the structured parse
rejection in `ModuleSourceIr`; dependency discovery and module-record
construction therefore share one parse attempt. The engine similarly retains a
`PreparedCompilation` so cache-key graph hashing and lowering consume the same
graph. Linked module text is a new, generated Script compilation unit and is
parsed once before ordinary Script lowering. The old reparsing functions and
the public reparsing stage have been removed.

A Script that contains `import()` keeps its Script-goal parse: request discovery
walks that retained AST and synthesizes only the graph record the linker needs.
It is not reparsed under Module grammar, so sloppy Script syntax and top-level
semantics cannot drift merely because the Script performs a dynamic import.

This closes the architectural double-parse defect, not T07 as a whole.
Current-pin parser and early-error buckets still lack a complete verified
Wasm-AOT aggregate, and the remaining grammar/diagnostic cases below still need
inventory-driven closure.

Duplicate formal parameters now have one closed diagnostic condition across
entry and retained dependency parsing. The classifier follows pinned Boa's two
exact, case-sensitive wordings and preserves the spec exception for sloppy
ordinary functions with simple parameter lists. This closes that bounded
misclassification only; it does not claim the remaining formal-parameter early
errors or the current-pin parser bucket are complete. The focused Cargo and
Test262 verification is deferred to the shared verification lane.

## Objective

Make parsing and static-semantics classification complete, deterministic and source-located for the pinned ECMAScript grammar. Keep the parse-once ownership boundary intact while closing the remaining pinned-suite failures.

## Architecture

- `ParsedScript` and `ParsedModule` own the Boa AST/interner pair; access stays inside their non-escaping compiler-session callbacks.
- Parse exactly once per compilation unit and retain failed module attempts as structured rejections rather than retrying them.
- Preserve script vs module goal, filename, spans, strictness and source text in the parsed product.
- Convert parser panics into structured diagnostics without hiding compiler bugs. Known unsupported parser constructs must be distinguishable from malformed JavaScript.
- Keep Boa as an implementation dependency, not the public IR contract, so it can be upgraded or replaced deliberately.

## Grammar coverage

Drive parser work from T01's failure inventory and Test262 feature metadata. Include:

- scripts, modules, hashbang, directives and strict-mode transitions;
- all declaration/expression/statement forms in the pin;
- classes, private names, static blocks and current standardized syntax;
- async/generator syntax, `yield`, `await` and contextual-keyword restrictions;
- import/export forms, import attributes and dynamic import syntax present in the pin;
- optional chaining, nullish operators, logical assignment, numeric separators, BigInt and regexp literals;
- Annex B grammar extensions where enabled.

## Early errors

Implement explicit static-semantics checks with correct phase and error type, including:

- duplicate lexical/private/export names;
- binding-name restrictions and strict reserved words;
- invalid `break`/`continue` targets;
- illegal `return`, `super`, `new.target`, `yield` and `await` contexts;
- duplicate parameters under the correct strict/simple-list rules;
- class constructor/private-element restrictions;
- module import/export conflicts;
- destructuring and assignment-target validity;
- `__proto__` duplicate literal restrictions;
- Annex B exceptions.

Do not treat runtime errors as acceptable substitutes for parse/early errors.

## Diagnostics

Add stable diagnostic codes, phase (`parse` or `early`), error constructor and source span. Test262 negative tests should compare phase/type through structured data, not string fragments.

## Acceptance criteria

- Compilation parses each unit once.
- Parser panic cases become deterministic compiler failures and have minimized regression tests.
- All negative parse/early cases are classified at the required phase.
- Script/module goal differences are covered.
- Upgrading Boa does not require feature modules to depend directly on Boa AST internals.
- The parser/early-error buckets from T01 reach zero for the pinned suite, excluding only explicitly documented upstream parser defects with a vendored fix task.

## Required tests

```sh
cargo test -p lila-front --quiet
cargo test -p lila-ir early_error --quiet
cargo test -p lila-engine --quiet
./target/debug/lila test262 run language --execution-backend wasm
```

During development run focused `language/expressions`, `language/statements`, `language/declarations`, `language/module-code` and negative-phase shards rather than the full language tree on every edit.
