# Lila compiler documentation

Lila compiles JavaScript through parsing, early errors, spec IR and lowering IR
into Wasm. The product uses the experimental Wasmtime feature set described in
[AGENTS.md](../../AGENTS.md); the debug interpreter is outside the product path.

Start with the [current failure tasks](../../tasks/README.md) for actionable
work. They own current execution evidence and root-cause triage. The documents
here explain enduring architecture and invariants. Retired implementation plans,
batch diaries and recovery narratives are available in Git history.

## Development and evidence

- [Contribution guide](../../CONTRIBUTING.md)
- [Batch workflow and verification ladder](batch-workflow.md)
- [Conformance ownership domains](conformance-ownership.md)
- [Conformance taxonomy](conformance-taxonomy.md)
- [Snapshot identity and comparison](test262-snapshot-comparison.md)
- [Publication driver](reproducible-publication-driver.md)
- [Publication restart recovery](publication-progress-recovery.md)

## Compiler and runtime

- [Architecture invariants](architecture-invariants.md)
- [Shared semantic invariants](semantic-invariants.md)
- [Operation descriptors and completion routing](operation-descriptors.md)
- [Value, heap and GC architecture](value-heap-gc.md)
- [Catchable stack exhaustion](stack-guard.md)
- [Realm intrinsics](realm-intrinsics.md)
- [Per-Realm ThrowTypeError](throw-type-error-realm.md)
- [Control-flow emission](aot-control-flow-review.md)
- [Native exception-control emission](aot-native-exception-control.md)
- [Generator expression suspension](generator-expression-suspension.md)
- [Suspended property References](aot-suspended-references.md)
- [Captured for-await heads](aot-captured-for-await.md)
- [For-await iteration environments](aot-for-await-iteration-environments.md)
- [Module loading boundary](aot-module-load-confinement.md)
- [Module path identity](aot-module-path-identity.md)
- [Module evaluation completion](deferred-module-completion.md)
- [Program cache identity](aot-cache-fingerprint.md)
- [Atomic cache publication](aot-cache-publication.md)
- [Product artifact validation](aot-product-artifact-validation.md)
- [Host randomness](host-randomness.md)

## Builtins and locale data

- [RegExp architecture](regexp-engine.md), [case folding](regexp-case-folding.md),
  and [reentrant execution](regexp-reentrant-execution.md)
- [Intl architecture](intl-architecture.md),
  [DateTimeFormat provider](intl-datetime-provider.md),
  [locale/calendar kernel](intl-datetime-locale-kernel.md),
  [NumberFormat provider](intl-numberformat-provider.md),
  and [named time zones](intl-named-time-zones.md)
- [Calendar domain](intl-calendar-domain.md) and
  [Intl.Locale options](aot-intl-locale-options.md)
- [Date parsing](aot-date-parsing.md) and [locale methods](date-locale-methods.md)
- [Temporal Duration fields](temporal-duration-number-fields.md),
  [field replacement](temporal-zoned-field-replacement.md), and
  [Buddhist calendar](temporal-buddhist-calendar.md)
- [Arguments iteration](aot-arguments-iteration.md)
- [Array callback iteration](aot-array-callback-iteration.md),
  [find methods](aot-array-find.md), [flatMap](aot-flat-map.md),
  [locale strings](aot-array-to-locale-string.md), and
  [present-index storage](array-present-index-outlining.md)
- [Property descriptor Realms](property-descriptor-realm.md),
  [definition completions](reflect-property-definition-completions.md),
  [JSON revivers](json-reviver-property-definitions.md), and
  [legacy accessor definers](legacy-accessor-definers.md)

The [contract index](contracts/README.md) points to the detailed type, lifecycle
and algorithm contracts near their implementing code and regression targets.
Vendored documentation and data provenance stay with their respective packages.
