# Dynamic-import calls share one dispatcher authority

At 2026-09-29, Module and Script entries use the same canonical import-job
dispatcher. `rewrite_dynamic_import_calls(unit, source)` performs the lexical
scan, checks the recorded call count for each phase, and projects each site
through `dispatcher_name(unit, phase)`. The phase projection is exhaustive.

The eager Script wrapper, exported dispatcher bindings and alternate public
rewriter are deleted. The former two-variant dispatcher-reference domain and
its dedicated structure test are deleted with that alternate authority; there
is no remaining Boolean, string selector or single-variant witness to maintain.
Source-phase Script graphs stop at explicit Unsupported admission. Source-phase
Module graphs retain their separately documented driver.

Source-record constructors validate identifiers from the retained AST before
rewriting. The private linker prefix is explicitly unsupported for identifier
bindings and references, including nested parameters and escaped identifiers.
Property names, comments and string data remain ordinary source. Script records
retain an empty module environment and stay outside the Module-key map even
when their URL is loaded separately as a Module.

Focused refresh commands for this batch are:

```sh
cargo test -p lila-ir --lib modules::dynamic::tests -- --test-threads=1
cargo test -p lila-ir --lib modules::admission::tests -- --test-threads=1
cargo test -p lila-ir --test import_phase_structure -- --test-threads=1
```

The phase-aware Script rewrite regression uses a retained Script parse product.
Admission regressions cover independent valid closures, direct versus dependency
rejection timing metadata, all-rejected targets, projected entry indices, host
contradictions, same-URL Script/Module ownership and private identifier hazards.
Central all-target compilation passes. Full IR verification passes 1,555 tests
across 97 groups, with one ignored documentation example. The thirteen Script
import tests and 95 neighboring module tests pass. Broad workspace and pinned
Test262 verification remain pending. This changes Script import-job behavior
and makes no broader module-linking or Test262 conformance claim.
