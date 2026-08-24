# Class field initializer `Contains SuperCall` early errors

**Status:** Product condition and GeneratorExpression-updated shared producer
census focused-verified 2026-08-24

## Decision

One pre-evaluation condition owns every class field initializer whose retained
syntax `Contains SuperCall`:

`EarlyErrorCode::ClassFieldInitializerContainsSuperCall`

Its sole wire spelling is
`E_CLASS_FIELD_INITIALIZER_CONTAINS_SUPER_CALL`. It derives phase `Early` and
native error type `SyntaxError` under both Script and Module goals.

The condition covers public and private fields, instance and static fields,
and public auto-accessors. Heritage is irrelevant. It rejects `super()`,
including calls reached through ordinary or async arrows, but does not reject
`super.value`. A nested class owns its own constructor and field semantics and
is therefore a traversal boundary.

## Specification boundary

ECMA-262 2026
[15.7.1, Class Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-class-definitions-static-semantics-early-errors)
rejects a `FieldDefinition` when its initializer is present and the initializer
`Contains SuperCall`. The same class-element restriction applies to the
current auto-accessor syntax represented by pinned Boa's accessor-field AST.

This is separate from:

- a base constructor whose `HasDirectSuper` is true;
- a class static block whose statement list `Contains SuperCall`;
- a non-constructor method whose parameters or body have direct `super()`;
- a whole Script or Module item list that `Contains super`; and
- direct-eval source, whose legality depends on the calling environment.

Those conditions have different syntax-directed owners even where the parser
previously reused the same diagnostic text.

## Pinned-Boa producer boundary

`ClassBody::parse` in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`
already applies the required predicate after each complete class element is
parsed. Four exhaustive match arms cover:

1. `PrivateFieldDefinition`;
2. `PrivateStaticFieldDefinition`;
3. grouped public `FieldDefinition`, `AccessorFieldDefinition`, and
   `StaticFieldDefinition`; and
4. `StaticAccessorFieldDefinition`.

Each branch tests an optional initializer with
`contains(initializer, ContainsSymbol::SuperCall)`. No grammar or accepted
syntax changes are needed.

Before this lane, all four branches emitted `invalid super usage`, which was
also emitted by the Script-body condition and five callable producers. A
broad classifier row could not distinguish those semantic owners. The bounded
producer repair changes only the four field branches to:

```text
class field initializer cannot contain super call
```

Boa appends a source position, so the classifier owns the complete fixed
prefix:

```text
class field initializer cannot contain super call at line
```

After the repair, that new raw message occurs exactly four times, all in the
reviewed field arms. The later ordinary function expression/declaration lanes
give two callable productions their own messages. The subsequent
AsyncFunctionExpression and GeneratorExpression lanes give two more
productions their own messages. On current head, `invalid super usage` occurs
exactly three times across pinned `boa_parser`: the fixed-position Script
producer, the shared generic declaration default and the async-generator-
expression producer. The base-constructor, static-block and four separately
typed function conditions retain their unique messages.

## Typed encoding and classifier safety

The closed front domain and parse table grow from 64 to 65 entries. One
`StartsWith` row maps the complete producer-owned prefix to
`ClassFieldInitializerContainsSuperCall`. `StartsWith` is required because
only the decimal position follows the prefix. A `ContainsAll` fragment would
allow user-controlled text inside another diagnostic to forge the code.

The enum addition is compile-time owned by:

- an evaluated `ParseClassified::from_parse_table` witness;
- an exact-single-owner assertion that independently spells the prefix and
  requires `StartsWith`;
- the table-wide populated-row, disjoint-witness, wire-name, parse-owner and
  IR-kind assertions;
- classifier checks that keep the field prefix distinct from Script,
  base-constructor, static-block and method-owned `super` messages; and
- an interpolated duplicate-export witness proving that a user-selected export
  name containing the complete field prefix remains `ModuleDuplicateExport`.

`lila-ir` adds one explicit exhaustive arm mapping the code to
`IrDiagnosticKind::EarlyError`; it does not add a second message table.

## Permanent behavior matrix

The direct front-end rejection matrix runs every source under both Script and
Module goals and requires the exact field code, phase `Early`, native
`SyntaxError`, and a nonempty span. It covers:

- public, static public, private and static private fields;
- instance and static auto-accessors;
- class declarations and expressions;
- base and derived classes, proving heritage is irrelevant; and
- ordinary, async and nested arrow traversal.

Positive controls preserve absent initializers, `SuperProperty`, a nested
derived constructor, and string text containing `super()`. The shared class
precedence matrix records that a parsed field rejection occurs before the
deferred base-constructor check, while the existing per-field
`ContainsArguments` check remains earlier than the ClassBody `SuperCall`
check.

A real failed Module parse crosses `module_parse_failure_diagnostic`, and a
real rejected dependency crosses `ModuleSourceIr::Rejected` and `build_graph`
without request rescanning. A valid dependency with `field = super.value`
remains a parsed graph node.

## Durable structural guard

The shared super-producer source guard recursively inventories pinned Boa and
requires:

- exactly three remaining `invalid super usage` literals: the fixed Script
  owner, shared declaration default and async-generator-expression owner, with
  their common or direct parameter-start positions preserved;
- exactly one ordinary-function-expression-specific message on its completed-
  node `Contains Super` branch;
- exactly one ordinary-function-declaration-specific message selected by the
  shared callable-declaration predicate;
- exactly one async-function-expression-specific message on its completed-node
  `Contains Super` branch;
- exactly one generator-expression-specific message on its completed-node
  `Contains Super` branch;
- exactly four field-specific messages, all in `class_decl/mod.rs`;
- exactly one match in each of the private, private-static, grouped-public and
  static-auto-accessor field arms;
- an optional initializer and exact `ContainsSymbol::SuperCall` predicate in
  each arm, retaining the class-element `position`;
- no old generic message in the class parser;
- the unique base-constructor and static-block producer messages and their
  existing predicate/order guards;
- ordinary and async arrow traversal plus ordinary callable and nested-class
  stopping boundaries in pinned `boa_ast`; and
- the existing parse-once and sole-classifier product boundary.

The raw-message counts are not sufficient by themselves: the bounded branch
shapes ensure moving all four messages to the wrong class element still fails
the guard.

## Complete pinned Test262 cohort

At Test262 revision `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, searching the class
expression and statement trees for the exact metadata statement
`Initializer is present and Initializer Contains SuperCall` yields 60 physical
files: 30 under `language/expressions/class/elements` and the corresponding 30
under `language/statements/class/elements`.

