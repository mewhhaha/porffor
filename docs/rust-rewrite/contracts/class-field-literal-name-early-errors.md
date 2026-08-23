# Class-field literal-name early errors

## Decision

Public class fields and auto-accessors have two closed literal-name rejection
conditions:

- `EarlyErrorCode::ClassFieldConstructorName` for a non-static literal
  `constructor`; and
- `EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName` for a static
  literal `constructor` or `prototype`.

Their sole wire spellings are `E_CLASS_FIELD_CONSTRUCTOR_NAME` and
`E_CLASS_STATIC_FIELD_CONSTRUCTOR_OR_PROTOTYPE_NAME`. Script and Module parsing
report both conditions during the early-error phase as a `SyntaxError`, with
the parser's source span. Rejected elements never reach class lowering.

## Measured parser boundary

Pinned `boa_parser-0.21.1` emits the exact, case-sensitive messages

```text
class may not have field definitions named 'constructor'
class may not have static field definitions named 'constructor' or 'prototype'
```

from eight branches in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`.
Four branches per condition cover ordinary public fields and public
auto-accessors, each with and without an initializer. Two exact classifier rows
own those producers. The exhaustive `lila-ir` projection requires both codes
to choose their rejection kind.

The pinned parser previously dispatched every non-static identifier token
`constructor` into constructor-method parsing, even when the following token
was `;` or `=`. Those valid `FieldDefinition` prefixes therefore failed inside
`UniqueFormalParameters` with a generic malformed diagnostic and never reached
the existing field-name early-error branch. The vendored dispatch now selects
the constructor-method path only when the next token is `(`. Identifier fields
consequently reach the same typed condition as their StringLiteral forms;
auto-accessors already reached their dedicated branches, and ordinary
constructor methods are unchanged.

## Computed-name and element-kind boundaries

The rules inspect the literal `PropName`. A computed name such as
`["constructor"]` or `["prototype"]` therefore remains valid even though it
evaluates to the same String value. An ordinary `constructor() {}` method is a
constructor definition rather than a field and also remains valid. Literal
non-static `prototype` fields and auto-accessors are permitted because that
name belongs only to the static condition.

Private `#constructor` is a distinct, already-classified private-name
restriction. Keeping these domains separate prevents a source-spelling scan
from collapsing public fields, computed names, methods and private names into
one brittle condition.

## Computed static `prototype` execution boundary

Computed `prototype` remains valid syntax, but class definition then attempts
to define a public static element on the constructor's existing `prototype`
property. Lila installs that property with the class-specific
`writable: false`, `enumerable: false`, `configurable: false` descriptor. A
single Wasm guard rejects the conflicting property key before every public
static field, method/accessor and auto-accessor definition. Private elements,
instance elements and ordinary objects do not enter that guard.

Ordering remains observable. A static field initializer runs before its
`DefineField` TypeError. A static auto-accessor installs its public accessor
before ordered backing-field initialization, so the conflicting definition
throws before its initializer runs. Computed static `constructor` remains a
positive boundary and may create the corresponding configurable public
element.

## Durable verification

Front-end tests exercise all eight ordinary-field/auto-accessor producer
shapes across declaration/expression forms and Script/Module goals. Positive
witnesses preserve computed instance/static fields, computed auto-accessors and
an ordinary constructor method. Retained Module regressions route both real
parse failures through `module_parse_failure_diagnostic` and require their
typed code, phase, error type and source span.

The exact pinned Test262 evidence is split along the same two conditions. The
non-static cohort contains four negative literal/string-name files plus one
computed-name positive boundary (`10` executions). The static cohort contains
eight negative literal/string-name files for `constructor` and `prototype`
plus five computed-name positive boundaries (`26` executions). Combined, the
cohort is 18 files and 36 sloppy/strict executions. This is bounded class-field-
name evidence, not class grammar, field execution, T07 or aggregate Test262
closure. The complete cohort passes `36/36`. The adjacent nine-file computed
static method/accessor and class-prototype-descriptor cohort passes `18/18`,
and the durable Wasm class-element fixture passes `1/1` while pinning field and
auto-accessor initializer order.
