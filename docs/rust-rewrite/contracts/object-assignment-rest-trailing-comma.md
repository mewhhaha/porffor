# Object assignment rest trailing comma

`ObjectAssignmentPattern` permits `AssignmentRestProperty` only as its final
property, without a following comma. `ObjectLiteral` permits spread properties
with a following comma and permits more properties after a spread.

The pinned Boa parser previously consumed the comma without recording it in
`ObjectLiteral`. Its `to_pattern` conversion already rejected a non-final spread,
but could not distinguish `{...rest}` from `{...rest,}`. The original pinned
Test262 assignment fixture is `var rest; 0, {...rest,} = {};`, with parse-negative
`SyntaxError` metadata and generated sloppy/strict variants.

The local vendor correction records `has_trailing_comma_spread` while parsing
an object literal and requires the flag in the AST constructor. It follows the
existing `ArrayLiteral` syntax-metadata convention. Cover conversion rejects
that flag before constructing an `ObjectPattern`; parsing object expressions
continues to accept the same punctuation. Nested conversions and for-in/for-of
assignment heads use the same conversion owner. No filename, fixture path,
rendered source, host engine, or Test262 outcome participates in the compiler
rule. Existing invalid assignment diagnostics remain the rejection boundary.

Vendor provenance is unchanged: `boa_ast` and `boa_parser` are 0.21.1 from
`https://github.com/boa-dev/boa`, upstream commit
`bc36c3fac0969ea21ea0570b62e7846f97389b73`, with upstream paths `core/ast` and
`core/parser` as recorded by the retained `.cargo_vcs_info.json` files. Existing
package versions, manifests, lockfile, licenses, registry sources and workspace
patch configuration remain unchanged. This is a source correction to the
already vendored packages.

`crates/scriptw-front/tests/object_rest_trailing_comma.rs` includes the actual
pinned assignment fixture and covers bare, computed, default, access, nested
and iterable-loop assignment patterns. Positive controls retain ordinary
property trailing commas, final rest without a comma, object spread expressions
before further properties or with a trailing comma, and valid spread expressions
inside pattern initializers or computed keys. Script, strict Script, Module
and UTF-16 input routes are covered. Existing vendored AST witnesses are updated
to provide the required flag; the existing spread-with-comma witness records it.

Native verification (2026-10-15) passes all 261 selected checks: seven new regressions, all 235 earlier published Front unit/integration IDs, and 19 focused Boa AST witnesses. The existing using-declaration witness keeps its original source and spans, using the current literal and StatementList constructors with closing-token linear end 19. Production libraries and the verified indoc macro are reused with exact source bindings.

The [portable native checkpoint](../object-rest-trailing-comma-native-20261015.json) preserves actual inventories, transcripts, both indoc setup failures, the stale test-API failure and the rejected stale source-view ELF. The fixture is included unchanged in the parser regressions. Full Test262 counts, ordinary full-workspace builds and the remaining project acceptance gates retain their separate requirements.
