# Contract: Script-entry `import()` and computed specifiers

`import()` is legal in Script goal (13.3.10 takes `GetActiveScriptOrModule`,
which a Script satisfies). A Script that writes one is compiled together with
the module graph its calls can reach, into one artifact, with no runtime
parser, loader or interpreter.

## Script entries use the canonical module driver

A Script whose retained AST contains an `ImportCall` of any phase always gets
a graph, even when no specifier is a string literal: the call must still
evaluate its operands, create its promise, coerce the specifier and settle,
and only a compiled dispatcher does that. The backend's
`emit_dynamic_import` stub is reachable only for sources lowered without a
graph at all (a `$262.evalScript` or harness prelude text, for example).

Unless a module of the graph writes a static source-phase request
(`import source x from ...`), a Script entry uses the same
`ModuleInstantiationGraph` as a Module entry. `CanonicalGraphEntry` is the
closed two-case domain the source builder consumes:

- `Module(unit)`: the entry is one of the activations and one private
  evaluation statement starts its DFS;
- `Script { unit, strict, source }`: the Script is not a module. It owns no
  activation, no namespace and no evaluation. Its statements follow the graph
  statement in the merged program, so every module environment is allocated
  and instantiated before the first Script statement runs, and a target body
  runs only from its import job.

`ModuleExecutionEntry` carries the same distinction into the trusted
definitions, so a Script's statements after the graph are never scanned for a
private operation.

The Script keeps Script semantics:

- strictness is the Script's own. Its Directive Prologue cannot lead the merged
  text, so a strict Script's is restated first; every generated function
  (module activations, dispatchers) carries its own `"use strict"` directive;
- its declarations stay in the global scope, and its root `this` is the global
  object. `ScriptLowerer::root_this_binding_for_owner` gives code owned by, or
  lexically nested in, a module activation the Module root binding
  (`undefined`), so the merged parse goal cannot leak into module code through
  an arrow chain;
- a Script that spells the linker's `$lila$module$` prefix anywhere is an
  explicit unsupported case, because its `import()` calls name a dispatcher
  declared in its own top-level scope.

Graphs with a source-phase request keep the retained merged driver. That
driver now restates a strict Script's strictness ahead of its wrapper as well.

## A Script is not in the module map

A Script has no Module Record. The host loader does not register the Script's
key, `build_graph` does not enter it into `ModuleGraphIr::keys`, admission
projections do not merge it with a Module Record of the same key, and host
identity checks key loaded text by goal. `import('./self.js')` written in
`self.js` therefore loads the file afresh as a Module Record, which evaluates
once and independently of the running Script.

Script entries also take the dynamic-rejection partition in
`modules/admission`: a dynamic-only target that fails to parse or link rejects
its own import with a SyntaxError instead of failing the Script's compile.

## Computed specifiers

A computed specifier names nothing at compile time. The host declares what it
serves through `ComputedImportSpecifiers`:

- `Undeclared` (the default): a program that writes a computed `import()` of
  any phase is an explicit compile-time unsupported diagnostic.
  Rejecting every string at run time would silently deny a module the
  filesystem host would load.
- `Closed(spellings)`: the complete set of spellings the host serves. The
  loader requests each spelling on behalf of every module that writes a
  computed call, exactly as if that module had written the literal. In a canonical graph `discover_components` then serves
  each computed call from the referrer's whole resolution table (static
  imports, literal `import()` specifiers and declared spellings) in the call's
  phase. The runtime match is still the exact coerced string; anything else
  rejects with the existing TypeError.

`RejectAll` loading is a complete (empty) universe. The retained driver
evaluates `import()` targets eagerly, so it reports a computed call as
unsupported instead of widening its registry; a unit the host loaded only for a
declared spelling is targeted by a resolution row and never becomes an eagerly
evaluated root.

## Dynamic source phase

A dynamic `import.source()` does not keep a graph on the retained driver; only a
static `import source` declaration does. Every module this host loads is a
Source Text Module, and ContinueDynamicImport calls GetModuleSource on the
loaded module without loading its requests, which for a Source Text Module
throws a SyntaxError. The canonical dispatcher therefore rejects a matched
source-phase request with a SyntaxError before any load reaction, a target
that fails to load or parse rejects with its own SyntaxError, and an unmatched
string rejects with the host's TypeError. No module source object is created,
and the target is never instantiated or evaluated.

The Test262 host declares, per INTERPRETING.md, the `./<name>` spelling of
every file in the test's directory whose name contains `_FIXTURE`. Literal
specifiers keep resolving through the ordinary loader, including a test that
imports its own file. A computed string naming a non-fixture test file of the
same directory is outside this host's module map and rejects.

## Verification

`crates/lila-ir/tests/script_entry_dynamic_import.rs` covers the canonical
Script graph shape, strictness, root `this`, computed-specifier components,
self-import identity, dynamic-only rejection and the reserved prefix.
`crates/lila-engine/tests/aot_script_dynamic_import.rs` executes the same
shapes on Wasm AOT: lazy single evaluation and shared namespaces, Script and
module semantics side by side, operand/coercion timing, declared and
undeclared computed specifiers, self-import, failing targets and a
top-level-await target.
