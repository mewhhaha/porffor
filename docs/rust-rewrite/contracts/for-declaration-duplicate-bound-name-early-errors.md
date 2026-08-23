# ForDeclaration duplicate-BoundName early errors

**Status:** Implemented, independently reviewed, and focused-verified
2026-08-23

## Decision

A `ForDeclaration` whose `BoundNames` contains a duplicate entry is one closed
pre-evaluation condition:

`EarlyErrorCode::ForDeclarationDuplicateBoundName`

Its sole wire spelling is
`E_FOR_DECLARATION_DUPLICATE_BOUND_NAME`. The code names the specification
condition rather than one loop spelling or binding-pattern shape. It therefore
owns the same duplicate-name rejection in `for-in`, `for-of`, and
`for-await-of` heads.

## Specification boundary

The edition-pinned ECMA-262 2026 section
[14.7.5.1, Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html#sec-for-in-and-for-of-statements-static-semantics-early-errors)
requires a `SyntaxError` if the `BoundNames` of a `ForDeclaration` contains any
duplicate entries. In that edition, `ForDeclaration` is
`LetOrConst ForBinding`, and `ForBinding` may be a `BindingIdentifier` or a
`BindingPattern`. A single identifier contributes one name, so the condition
is reachable through an array or object binding pattern with repeated bound
names. It is independent of strictness and applies before evaluation under
both Script and Module goals.

The
[corresponding living-specification clause](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-for-in-and-for-of-statements-static-semantics-early-errors)
retains the same duplicate-`BoundNames` rule and extends `ForDeclaration` with
`using` and `await using`. Those new alternatives require
`ForBinding[~Pattern]`, however, so each can bind only one identifier and
cannot make this condition true. Pinned Boa's actual loop-head dispatch enforces
that boundary earlier: `for_statement.rs` calls
`lexical.rs::using_declaration_kind(..., for_head = true)`, whose ordinary and
`await` branches return `None` when the would-be binding token is `[` or `{`.
Those spellings therefore do not call `LexicalDeclaration::for_head` and do not
construct a resource `ForDeclaration`. A plain `using [x, x]` spelling may
instead continue through the regular-expression route as the computed property
access `using[x, x]`; this is not a resource declaration. `BindingList` retains
a defensive pattern rejection if invoked through another path, but it is not
the current loop-head owner. The iterable producer's `Using` and `AwaitUsing`
arms consequently receive only identifier-backed declarations and cannot add a
reachable duplicate case.

A classic `for (let ...; ...; ...)` head contains a `LexicalDeclaration`, not
a `ForDeclaration`; duplicate names there remain owned by
`EarlyErrorCode::DuplicateLexicalDeclaration`. A `var` iterable-loop head is
also not a `ForDeclaration` and may contain duplicate bound names. Ordinary
lexical declarations, formal parameters, catch parameters, and head/body name
intersections retain their existing distinct owners.

## Measured Boa boundary

Across every Rust source in pinned `boa_parser-0.21.1`, exactly one producer
uses the fixed, case-sensitive raw message:

```text
For loop initializer cannot contain duplicate identifiers
```

The sole producer is in
`vendor/boa_parser-0.21.1/src/parser/statement/iteration/for_statement.rs`, in
`parse_iterable_loop_tail`'s shared iterable-loop `ForDeclaration` semantic
block after conversion of a lexical initializer. That block creates
`FxHashSet::default()`, traverses `bound_names(&init)`, and emits the message
only when `!names.insert(name)`. There is no corresponding classic-`for`
producer.

Before this extension, that producer was unreachable from source. Every
`let`/`const` for-head was first parsed through Boa's reusable
`LexicalDeclaration` parser, whose unconditional duplicate-`BoundNames` check
emitted `lexical name declared multiple times` before the surrounding parser
could distinguish classic `for` from `for-in`, `for-of`, or `for-await-of`.
Lila consequently classified the iterable condition as the broader existing
`DuplicateLexicalDeclaration`; the condition-specific fixed message never
crossed the product path. Resource-declaration patterns remain unreachable for
the separate grammar reason above.

The parser repair replaces both raw `loop_init: bool` fields in the lexical
declaration and binding-list parsers with one private closed context:

```text
LexicalDeclarationContext = Statement | ForHead
```

Callers select named `statement` or `for_head` constructors. `ForHead` changes
terminator and missing-initializer handling as before but defers only the
duplicate-name check; the forbidden bound name `let` is still checked while
parsing the lexical head. The generic raw message lives in one shared
duplicate-name validator. Ordinary declarations call it immediately. Every
context-dependent branch goes through one exhaustive `Statement` / `ForHead`
projection, so adding another context cannot silently inherit either policy.

The surrounding for parser retains lexical heads in a distinct
`ParsedForInitializer::DeferredLexical` state carrying the lexical keyword's
source `Position`. Its delimiter match is exhaustive over ordinary and
deferred initializers:

- an `in` or `of` delimiter converts the deferred lexical head to a
  `ForDeclaration` and routes it to the existing iterable-loop producer;
- the classic path invokes the shared generic validator at the retained keyword
  position before parsing the rest of the classic loop, preserving
  `DuplicateLexicalDeclaration` and its source location.

The retained state prevents a deferred lexical head from silently entering
either path without the correct duplicate-name owner. No source-text heuristic
or second classifier table is introduced.

`Error::general` appends the source position. Exactly one classifier row owns
the new code and uses the complete rendered prefix
`For loop initializer cannot contain duplicate identifiers at line` through
`ParseFailurePattern::StartsWith`. An unanchored substring is not the producer
contract.

The vendored-source drift guard recursively inventories all Rust files in the
pinned Boa package and requires exactly one condition-specific raw-message
occurrence in the reviewed file. It also pins the named lexical contexts, the
exact `Statement` and `ForHead` constructor projections, all four current
context-dependent branches through the one exhaustive projection, the single
shared generic validator inside the complete deferred-classic match arm, both
exhaustive delimiter routes, the two resource-pattern lookahead exits, and the
semantic shape around the iterable occurrence: `FxHashSet` creation, traversal
through `bound_names(&init)`, and the failed `insert` condition must remain
together after the head/body intersection. Literal count alone is insufficient
because it would not prove that either message still owns the right grammar
production.

## Goal and diagnostic projection

After the parser repair, the condition is reachable under both Script and
Module goals. Direct entry
parsing must produce
`ParseCode::Early(ForDeclarationDuplicateBoundName)`, phase `Early`, native
`SyntaxError`, and a nonempty source span under either goal. Before the repair,
the earlier generic lexical producer instead selected
`DuplicateLexicalDeclaration`; this is a condition-identity correction, not a
malformed-parse recovery.

A loaded dependency is parsed under the Module goal. Its real parse rejection
must be retained in `ModuleSourceIr`, cross `build_graph`, and project through
`module_parse_failure_diagnostic` to `IrDiagnosticKind::EarlyError`, preserving
the same code, `Early` phase, `SyntaxError` constructor, and nonempty span. A
constructed diagnostic or direct table witness cannot replace this retained
front-to-IR path.

## Typed encoding

- Add the one closed-domain variant and its sole wire spelling.
- Add exactly one anchored-prefix parse-classifier row.
- Add an evaluated `ParseClassified::from_parse_table` const assertion so
  deleting the row while retaining the variant fails to build.
- Add a const ownership assertion requiring exactly one row for the code and
  the exact independently spelled `StartsWith` prefix.
- Extend `lila-ir`'s exhaustive early-error mapping; no catch-all may absorb
  the new variant.
- Encode `Statement` and `ForHead` as a closed lexical-parser context selected
  only through named constructors, and retain deferred lexical heads as a
  distinct for-parser enum variant carrying the keyword position.
- Route the delimiter match exhaustively: iterable declarations reach the
  condition-specific producer; classic declarations pass through the one
  shared generic duplicate validator.
- Reuse or strengthen the adjacent iterable-loop producer guard so the
  body-conflict and duplicate-name checks remain structurally reviewed,
  including their ordering.

The pre-extension closed domain has 58 variants and the parse-failure table has
57 rows. This extension grows those array-typed counts to 59 and 58
respectively.

## Direct source matrix

Every rejecting row below must report the new code under both Script and Module
goals:

| Production | Declaration and pattern | Source |
| --- | --- | --- |
| `for-in` | `let`, array | `for (let [x, x] in {}) {}` |
| `for-in` | `const`, object | `for (const { a: x, b: x } in {}) {}` |
| `for-of` | `let`, object | `for (let { a: x, b: x } of []) {}` |
| `for-of` | `const`, array | `for (const [x, x] of []) {}` |
| `for-await-of` | `let`, array | `async function f() { for await (let [x, x] of []) {} }` |
| `for-await-of` | `const`, object | `async function f() { for await (const { a: x, b: x } of []) {} }` |

Positive controls keep distinct names valid across both binding-pattern shapes
and all three productions. `for (var [x, x] in {}) {}` and
`for (var [x, x] of []) {}` remain valid parse boundaries because the rule
does not apply to `var`.

The permanent precedence controls are deliberately narrower than a general
ordering claim:

- `for (let [x, x] of []) { var x; }` reaches Boa's existing head/body
  intersection check before the failed set insertion and must remain
  `ForHeadBodyDeclarationConflict`.
- `for (let [x, x] = []; ; ) {}` is a classic-`for`
  `LexicalDeclaration` and must remain `DuplicateLexicalDeclaration`.
- `for (let let of []) {}` rejects while parsing the forbidden bound name,
  before both the body intersection and duplicate insertion. Pinned Boa leaves
  that wording at `ParseCode::Malformed` (phase `Parse`, native `SyntaxError`),
  and it must not acquire the new code.
- `for (using [x, x] of []) {}` remains a valid ordinary-expression loop head:
  the lookahead declines the resource-declaration interpretation, and the
  regular path reads `using[x, x]` as a property access.
- `async function f() { for (await using [x, x] of []) {} }` likewise stays on
  the expression path. Its invalid iterable left-hand side remains
  `ParseCode::Malformed` (phase `Parse`, native `SyntaxError`) and must not
  acquire the new code. Neither resource-like spelling is a resource
  `ForDeclaration` witness. A single-identifier `using` declaration in a
  `for-in` head remains owned by
  `ForInUsingDeclaration`.

These controls encode pinned Boa's diagnostic ownership where multiple invalid
properties coexist. They do not assert that ECMA-262 assigns an observable
ordering among simultaneous early errors.

## Exact pinned Test262 cohort

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains exactly four direct negative witnesses for this condition. Their full
suite-relative paths are:

- `language/statements/for-in/head-let-bound-names-dup.js`
- `language/statements/for-in/head-const-bound-names-dup.js`
- `language/statements/for-of/head-let-bound-names-dup.js`
- `language/statements/for-of/head-const-bound-names-dup.js`

Each declares `phase: parse` and `type: SyntaxError` and has no execution-mode
flag, so the harness expands the four physical files to exactly eight
sloppy/strict variants. The sibling `head-var-bound-names-dup.js` files are
positive controls, not members of the negative cohort. The pin contains no
direct `for-await-of` duplicate-name file; the permanent source matrix owns
that production boundary.

The CLI filter is path-prefix based. Focused verification must therefore pass
each complete suite-relative path above separately and require one discovered
physical file per invocation, rather than using basename or suffix filters.

## Focused verification and nonclaims

Bounded verification completed on 2026-08-23 under the shared CPU cap.
Independent reviews of both the vendored parser repair and the front/IR closure
were clean. The capped `cargo fmt --all -- --check` and `cargo xc` gates were
green. The final complete `lila-front` library run passed `103/103`; its first
run had caught two incorrect text-guard count/indent assumptions, which were
repaired before the exact vendored-source guard passed `1/1` and the complete
front library reran green. The focused `lila-ir` early-module group passed
`42/42`, and the exact retained-dependency graph witness passed `1/1`.

The four complete suite-relative Test262 paths were each run separately with
the Wasm-AOT backend, `--jobs 1`, and `--threads 1`:

| Exact path | Result |
| --- | --- |
| `language/statements/for-in/head-let-bound-names-dup.js` | `2/2` |
| `language/statements/for-in/head-const-bound-names-dup.js` | `2/2` |
| `language/statements/for-of/head-let-bound-names-dup.js` | `2/2` |
| `language/statements/for-of/head-const-bound-names-dup.js` | `2/2` |

Together they passed exactly `8/8` Wasm-AOT variants, with every failure and
non-success bucket at zero.

The lane repairs and classifies one parser-owned static-semantics condition. It
does not implement iterable-loop execution or destructuring, change the
classic-`for` duplicate taxonomy, make resource-declaration patterns valid,
implement dynamic source evaluation, close all iteration grammar, close T07,
measure a new Test262 pass gain, refresh aggregate status, or publish a
conformance result. The negative Test262 cases may already count as successful
parse-phase `SyntaxError` tests while carrying the broader
`DuplicateLexicalDeclaration` taxonomy; this lane's material result is the
condition-specific closed identity and honest retained-dependency projection.
