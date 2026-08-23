# Duplicate class private-name early errors

## Decision

A duplicate entry in a class body's `PrivateBoundIdentifiers` is one closed
condition:

`EarlyErrorCode::ClassDuplicatePrivateName`

The sole wire spelling is `E_CLASS_DUPLICATE_PRIVATE_NAME`. Both Script and
Module parsing report the condition during the early-error phase as a
`SyntaxError`, with the parser's source span. The rejected class never reaches
IR lowering.

## Measured parser boundary

Pinned `boa_parser-0.21.1` emits the exact, case-sensitive message

```text
private identifier has already been declared
```

from five branches in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`.
Those branches cover duplicate private fields, methods and accessors, including
static/instance conflicts. One exact classifier row owns all five producers;
source spelling and element kind do not create separate diagnostic conditions.

The front-end code domain and parse-failure row count remain type-level
invariants. The exhaustive `lila-ir` projection maps the new variant to
`IrDiagnosticKind::EarlyError`, so adding a new code without choosing its
rejection phase fails to compile.

## Positive boundaries

A private getter and setter with the same name are one permitted accessor pair
when their static placement agrees. A nested class body also owns an independent
private-name domain, so an outer and nested class may each declare `#x`.

These are parser-state boundaries, not source-text exceptions. A source scan for
repeated `#x` would reject both valid forms and is therefore outside the
contract.

## Durable verification

Front-end tests cover class declarations and expressions under both Script and
Module goals, with field/field, method/static-field, getter/getter and
setter/field conflicts. Positive tests preserve getter/setter pairs and nested
classes. The retained Module regression sends the real parse failure through
`module_parse_failure_diagnostic` and requires the same code, phase, error type
and source span.

At `2026-08-23`, the capped serial verification passes:

- `lila-front`: `42/42`;
- focused `lila-ir` early-error tests: `3/3`; and
- the exact pinned declaration/expression duplicate-private-name cohort: 32
  physical files and `64/64` sloppy/strict Wasm-AOT executions, with every
  failure and non-success bucket at zero.

The Test262 cohort contains the two `fields-duplicate-privatenames.js` files,
the 26 `grammar-privatemeth-duplicate-*` early-error files and the four valid
getter/setter or nested-class boundary files under
`language/{expressions,statements}/class/elements`. This is bounded evidence,
not class-grammar, T07 or aggregate Test262 closure.
