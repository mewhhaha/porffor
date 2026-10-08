# Ordinary generator object-pattern regions

The checked object-pattern source owner consumes the complete RHS before any
pattern operation, including an eager RHS when a computed pattern key, default
or assignment target suspends. It allocates the actual guarded default regions
through the same complete expression-region authority used by value branches.
The lowerer consumes those ranges and preserves every plain/delegated Yield.

Opaque source and key factories emit the actual raw capture, ToObject and
ToPropertyKey initializers themselves. Their private reads name distinct exact
activation cells validated against the actual generated inventory. GetV and
rest operations accept only these prepared owners; arbitrary unboxed sources
and unnormalized keys cannot enter their constructors.

Each property evaluates and normalizes its PropertyName, captures its target
Reference, performs GetV once, then enters the default region only for
undefined. A nested pattern is evaluated after its acquired value/default.
Rest captures its target before CopyDataProperties and consumes the retained
normalized exclusion keys, including Symbols. The raw primitive RHS remains
the getter receiver even though lookup uses its retained boxed object.

Assignment identifiers retain the actual selected record or cell through the
existing WriteOnly captured Reference transport. Var bindings use that same
ResolveBinding owner. Lexical bindings initialize their original predeclared
storage only after the value/default completes; their DeclarationEvaluation
consumer participates in actual entry TDZ initialization. Scoped classic For
heads retain the original BoundName map and per-iteration closure cells.

Member assignment targets retain their raw base and raw computed key in
distinct activation cells. The checked PutTarget constructor accepts only
those actual inventory reads, lexical binding targets or an eager nested Array
pattern. The native consumer reuses the original prepare/Put body: nullish
target refusal precedes key conversion at PutValue, after the property read and
default. Private writes preserve the original brand checks. No single pending
property-reference field is reused and no target selection is replayed.

GetV, CopyDataProperties and final target Put share their physical native
owners with ordinary destructuring. The result of assignment remains the
whole original RHS, with mutable heap facts discarded. Original anonymous
default AST nodes retain the parser's NamedEvaluation information. Abrupt
completion uses the existing pending whole Completion and captured-reference
retirement; normal suspension retains the original activation roots.

Array-owned Yield still needs a retained IteratorRecord/close foundation and
remains unsupported. Eager nested arrays retain the existing actual iterator
consumer. Async generators, iterator-owned LinearOnly regions and suspended
Super targets remain explicit separate boundaries.

Three source controls exercise exact acquisition/Reference/Get/default/Put
order and ranges, deferred raw Member keys, original classic For cells and
honest exclusions. A paired strict/sloppy Wasm fixture covers primitive
receivers, Symbols, lazy defaults, Proxy/rest order, TDZ, nested patterns,
closure cells, GC and injected whole Return/Throw. Anonymous class labels are
tested separately from explicit inner class bindings, including outer lexical
TDZ and an existing outer var value. A separate sloppy fixture checks that
With-object and global target selection survives getter mutation and resumes.
The batch is source-authored;
compilation and runtime verification remain mandatory and have not run.
