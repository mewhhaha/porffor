# Parser-owned module ranges and grouped assignment targets

The original Module parse retains ordered source ranges for imports, export
lists/re-exports, export modifiers, and default keyword pairs. Default function
and class declarations also retain their end position. `ModuleRecordIr` converts
these UTF-16 positions to byte ranges in one ordered pass and owns the result.
Script and JSON records do not carry a source-module edit plan.

Dynamic imports retain their exact keyword/phase ranges from the same parser.
The linker maps those original ranges through earlier source edits, then rewrites
only real call heads. Script discovery also walks the retained AST. Methods named
`import`, regex literals after blocks/control heads and nested imports therefore
need no second lexical interpretation. Phase-head trivia keeps its exact line
terminator sequences. The old dynamic scanner remains only as conservative
evidence after Script parsing has failed; it never rewrites valid source.

The linker applies this plan after its length-preserving `import.meta` rewrite.
It no longer lexes module text or reparses anonymous default declarations. Blank
edits and generated default binding heads preserve byte length and the exact
ordered ECMAScript line-terminator sequences, including separate CR/LF barriers.
Only after those edits does an anonymous function/class initializer receive its
required semicolon at the original declaration boundary. Callable source text
still comes from its canonical original owner.

Module item source provenance does not participate in semantic AST equality,
consistent with existing function `LinearSpanIgnoreEq` metadata. Separate parser
controls assert actual ranges, UTF-16 coordinates, ASI boundaries, import
attributes and the distinction between declarations and default expressions.
The old tests enumerating handwritten slash-state transitions were retired
alongside that implementation. Actual regex, template, line-terminator and
module-execution regressions remain.

Assignment cover conversion now delegates grouped simple targets to the same
`AssignTarget::from_expression_simple` authority as ordinary assignments.
Parenthesized identifiers, property references, rest targets and `super`
references retain normal evaluation order; parenthesized patterns, calls,
optional chains and strict-mode `eval`/`arguments` targets remain syntax errors.
Controls cover Script/Module parsing, semantic IR and actual emitted Wasm.

The first full IR run exposed missing semicolon validation on named imports.
Every import form now passes through the same semicolon/ASI check after its
attributes; previously only bare imports did so. Tests retain explicit
semicolons, EOF and all relevant line-terminator boundaries, while rejecting
same-line trailing declarations.

The frozen 2026-10-11 checkpoint passes:

- 217 frontend checks and 1,500 IR unit checks;
- 11 focused IR integration checks and all-feature/all-target type checking;
- 38 native checks for assignment targets, module definitions and import jobs;
- the default CLI build and all 191 exact product fixture IDs over 190 files,
  including all 187 Wasm-safe IDs, with zero failures or timeouts.

The fixture replay completes in 198 watched seconds with four case workers,
one compiler worker per case and the original 60,000-ms case limit. Cargo and
libtest stay serial under the inherited 32-GiB/four-CPU cloud limits. The
[compact receipt](parser-module-ranges-20261010.json) records source, command,
log, executable and snapshot identities. Raw logs remain in
`target/parser-module-20261010/` and `target/watched/`.

The earlier static-only native check exposed a dynamic-scanner failure on a
valid regex after a block. Its red receipt is retained; the final checkpoint
passes that same control and adds Script/Module methods named `import`, nested
calls, phase-head line terminators and tracked default-declaration insertion.
The three wrong-source scanner checks are now one source-ownership control
with three inputs; actual import rewriting is checked against the parser.

Workspace formatting and architecture/task/accounting guards pass. The vendored
import parser has five pre-existing formatting hunks; a formatter comparison
against the base revision proves this patch adds none. The other modified vendor
files pass their standalone formatter checks.

This checkpoint does not establish complete workspace or pinned Test262 closure.
Unfinished task records remain in place until their acceptance is established.
