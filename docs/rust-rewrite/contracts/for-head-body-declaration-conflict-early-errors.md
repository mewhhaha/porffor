# For-head/body declaration-conflict early errors

**Status:** Normative T07 extension focused-verified, 2026-08-23

## Decision

A lexical declaration in an iteration head whose bound name is also declared
by `var` in that iteration statement's body is one closed pre-evaluation
condition:

`EarlyErrorCode::ForHeadBodyDeclarationConflict`

Its sole wire spelling is `E_FOR_HEAD_BODY_DECLARATION_CONFLICT`. The code
names the shared static-semantics intersection, not any one loop spelling. It
therefore owns both a classic `for` head's `LexicalDeclaration` and a
`for-in`/`for-of` head's `ForDeclaration` when a head `BoundName` also occurs
in the body `Statement`'s `VarDeclaredNames`.

## Specification boundary

The edition-pinned ECMA-262 2026 sections
[14.7.4.1, Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html#sec-for-statement-static-semantics-early-errors)
and
[14.7.5.1, Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html#sec-for-in-and-for-of-statements-static-semantics-early-errors)
establish the `let`/`const` base rule: a `SyntaxError` is required when any
element of the `BoundNames` of a classic
`for` statement's `LexicalDeclaration` also occurs in the
`VarDeclaredNames` of its body `Statement`, and the same intersection rule
applies to a `ForDeclaration` in `for-in`, `for-of` and `for-await-of`.

The living specification's corresponding
[classic-`for` clause](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-for-statement-static-semantics-early-errors)
and
[`for-in`/`for-of`/`for-await-of` clause](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-for-in-and-for-of-statements-static-semantics-early-errors)
incorporate Explicit Resource Management into `LexicalDeclaration` and
`ForDeclaration`. Those living productions extend the same intersection to
`using` and `await using` where the grammar permits them. This contract does
not attribute those later declaration forms to the frozen 2026 text.

This is a relation between the lexical head and the body of the same
iteration statement. It is not the general Script/Module lexical-name
collision and is not duplicate `BoundNames` inside the head. A `var` head is
outside the rule because it is not a `LexicalDeclaration` or
`ForDeclaration`. A `var` declaration inside a nested function is outside the
body statement's `VarDeclaredNames`, because function bodies are traversal
boundaries for that static-semantics operation.

The rule applies to `let` and `const` heads and to `using` and `await using`
where those declaration forms are permitted by the surrounding iteration
grammar. A `using` or `await using` declaration in a `for-in` head is already
a distinct, earlier early error owned by
`EarlyErrorCode::ForInUsingDeclaration`; adding a conflicting body `var` must
not transfer that source to this new code.

## Measured Boa boundary

Across every Rust source in pinned `boa_parser-0.21.1`, exactly two producers
use the same fixed, case-sensitive raw message. Both are in
`vendor/boa_parser-0.21.1/src/parser/statement/iteration/for_statement.rs`:

```text
For loop initializer declared in loop body
```

- the classic-`for` producer currently at lines 279-290 computes
  `var_declared_names(&body)`, walks `bound_names(initializer.declaration())`
  only for `ForLoopInitializer::Lexical`, and rejects their intersection;
- the iterable-loop producer currently at lines 329-350 computes
  `var_declared_names(&body)`, walks `bound_names(&init)` only for the four
  lexical `IterableLoopInitializer` variants, and rejects the same
  intersection.

`Error::general` appends the source position. Exactly one classifier row owns
the new code and uses the complete rendered prefix
`For loop initializer declared in loop body at line` through
`ParseFailurePattern::StartsWith`. An anywhere-substring rule is not an
acceptable encoding: parser messages elsewhere may interpolate user source,
while this fixed message is parser-owned at the beginning of the rendered
diagnostic.

The source-level producer guard recursively inventories every Rust source in
the pinned Boa parser package and requires exactly two copies of the raw
message. It separately requires both copies to remain in the reviewed
`for_statement.rs` file and pins the two surrounding semantic shapes: each
branch must compute `var_declared_names(&body)`, iterate the appropriate
`bound_names` source, and test `vars.contains(&name)` before emitting the
message. A literal-count assertion alone would permit a moved or duplicated
message to conceal loss of the actual intersection check.

## Goal and diagnostic boundary

The condition is reachable under both Script and Module goals. Direct entry
parsing must produce
`ParseCode::Early(ForHeadBodyDeclarationConflict)`, with phase `Early`, native
`SyntaxError` and a nonempty source span. The two goals consume the one
front-end classification table.

A loaded dependency is parsed under the Module goal and retained as a
structured rejection in `ModuleSourceIr`. A real rejected dependency must
cross `build_graph` and project through `module_parse_failure_diagnostic` to
`IrDiagnosticKind::EarlyError`, preserving the same code, `Early` phase,
`SyntaxError` constructor and nonempty span. A hand-built diagnostic is not a
substitute for this retained front-to-IR path.

## Typed encoding

- One `EarlyErrorCode` variant and one wire spelling extend the closed
  front-end domain.
- Exactly one anchored-prefix classifier row carries the complete rendered
  Boa message.
- An evaluated `ParseClassified` const assertion makes the code parse-owned;
  deleting its classifier row while retaining the variant must fail to build.
- A const ownership assertion requires exactly one row for the code and the
  exact independently spelled `StartsWith` prefix.
- The variant extends `lila-ir`'s exhaustive rejection-kind mapping. No
  catch-all can absorb it.
- Real Script, Module, module diagnostic and retained-dependency graph
  witnesses exercise the typed projection without a second message table.

The pre-extension closed domain has 57 variants and the parse-failure table
has 56 rows. This written extension grows them to 58 and 57 respectively;
both counts remain encoded in array types.

## Durable witnesses

Eleven direct front-end source shapes run under both Script and Module goals.
They cover classic `for` with `let`, `const`, `using`, and async-context
`await using`; `for-in` with `let` and `const`; `for-of` with `let`, `const`,
`using`, and async-context `await using`; and async-context `for await` with a
`let` head. Positive boundaries retain valid `var` heads and head-name
redeclarations inside both nested function expressions and a nested
`FunctionDeclaration`. An ownership control combines a forbidden
`for-in using` head with a body `var` collision and requires the existing
`ForInUsingDeclaration` code rather than the new code.

The front-to-IR witness parses a real rejected Module. The graph witness uses
a valid root module importing a dependency whose retained `ModuleSourceIr`
contains `for (let x of []) { var x; }`; `build_graph` must return the typed
dependency rejection and its nonempty span.

## Pinned Test262 cohort

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains exactly these eight direct `phase: parse`, `type: SyntaxError`
witnesses:

- `language/statements/for/head-let-bound-names-in-stmt.js`
- `language/statements/for/head-const-bound-names-in-stmt.js`
- `language/statements/for-in/head-let-bound-names-in-stmt.js`
- `language/statements/for-in/head-const-bound-names-in-stmt.js`
- `language/statements/for-of/head-let-bound-names-in-stmt.js`
- `language/statements/for-of/head-const-bound-names-in-stmt.js`
- `language/statements/for-of/head-using-bound-names-in-stmt.js`
- `language/statements/for-of/head-await-using-bound-names-in-stmt.js`

The first seven files have no execution-mode flag and expand to fourteen
sloppy/strict executions. The final file has `flags: [module]` and contributes
one Module execution, for exactly fifteen variants. The cohort spans both
specification productions and every permitted lexical declaration kind in the
pinned parser. Valid `head-var-bound-names-in-stmt.js` siblings are controls,
not members of the negative cohort.

## Verification and nonclaims

The shared eight-core-capped verification phase passed:

- `cargo xc` for the workspace;
- `cargo test -p lila-front --lib -- --test-threads=1`, `101/101`;
- the filtered `lila-ir` early-module projection tests, `41/41`;
- the exact retained-dependency graph witness, `1/1`; and
- all eight pinned Test262 files above, run one exact path at a time with
  `--jobs 1 --threads 1`, for exactly `15/15` successful Wasm-AOT variants.

This lane classifies one static-semantics condition. It does not change the
vendored parser, implement loop execution or resource disposal, claim that
these were newly passing, close all iteration grammar, close T07, refresh
aggregate Test262 status or publish a conformance result.
