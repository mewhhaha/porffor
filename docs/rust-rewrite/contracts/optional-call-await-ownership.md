# Optional Call await ownership

Status: 2026-10-04 dry source implementation. Compilation, Wasm validation and
execution remain pending. T14 and published conformance results remain open.

Plain async expressions outside all loops admit optional chains containing
Property and Call links with awaited computed keys or arguments. The consumed
`CheckedAwaitedOptionalChainSource` requires the existing plain async branch
owner and rejects Call-bearing tails when loop depth is nonzero. Its closed
link domain retains each actual shorted flag, property field and argument
slice. The mandatory prefix admission walk checks this context and recursively
checks keys and arguments before consuming continuation states. Private links,
direct private/super targets, mixed Await/Yield and generator protocols retain
their boundaries.

The next link determines the retained operand. Property links consume a saved
GetValue. Call links consume the existing `EvaluatedCallReference`, keeping the
callee and its original raw receiver together. A first Call such as
`object.method?.(await argument)` uses the existing Reference capture; a
preceding selected property read supplies its actual chain directly to
`OptionalCallReferenceCaptureIr`. Neither route repeats the Get to recover
the receiver. A completed Call produces a Value; it is retained before a later
key or Call can suspend. A Call immediately following another Call has no
property receiver. Optional eval Calls retain the existing indirect route.

Each shorted link tests only strict null and undefined on the saved operand.
The skipped branch supplies undefined and bypasses the entire suffix, including
computed keys, argument evaluation, spread iteration and later ordinary links.
The selected branch alone consumes its Reference and argument source. It reuses
the existing private arm scopes, result cell, checked If state plan and erased
await fallback. Property-only eager residual tails still use the existing
optional-chain emitter.

Selected Calls use the shared suspended argument capture and completion owner.
GetValue and receiver capture precede all arguments; spread iteration is saved
before a later argument suspends. Earlier argument identities survive later
mutations while their mutable shape facts are invalidated as before. Ordinary
noncallable values evaluate the complete argument list before the existing
callability error. Argument-list spread failures retain their existing abrupt
completion behavior. Only Normal completion publishes a Call result; rejection,
arbitrary throws, intrinsic error Realm and awaited finally remain with the
existing activation owners.

Grouping ends optional shorting. A terminal Property keeps the existing
Reference capture and raw receiver; earlier same-chain Calls may precede it.
The grouped source also admits target-only Await through the same full checked
Property/Call tail. A completed awaited base precedes every synchronous tail Get;
its receiver and callee are saved before outer operands. Recursive first-Call
preflight consumes this same terminal-Property proof before argument states.
A terminal Call supplies a completed Value through the
[grouped Call Value owner](grouped-optional-call-value-await-ownership.md).
Its private actual-source terminal kind and checked full-tail consumer also own
target-only suspension, preserving a nested grouped method receiver inside the
chain while the outer Call or tag receives undefined as its receiver. Mandatory
preflight checks the complete grouped source before state allocation. The joined
callee is pinned before outer arguments or GetTemplateObject/substitutions.

Constructors retain their established Value route. Private links and Super targets
consume their original Get and receiver owners. Grouped terminal private properties
retain the actual Reference before outer operands. Delete uses a separate final
Reference destination: the skipped arm commits true and the selected arm emits the
original Delete without a terminal Get; terminal Calls still supply DeleteValue.
Unowned loop Calls and other loop heads retain their boundaries. Generator and mixed
optional regions consume the same physical tail with their own checked source tapes.

Three paired strict/sloppy Engine sources cover Proxy/Symbol/getter order,
callee replacement, strict primitive receivers, spread snapshots, optional
indirect eval with a non-string operand, complete suffix skipping, saved Call
results across later keys, grouped Call/tag receivers after preceding Calls,
grouping boundaries, foreign abrupt identity, intrinsic error
prototypes and awaited finally. They execute through the actual Wasm-AOT Engine
wrapper when verification resumes. Authored IR controls inspect the consumed
Reference/result/state ownership and the remaining source boundaries. None of
the new target-only controls has been compiled or executed for this batch.
