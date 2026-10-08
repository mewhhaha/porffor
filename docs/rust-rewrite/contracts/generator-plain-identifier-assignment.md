# Plain generator Identifier assignment

Ordinary generator plain assignments retain their actual Identifier Reference
before evaluating the RHS. One closed `IdentifierReferenceCaptureAccess` argument
is required by every capture constructor. `WriteOnly` publishes the same native
GC record as compound/logical `ReadBeforeRhs`, with no target GetValue. The
write-only lowering owner has no old-value accessor. Compound and logical
assignment keep their existing old-value cell and early GetValue behavior.

`generator_plain_assignment.rs` owns the ordered capture, complete staged RHS
and consuming `PutCapturedReference`. The parent value and discarded expression
routes and the expression statement route all use that owner. Multiple operands,
templates and classic-loop regions use the existing staged RHS producers; they
do not contain a second plain-assignment implementation. Linear-only iterator
and async-generator routes keep their existing admission and transport.

The Reference selects the actual runtime Environment Record, declarative cell,
ordered with object or global fallback once. Selection can throw from HasBinding
or unscopables before the RHS. Target getters never run for plain assignment.
The saved cell's TDZ and immutability and an original strict unresolvable
Reference are checked at PutValue after the RHS. An RHS abrupt therefore takes
precedence over those write failures. A sloppy original unresolvable Reference
writes the global object, even if a with object acquires that name while the
generator is suspended. Put never repeats Reference selection.

The actual native `CAPTURED_IDENTIFIER_REFERENCE` field retains the same record
and whole base across suspension. Existing consuming restoration clears that
edge before PutValue, and existing invocation cleanup clears abandoned
References before catch/finalizer transfer. Yield and delegated return with
`done: false` keep the pending Reference. No GC schema, object model, independent
lookup or cleanup algorithm is added. An opaque Object Environment setter may
mutate the returned RHS object, so the result preserves its whole value and kind
while dropping heap-shape facts across PutValue.

Two new IR controls cover nine global/declarative/with/runtime source cases,
paired owned Reference capture/Put order and complete RHS suspension counts.
The existing read controls explicitly retain `ReadBeforeRhs`. Two new Engine
cohorts cover strict/sloppy writes and sloppy with writes: omitted getters,
delayed TDZ/constant/unresolvable failures, original References after domain
changes, multiple yields, template/runtime/iteration consumers, setter mutation,
GC while suspended, whole RHS/setter/selection abrupts, yielding handlers and
finalizers, and retained delegated completion. These source controls are
authored and unrun. Isolated Rust formatting is the only executed tool affecting
this packet; no compilation, test, runtime, export or conformance result is
claimed.
