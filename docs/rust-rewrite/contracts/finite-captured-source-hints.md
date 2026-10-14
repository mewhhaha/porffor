# Finite source hints on captured bindings

Source discovery belongs to the analyzed Environment Record and its physical
storage key. Independently lowered function owners return captured-cell hints
to the source-discovery fixed point. Siblings sharing a spelling do not share a
key. Captures install the EnvironmentId supplied by capture analysis; shadowing
declarations replace that origin, and leaving a scope retires it. Separate
parameter/body records retain separate keys, and the existing parameter-to-body
initialization copies only the corresponding candidate hints.

Candidates remain optional compilation inputs. The existing 256-candidate
limit, deduplication and supplemental-candidate priority stay in force. Only
captured local cells and Script-global sources need cross-owner retention;
uncaptured body hints remain local to avoid unnecessary clone cost. Identifier
source discovery does not import a bare-name static-string fact from another
owner. No flow fact, closure cell, argument evaluation, ToString ordering or
complete runtime tuple match is replaced. Unknown tuples continue to report the
existing Wasm-AOT dynamic-source limitation.

The prepared regression fixture retains all eight original source strings from
`staging/sm/syntax/escaped-let-static-identifier.js`: each ordinary body must
compile, and both strict-directive variants must throw SyntaxError. Additional
controls cover unrelated same-name owners, parameter shadows, escaped block
cells, mutation through a sibling closure, separate parameter/body records,
recursive activations, live conversion effects and the existing256-body limit.

This is a prepared repair based on115279841. No Rust build or native execution
was performed while authoring the private patch. Root must run the IR controls,
actual native controls, unchanged accepted inventories and both original
Test262 execution modes before claiming the historical failure is eliminated.
