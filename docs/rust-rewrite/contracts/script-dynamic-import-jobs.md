# Script dynamic imports retain the canonical module lifecycle

A Script that writes `import()` keeps its Script parse goal, declarations,
global `this`, strictness and final completion. The compiler discovers its
finite module targets ahead of time and records them in the same canonical
module graph used by a Module entry. Discovery never authorizes evaluation.
Uncalled branches and functions can name compiled targets without running
their bodies, and nested dynamic requests remain idle until their import jobs
execute.

Imported owners use strict module activations with their own environments,
live export cells and cached namespaces. Module-root `this` and arrows that
capture it evaluate to `undefined`; ordinary function calls retain their own
receiver and arrows inside those calls capture that receiver. Root Script
`var` and function declarations remain global declarations, lexical bindings
remain lexical, and drained import jobs cannot replace the Script's final
normal completion. Private compiler module owners stay outside the visible
global own-key set and direct-eval binding lookup before and after import jobs.

The Script root is outside the host module cache. Importing its own filename
loads that file with the Module goal into a distinct owner, even when the exact
same source is valid in both goals. The Module's declarations cannot overwrite
the Script's globals, its root arrows capture `undefined`, and later imports
reuse its evaluation and namespace. In-memory harness text is not substituted
for the file's Module source.

Each import call returns a fresh promise from the calling realm's intrinsics.
Repeated calls share module evaluation and namespace identity. A completed
evaluation failure, including an object or `undefined`, is cached without
coercing or replacing its reason. Static dependencies, live-binding updates,
cycles and top-level await flow through the canonical lifecycle before the
ordinary import fulfills. Public `Promise`, `.then`, reflection and error
constructor replacements cannot redirect internal import operations.
User string-keyed property data that spells a private dispatcher name remains
ordinary property data and cannot replace the compiler-owned dispatcher.
An ordinary `with` environment still resolves the source specifier, while
compiler-private dispatch operations bypass its property lookup and Proxy
`has` traps.

Callable reflection retains the exact original source. Compiler-owned byte
origins survive import rewriting, anonymous default-export terminators and
deferred wrappers; parsed UTF-16 callable spans resolve through those origins
before lowering. Functions, arrows, object methods and public/private class
methods use their original snippets, and a class constructor retains the whole
original class. User string data is never rewritten to recover source.

Admission keeps direct target load failures distinct from transitive
dependency/link failures. A direct malformed target rejects its import promise
at the load stage; a malformed dependency, unresolved export or missing
deferred dependency rejects through the dependency continuation. Such dynamic
failures become observable only when their import executes. Invalid closures
cannot run their bodies or poison a separately valid shared dependency.
Contradictory host source/resolution rows, parse-goal misuse and unsupported
source-phase capabilities remain compilation errors rather than runtime
rejection rows.

## Product evidence

`crates/lila-engine/tests/aot_script_import_jobs.rs` contains thirteen tests and
eighteen Wasm-AOT graph executions. Twenty-six standalone JS fixtures live below
`crates/lila-engine/tests/fixtures/script_import_jobs/`; they do not modify the
CLI semantic-golden corpus. The engine tests cover:

- the unchanged pinned `update-to-dynamic-import.js` and both original fixture
  siblings, with vendored `sta.js`, `assert.js`, `doneprintHandle.js` and
  `asyncHelpers.js`, in sloppy and strict Script executions;
- unreachable imports, root Script semantics and module lexical `this`;
- same-file Script and Module environments, each evaluated once, with one
  cached Module namespace in sloppy and strict Script executions;
- nested/transitive live updates, fresh import promises and namespace caching;
- exact ordinary-import reaction order for fulfillment and arbitrary rejection;
- a cyclic top-level-await graph and cached object/`undefined` failures;
- direct versus transitive rejection stages and valid shared dependencies;
- calling-realm intrinsic promises and errors after mutable public hooks change,
  including a foreign-realm specifier object;
- private dispatcher invisibility to direct eval and Proxy `with` lookup,
  alongside ordinary source specifier resolution and user property data;
- exact callable source across Script/Module imports, default exports, classes,
  Unicode source positions and ECMAScript line terminators.

The primary engine command is:

```sh
cargo test -p lila-engine --test aot_script_import_jobs -- --test-threads=2
```

The minimum pinned real-suite cohort has seven physical cases and fourteen
ordinary sloppy/strict executions. These paths are relative to
`language/expressions/dynamic-import/`:

| Case | Obligation |
|---|---|
| `update-to-dynamic-import.js` | Nested import remains idle until the exported function calls it. |
| `always-create-new-promise.js` | Each call creates the intrinsic promise. |
| `eval-rqstd-once.js` | Concurrent and later imports evaluate one canonical owner once. |
| `import-errored-module.js` | A later import receives the cached evaluation failure. |
| `usage/nested-arrow-assignment-expression-eval-gtbndng-indirect-update.js` | An imported namespace observes live binding updates. |
| `usage/nested-if-import-then-eval-script-code-host-resolves-module-code.js` | Script declarations remain valid while importing module syntax. |
| `catch/nested-while-import-catch-instn-iee-err-circular.js` | Indirect-export linking failures reject the import. |

On 2026-09-29, all thirteen Script tests passed, including both callable-source
Script modes. All 95 neighboring module tests and seventeen eval-environment
tests passed. Full IR verification passed 1,555 tests across 97 groups with one
ignored documentation example, and all-target checking passed. Broader workspace
integration and the real-suite cohort remain pending.
No Test262 aggregate, entire dynamic-import directory closure or T12 completion
is claimed by writing this contract or these regressions.