All 60 declare only `flags: [generated]`; none declares `onlyStrict`,
`noStrict`, `raw`, or `module`. The closed execution plan therefore contains
exactly 120 sloppy/strict Wasm-AOT variants. The files span literal, string,
computed and private names; instance and static fields; direct and
arrow-carried calls; and conditional, equality and `typeof` expression
placement.

Dynamic direct/indirect-eval files are excluded: their metadata names a
StatementList condition inside eval source, not the statically retained field
initializer predicate owned here.

## Verification

The coordinated batch verifier ran:

```sh
cargo test -p lila-front class_field_initializer_super_call -- --test-threads=1
cargo test -p lila-front known_script_and_class_super_producers_stay_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-front pinned_contains_super_traversal_stays_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-ir class_field_initializer_super_call_module_parse_maps_to_an_early_syntax_error -- --exact --test-threads=1
cargo test -p lila-ir rejected_class_field_super_call_dependency_keeps_its_code_through_graph_build -- --exact --test-threads=1
```

All five focused commands passed `1/1` at the field-lane checkpoint. `cargo fmt
--all -- --check` and `cargo xc` were green. The complete front library passed
`125/125`; the relevant IR early and graph groups passed `46/46` and `43/43`.
The later FunctionExpression lane updated the shared producer census; the
complete front library passed `129/129`, and the relevant IR early and graph
groups passed `47/47` and `45/45` with that census in place. The subsequent
FunctionDeclaration update leaves the complete front library at `134/134` and
the relevant IR early and graph groups at `48/48` and `47/47`.
The subsequent AsyncFunctionExpression producer-census update passes the
complete `138/138` front gate and the relevant `49/49` IR early and `49/49`
graph groups.
The subsequent GeneratorExpression producer-census update passes the complete
`142/142` front gate and relevant `50/50` IR early and `51/51` graph groups.

The metadata-derived 60-file cohort was enumerated one exact suite-relative
path at a time with the Wasm-AOT backend, `--jobs 1`, `--threads 1` and the
repository timeout. It passes exactly `120/120` sloppy/strict variants with
every non-success bucket at zero.

## Nonclaims

This lane does not change valid class-field execution, initializer order,
`this` initialization, static-field installation, auto-accessor installation,
computed-name evaluation, method-owned `super()` conditions, direct eval, or
dynamic source support. It does not claim a new Test262 pass, refresh aggregate
status, close the class grammar bucket, or complete T07.
