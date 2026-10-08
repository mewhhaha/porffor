# Checked mixed ForIn analysis

The complete mixed ForIn owner is selected from the actual checked source in a
FunctionBody domain. `MixedAsyncGeneratorForInOwner` carries the existing typed
ForIn source identity, and its only consumer reacquires checked source from that
same AST. An equal-shaped foreign AST cannot replace it. Eager and suspended
lawful loops use this owner; a resumable execution kind alone cannot mint it.

Foreign ForOf/ForAwait bodies and resource suffixes retain their original owner.
A source factory that rejects the original initializer or a foreign body never
widened that ancestry. Phase-free unsupported heads, including the existing
WebCompat throwing-head path, remain with their old eager/linear domain.

The original analysis creates distinct lexical records for head TDZ and each
iteration. Head closures capture the TDZ record; body closures capture the real
per-key record. Nested source Blocks and With Object Environment Records keep
their own records and existing capture-hop computation. Whole With hidden rows
are still reserved by the existing checked With path before capture slots and
hops are finalized. The ForIn owner neither invents slots nor changes binding
or Reference selection.

All three complete ForIn carriers consume one private checked storage proof.
It owns the four actual invocation cells, the original optional environments,
and the eager per-key initialization block that was validated. The sole shared
initialization validator checks original source binding/Reference evidence and
retained String key consumption. A carrier cannot retain a different raw
initializer alongside that proof. Protocol ranges and suspension tapes remain
the responsibility of each checked carrier.

Three analysis controls cover head/body/With capture cells, source identity and
captured Identifier heads, and sticky foreign/resource domains with an
independent nested callable. Existing With and Switch controls keep every test
name and source literal while changing lawful eager ForIn expectations to whole
ownership. These controls are authored source only; compilation and execution
remain deferred to the coordinated batch checkpoint.
