# Optional property deletion

The opaque `DeleteOptionalPropertyChainIr` owns the actual target and consumes
the complete optional operation list to select its final Property operation.
It retains that operation's key, short-circuit flag and closed Strictness; the
remaining operations form its prefix. A caller cannot separately fabricate a
terminal Reference kind. Terminal Call chains remain whole Values and use the
existing value-deletion rule. Private terminals retain their syntax boundary.

The native emitter has one optional-chain pipeline in
`expressions/optional_chain.rs`. Its private terminal destination distinguishes
ordinary Get, optional callee-Reference capture and final property Delete.
Get and Delete consume the same original prefix operation bodies. Intermediate
public/private property Get, arguments, calls and retained call receivers keep
their source order. A grouped call starts the existing new short-circuit segment
and preserves argument evaluation before callability checks.

A shorted Delete sets the result to true and skips the rest of its current
segment. It also marks the receiver undefined and retires its call-Reference
flag, so a later grouped segment cannot mistake a shorted Value for a live
callee Reference. A nonshorted undefined receiver still takes its ordinary
abrupt path; it does not acquire short-circuit success.

The final property Delete retains the actual whole receiver in the existing
local-operand owner and invokes `compile_delete_property_i32`. The same native
owner used by ordinary Delete evaluates the raw key once, checks the base,
normalizes the key and calls the actual object/Proxy deletion algorithm. It
performs no terminal Get. The whole receiver stays rooted through coercion and
reentrant traps. Symbol identity, integer-index/string descriptors, strict
failure, defining Realm and whole thrown values stay with that existing owner.
No public Reflect hook implements this operation and no second object model is
introduced.

The eight existing `aot_delete_reference` controls retain their names and source
literals. Two added real Wasm controls exercise terminal accessor/Proxy Get
poisoning, method receiver identity, argument/key/coercion order, GC during
calls/coercion/traps, nullish suffix skipping, grouped-call errors, strict false
deletion, primitive string descriptors and whole abrupt identities. Suspension
source/IR admission and its generator controls belong to the separately joined
source lane.

This batch is source-authored. Compilation and runtime controls have not run.
