# Duplicate import-attribute-key early errors

## Decision

A `WithClause` whose `WithClauseToAttributes` result contains two different
entries with the same `[[Key]]` is one closed pre-evaluation condition:

`EarlyErrorCode::ModuleDuplicateImportAttributeKey`

Its sole wire spelling is `E_MODULE_DUPLICATE_IMPORT_ATTRIBUTE_KEY`. The code
is Module-specific because `WithClause` occurs only on static import and
export-from declarations. `WithEntries` derives each `ImportAttribute`
`[[Key]]` through `AttributeKey`; this is not a generic object-property or
module-request validation code.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has one textual producer of the complete,
case-sensitive message `duplicate import attribute key`, in the shared
`parse_module_request_attributes` owner in
`parser/statement/declaration/import.rs`. The static-import request parser and
the typed re-export request parser both call that owner, so they intern
identifier-name, keyword and string-literal keys and apply the same equality
check. Consequently `type` and `"type"` denote the same key and a list
containing both must reject. Distinct keys and a trailing comma remain valid.

Boa renders each `Error::general` as the raw message followed by a source
position. One classifier row therefore requires the anchored rendered prefix
`duplicate import attribute key at line`. It must not use an anywhere-substring
rule: Boa also interpolates user-chosen local export names into another
`Error::general`, so a source can otherwise forge this complete text inside a
different diagnostic. The rule's closed `ParseFailurePattern::StartsWith`
variant makes that anchoring explicit and leaves the existing
`ContainsAll` fragment sets unchanged. A broader fragment such as `import
attribute` would also collide with malformed attribute syntax; run-time
dynamic-import option processing is not this grammar condition.

## Parse and module-record boundary

The parser rejection becomes `Early` / `SyntaxError` under the Module goal.
The focused IR witness crosses the front-to-IR diagnostic projection; the
generic retained-dependency path consumes that same typed parse diagnostic but
is not re-proved by this lane. Script parsing does not gain a special
classification: static import/export syntax is itself unavailable under that
goal.

`lila-ir::modules::record::ModuleRequestAttributesIr::try_new` independently
protects the canonical IR module-request representation and returns
`DuplicateImportAttributeKeyIr`. Its display text is
`duplicate import attribute key: <key>`. That constructor also protects
programmatically assembled records and is not a Boa parse failure. The
classifier's required `at line` suffix deliberately does not match this keyed
display. This lane must preserve it as a separate owner rather than route,
delete or duplicate it through the parse classifier.

## Typed encoding

- Add the one `EarlyErrorCode` variant and wire spelling at the parse-owned
  front-end boundary.
- Add exactly one anchored-prefix row whose witness is the complete rendered
  Boa message.
- Evaluate a const ownership assertion proving every row for this code uses
  `StartsWith`; replacing it with `ContainsAll` must fail `cargo check`.
- Evaluate a const adversarial assertion proving the forged local-export
  message remains unclassified and the overlapping duplicate-export message
  retains `ModuleDuplicateExport` ownership.
- Include the new variant in `lila-ir`'s exhaustive rejection-kind mapping.
- Exercise a real failed Module parse through
  `module_parse_failure_diagnostic`; a hand-built message alone would not prove
  the front-to-IR projection.

The variant and row counts remain written into array types. The existing const
assertions prove wire-name closure, table witness uniqueness, classifier
reachability and parse-to-IR early-error consistency. After this extension the
closed domain has 56 variants and the parse table has 55 rows.

## Durable witnesses

Front-end witnesses cover both static grammar forms, identifier/string spelling
equivalence, distinct keys and trailing commas. Adversarial real sources prove
that a user-chosen export name cannot forge the anchored message and that a
duplicate exported name containing the same text remains
`ModuleDuplicateExport`. The IR projection uses an actual duplicate-key
export-from declaration and requires the typed code, `Early` phase,
`SyntaxError` constructor and a source span.

The source-level contract pins one reviewed message occurrence in the shared
vendored attribute parser, none in the export parser, the typed re-export
parser plus both export-from calls, and the separate
`DuplicateImportAttributeKeyIr` display prefix. These assertions are structural
drift alarms, not a substitute for compilation or behavior tests.

## Verification

The centralized eight-core lane passed `cargo xc`, the focused front classifier
and IR projection filters (`3/3` and `1/1`), the broad front and module-early
cohorts (`93/93` and `39/39`), and the independent keyed IR record invariant
(`1/1`). The exact
`language/module-code/import-attributes/early-dup-attribute-key` Test262 filter
passes `3/3` Wasm-AOT executions with Unsupported, Crash and Bug all at zero.

## Nonclaims

This lane does not implement import-attribute host semantics, supported-key or
supported-value validation, module loading, dynamic `import()` option
processing, JSON-module behavior, or the broader module grammar. It does not
claim a broader current-pin Test262 cohort size, a measured pass gain, T07
closure or an aggregate conformance result.
