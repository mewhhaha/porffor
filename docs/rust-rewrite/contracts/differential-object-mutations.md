# Generated object mutation sequences

Status: source and controls authored on 2026-10-06; compilation, replay and
campaign acceptance remain unrun.

`object-mutations-v2` extends the actual object graph generator with 1–64 ordered
operations over 1–15 Object/Array nodes. The existing `object-probe-v1` constructor,
random draws, emitted source and case identity remain unchanged. V2 uses the
same initial descriptors, accessors, string/Symbol keys, prototypes and aliases,
then draws a closed sequence of Reflect Get, Set, DefineProperty, DeleteProperty,
Has, SetPrototypeOf, PreventExtensions, Object seal/freeze and Array length
definitions. Receiver identity is explicit for Get/Set. Prototype cycle attempts
and rejected property changes retain their actual Reflect Boolean outcomes.

The private checked program validates every target, receiver, prototype, value
and Symbol reference. Array length operations require a real Array node and a
bounded integral length; ordinary Set/Define rows cannot accidentally supply a
foreign length value. Initial prototype edges remain acyclic. Runtime prototype
changes may attempt cycles, leaving the real object algorithm to reject them.
There is no expected-result evaluator or compiler special case.

Each operation prints its position and kind, then stores its actual result in
one null-prototype observation object. The existing v5 captured-primordial
observer compares this root alongside the final graph and ordered getter/setter
prints. Primitive results retain exact payloads; Object/Symbol results retain
identity through the existing graph. Object results are observed in their final
state, not as historical snapshots. The extra root explains V2's fifteen-node
limit. Unexpected throws and observation failures stay red through the original
worker/report boundary.

One exhaustive reference traversal feeds operation validation, graph liveness
and node/Symbol remapping. A reducer therefore cannot delete an object still
used only as a receiver, prototype, operation target or assigned value. It can
remove operations and simplify assigned values, prototype operands and lengths,
alongside the original graph reductions. Every retained candidate must be
strictly smaller and preserve the original graph/print difference dimensions or
backend failure phase. Its grammar identity is checked before case publication.

The SDK consumes `ObjectGenerationPlan::for_grammar`; the existing campaign CLI
accepts `--grammar object-mutations-v2 --nodes N --properties N --steps N`.
The aggregate records the actual step budget. The serial PR/nightly/release
campaign script consumes this seventh grammar through the same capped worker
route. Authored controls cover operation-only aliases, remapping, foreign
references and array targets, reducer closure, CLI budget separation and actual
selected-worker replay. Proxy hooks and broader stateful builtin grammars remain
separate generation work.
