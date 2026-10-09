# RegExp runtime capability rejection

A missing pattern-compiler capability is a T19 failure, not a JavaScript
exception. `RuntimeSemanticGap::RegExpRuntimePatternCompilation` owns ABI code 7
in the existing mandatory `lila_host.reject_runtime_semantics` import. Dynamic
source codes 0–5 and Temporal code 6 remain unchanged. The generic host binding
retains this typed gap; Test262 reports Unsupported/NotImplemented before testing
runtime-negative expectations. Message text does not determine that classification.

The emitted compiler now returns only its closed Compiled/SyntaxError/
ResourceExhausted/CorruptProgram status. Complete computed q and all seven v
properties of strings replace its last pending capability node and producer.
The unused helper Unsupported word 2 and decoder arm are deleted; surviving words
remain 0/1/3/4. The independently live static entry-kind Unsupported is separate
and still invokes the emitted compiler for candidate rows that static compilation
cannot provide. Missing literal/compiled programs retain their typed semantic
rejection before execution. SyntaxError, resource RangeError and corrupt-program
Error retain their realm-aware JavaScript routes. Only Compiled with a nonzero
program may publish; failed compilation preserves an existing receiver.

The literal emitter matches static compilation exhaustively. Program supplies
`&RegExpProgram` to allocation and the shared slot writer; InvalidSyntax retains
the syntax-error route; None rejects when the literal is evaluated. Neither
allocation nor the static slot writer accepts an optional program or invents a
zero handle. Exec rejects a missing program through the same nonreturning semantic
import before matcher layout validation. The unreachable zero-program simple
matcher and final pattern fallback are deleted; exec has one compiled-program
route. Brand checks and corrupt-descriptor errors retain their real JavaScript
errors. `@@match` now follows one ordinary protocol for every object receiver:
ToString input, Get flags, then observable exec dispatch. Its source catalogue
and orphaned helper owners are deleted; custom exec receivers remain supported.
The cached exec operation calls callable values (including proxies) and validates
Object/null, or checks the actual RegExp brand before intrinsic fallback. String.match
invokes its created RegExp's observed `@@match`, including the intrinsic; its
raw-pattern fallback is deleted. Separate String.matchAll, `@@search` and other
String source routes remain an audit.

Existing Engine controls now require real matching for computed finite strings
and all seven string properties, alongside names, lookbehind, code-point properties,
true syntax errors and resource-cap receiver preservation. Constructor and
recompile controls observe the installed matcher and public slots. Three additional
paired semantic sources cover finite algebra, longest-priority backtracking,
empty progress, iv folding, reverse UTF-16 bounds and rollback. The separate runner
witness requires successful computed matching to pass positive cases and to fail
runtime negatives because no expected exception occurred; its static finite iv
matching control remains. No catch or negative expectation grants unsupported
semantics a pass.

Existing raw compiler controls move q from the failure-only table to actual
descriptor roundtrip and released-workspace reuse controls. Remaining syntax and
resource rows retain zero-handle, rollback and failure-publication assertions.
All new source remains uncompiled and unexecuted. Source formatting and patch
checks are not executable evidence. No pinned aggregate, broad pass or T19
closure is claimed. The later combined checkpoint includes these focused controls:

```sh
cargo test -p lila-ir --lib runtime_semantics::tests
cargo test -p lila-aot-wasm --test runtime_link -- runtime_regexp_entry_kind_structure::
cargo test -p lila-engine --lib invalid_runtime_semantic_host_codes_remain_abi_errors
cargo test -p lila-engine --test aot_regexp -- aot_regexp_runtime_gap:: --test-threads=1
cargo test -p lila-test262 --lib computed_finite_regexp_matching_passes_positive_and_rejects_runtime_negative
cargo test -p lila-engine --lib runtime_regexp_compiler_tests -- --test-threads=1
```

The primary semantics are [RegExpInitialize](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpinitialize)
and [Patterns](https://tc39.es/ecma262/multipage/text-processing.html#sec-patterns):
a valid pattern must receive real matcher semantics, and proven invalid grammar
must retain SyntaxError. Complete Unicode
set string/algebra validation now feeds the completed finite atom, while executable
verification remains due. A conservative typed gap cannot claim that an unvalidated
input is valid or satisfy a negative syntax expectation.

The [code-point property contract](runtime-regexp-codepoint-property.md) records
the complete alias image and full-pattern-validation-before-publication handoff.
