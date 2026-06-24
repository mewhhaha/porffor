# T07 — Parser boundary, grammar coverage and early errors

**Status:** Ready after T02 front/IR split  
**Parallel group:** Core foundations  
**Depends on:** T01, T02  
**Blocks:** T08, T09, T12, T24 and parser-failure closure

## Objective

Make parsing and static-semantics classification complete, deterministic and source-located for the pinned ECMAScript grammar. Remove the current double-parse shape where `porffor-front` validates and stores source text while `porffor-ir` reparses independently.

## Architecture

- Define a parsed source product that owns the Boa AST/interner/scope data needed by lowering, or perform lowering inside a controlled parser session and return a Porffor-owned syntax representation.
- Parse exactly once per compilation unit.
- Preserve script vs module goal, filename, spans, strictness and source text.
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
cargo test -p porffor-front --quiet
cargo test -p porffor-ir early_error --quiet
cargo test -p porffor-engine --quiet
./target/debug/porf test262 run language --execution-backend wasm
```

During development run focused `language/expressions`, `language/statements`, `language/declarations`, `language/module-code` and negative-phase shards rather than the full language tree on every edit.