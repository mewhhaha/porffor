# Direct eval Environment Records

Prepared direct eval runs through independently parsed/lowered Script thunks and
the ordinary Wasm execution path. Source-call metadata and the original realm
`%eval%` identity decide whether a call receives the caller's lexical and
variable environments. A replacement or foreign eval function uses ordinary
call semantics.

`EvalEnvironmentRoleIr` carries source binding names and physical cell slots
from Analysis. Runtime code does not recover names from generated storage names.
The ordinary environment header starts its repeated tagged cells at byte 48:
parent at 0, resumable function-body record at 8, named table at 16, entry count at 24, with-object cell at 32, and
record kind at 40. Terminal Global Environment roots retain their separate
realm/lexical-table layout; consumers check the parent before reading an
ordinary record kind.

Each named entry holds a source name, pointer to an existing tagged cell,
mutability, deletability, presence, lexical declaration conflict, and immutable
binding strictness. A named function expression's self binding is immutable with
strictness false. Lexical constants are immutable with strictness true. Cells
retain TDZ state through their tag. Existing closure and mapped Arguments reads
therefore observe the same storage as direct eval.

A resolved identifier Reference owns its source name, selected record and
receiver until the operation finishes. Assignment resolves the Reference before
evaluating its right-hand side. PutValue searches the selected record again by
name, since eval may have deleted or recreated that binding; it never searches
the whole lexical chain again. Later identifier evaluations search the current
chain, including bindings introduced by eval. With records perform HasBinding
with `Symbol.unscopables` and provide their object as the call receiver.

Sloppy direct eval creates an invocation-local lexical frame and borrows its
caller's VariableEnvironment. Strict eval creates its own variable environment.
Declaration instantiation validates required names before installing functions
or variables, retaining the existing global descriptor checks when the selected
VariableEnvironment is global. Optional Annex B copies use private admission
cells and skip rejected copies. New eval variable cells are deletable; existing
parameter and variable cells retain their original attributes. Every function
declaration allocates a fresh function closing over the eval lexical frame.

Functions with parameter expressions have distinct parameter and body records.
Defaults resolve through the parameter chain; body declarations receive new
cells initialized to undefined or a copy of the corresponding parameter value.
A function declaration overrides that copy. Sloppy eval in defaults uses a
separate variable record outside the parameter bindings. Resumable activations
retain the same body record across yield and await.

Primary algorithms:

- [FunctionDeclarationInstantiation](https://tc39.es/ecma262/#sec-functiondeclarationinstantiation)
- [PerformEval](https://tc39.es/ecma262/#sec-performeval)
- [EvalDeclarationInstantiation](https://tc39.es/ecma262/#sec-evaldeclarationinstantiation)
- [GetValue and PutValue](https://tc39.es/ecma262/#sec-reference-record-specification-type)
- [Object Environment Records](https://tc39.es/ecma262/#sec-object-environment-records)

Focused runtime coverage is in `aot_direct_eval_environment.rs`; integration and
verification are coordinated with the direct source-call/frontend batch. The
parameter-expression and function-body environments must remain distinct when
required by FunctionDeclarationInstantiation.

```sh
cargo test --release -p lila-engine --test aot_direct_eval_environment -- --test-threads=1
cargo test -p lila-aot-wasm --test environment_heap_slot_structure
```

## Shared object-environment writes

The object branch of `emit_environment_identifier_put` calls the existing
`emit_ordinary_set_result_via_helper` owner. It passes the already selected
Reference base as both target and receiver, followed by the held property key
and evaluated value. The helper does not repeat identifier resolution or RHS
evaluation. Declarative and global lexical writes retain their cell operations.

Before this call, a resolved object Reference performs `HasProperty` on its held
binding object and key, after RHS effects. A missing binding throws a
ReferenceError in strict code; sloppy code still writes to that selected object.
The check can itself throw and remains observable to a Proxy's `has` trap.
Initially unresolvable sloppy References go directly to the global object's
`Set`, while strict References remain unresolvable even if the RHS creates a
binding. This follows
[Object Environment Record SetMutableBinding](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-object-environment-records-setmutablebinding-n-v-s)
without searching the lexical chain a second time.

`OrdinarySet` returns a Boolean success result through the ordinary four-result
completion ABI. The caller retains the Reference's strictness and owns both the
unresolvable-assignment ReferenceError and the failed-set TypeError. The helper
receives the existing set-path caller-Realm argument; its abrupt completion
preserves the thrown value and propagates through the caller's active catch or
return path. These are the same helper and call protocol used by ordinary
property assignments, with no new runtime representation or helper ABI.

The inline call previously expanded the full descriptor and exotic-object set
state machine at every environment assignment. The shared helper already
outlines its receiver operations. A single plain string assignment after
`eval('')` had added 509,638 bytes to the entry function; that growth belonged to
the assignment emitter, not string concatenation or eval source compilation.

On 2026-09-08, the unchanged source from
`comma_eval_candidates_preserve_callee_identity_and_argument_effects` was
emitted and run with frozen compilers before and after the call-site change.
It uses four string compound assignments to verify the `lrsc` evaluation order
while replacing eval between callee selection and invocation.

| Artifact | Before shared call | After shared call |
| --- | ---: | ---: |
| Entry function bytes | 3,565,034 | 1,511,403 |
| Module bytes | 11,420,722 | 9,183,934 |
| Native Wasm execution | Function-size rejection | `boolean(true)` |

The source SHA-256 was
`6cc19cf9011810c55337a546d775cee7f3a8805898b3c520a0384872921c03a1`.
The before compiler SHA-256 was
`76247aff3c3ea19b5d52cc3a30647a6e78f27b3842737f16ff3c5ec90754108e`;
the after compiler SHA-256 was
`982a273643385c4b8c3c7e79c9882a81db5f5d3ba4adef6de3a4e792c666d4df`.
Local frozen-artifact evidence and its rerunnable driver are
`target/failure-review/has-property-source-gap/shared-set-comparison.json` and
`target/failure-review/has-property-source-gap/verify_shared_set.py`.
This measurement covers the reproduced function-size failure, not a general
upper bound on generated function size.

The frozen before compiler for the subsequent object-binding recheck repair
produced `has:p,get:unscopables,set:p`, omitting the required second `has:p`;
strict RHS deletion recreated the property, and a throwing second `has` trap
never ran. That reproduction is recorded in
`target/failure-review/object-environment-set/before.js` and `before.log`.
Four native tests in `aot_direct_eval_environment.rs` cover simple and compound
write order, strict deletion during RHS or unscopables access, abrupt trap
identity, and initially unresolvable References. The adjacent GetBindingValue
path already performs its own held-object presence check.

The source regression remains in the native engine suite:

```sh
LILA_MODULE_MEMORY_CACHE_ENTRIES=1 cargo test --release --locked -j2 -p lila-engine --test aot_direct_eval_call_identity comma_eval_candidates_preserve_callee_identity_and_argument_effects -- --exact --test-threads=1
```
