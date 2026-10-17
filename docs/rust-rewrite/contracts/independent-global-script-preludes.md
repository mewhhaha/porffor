# Independent global Script preludes

`CompileOptions.module_prelude` accepts a global Script before either a Script or Module entry. The prelude and entry are parsed separately. Their original bytes and directives remain independent; both execute in the same Realm and Global Environment. The entry parses before any prelude executes. Harness failures have checked setup provenance and cannot satisfy an entry's expected parse, resolution or runtime error.

For Script entries, lowering retains a private global entry owner alongside the existing prelude owner. Main contains no test declarations or source body. It executes the prelude first, then instantiates and evaluates the original entry, preserving normal and abrupt completion. The preceding Script invalidates compile-time assumptions about mutable globals and intrinsics. Prepared imports retain their actual source graph and host loader authority.

The program-local `lila.global_prelude_phase` scalar is declared after all shared runtime globals and program guards, before the section is sealed. Only P exports it. It records prelude execution and successful completion so native errors keep their underlying classification and their separate setup origin. R's schema, exported shared globals and runtime layout version remain unchanged.

Test262 evaluates its exact harness bytes as this separate Script. Strict mode prefixes only the unchanged test contents; raw cases remain exact and have no harness. Agent worker bodies use the same independent prelude boundary. The optional oracle mirrors these separate Script phases in one context. Jobs retain the existing shared final checkpoint; successful-prelude jobs do not gain separate asynchronous source-lineage attribution.

Validation for this private source component consists of formatting, strict patch replay and source mutation controls. Authored Rust controls and repaired real Test262 cases remain unrun until the root's coherent verification batch.
