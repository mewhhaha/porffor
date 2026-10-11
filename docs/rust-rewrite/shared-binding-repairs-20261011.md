# Shared binding and generator repair — 2026-10-11

The complete-inventory attempt on `1ad5e2be0` runs 478 native tests: 472 pass,
six fail and none are ignored. All sixteen original five-second differential
worker controls and all 447 Test262 library controls pass. The fifteen-test
generated campaign target has nine passes and six failures. Two Cargo builds
exhaust disk before native admission, and 243 targets remain unstarted. The
[compact receipt](shared-binding-repairs-20261011.json) preserves these outcomes;
no full-suite acceptance is claimed.

The retained control-flow campaign proves a generator compiler invariant
failure. Every generator try clause owns an exit state, even without a yield.
A conditional containing such a clause previously used a stateless branch,
leaving its following region at the wrong continuation state. The source
presence check now selects the complete conditional owner. IR and actual Wasm
AOT controls cover selected/skipped clauses, a later try region, a yielding
finalizer, held returns and injected return/throw identity in both strictness
modes.

Separate fresh-cache diagnostics retain the original inputs and engine limits
but omit the parent transport deadline to measure the complete cost. The
collections case takes 5.889 seconds, including 2.727 seconds of P compilation;
the module-graph case takes 11.608 seconds, including 8.227 seconds of P
compilation. Actual JavaScript execution takes 17–18 milliseconds. These are
diagnostics, not original-deadline acceptance passes. Their largest emitted
functions are 263,996 and 535,787 bytes, dominated by repeated environment
lookup, GC projections and error paths.

ResolveBinding and GetBindingValue now call shared runtime bodies. Resolution
returns its complete completion and every held Reference edge through a typed
twelve-result ABI; callers root all results before propagating a throw. Search
still precedes RHS effects, and reads/writes retain the originally selected
record. Strict/sloppy and typeof read behavior have separate closed helper
declarations. Caller execution Realm and lexical environment remain explicit
operands. Proxy trap order, TDZ, missing bindings, global delegation refresh and
abrupt completion reuse the existing semantic owners. A body-growth control
checks actual caller-to-helper edges as well as repeated-read size.

The next verification driver uses a stable workspace package selection for
every Cargo test selector. Duplicate target names are mapped by exact Cargo
package ID, name and kind, with missing, duplicate and unexpected executables
rejected. One selector build feeds separate native inventories and verdicts.
Completed incremental caches can be retired during Cargo only after observing
the producer's exact rustc process exit, its matching successful artifact, and
the absence of a live compiler using that path. Dependencies, active compiler
caches, runtime images, native worker counts and deadlines are preserved.

Final all-target/all-feature workspace types, all 465 tooling controls, eleven
repository guards and eight regenerated Intl reproduction checks pass. The
initial boundary check caught the expanded registration file exceeding its
existing line limit; related declarations now share one registration block,
and the final check passes without changing that limit. The complete native
sweep, default/release runs and full pinned acceptance remain pending. The handoff and task directory remain until actual acceptance and
ownership migration finish.
