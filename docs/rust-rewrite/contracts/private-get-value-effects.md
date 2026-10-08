# Private GetValue effects

`lowering/property_access.rs::lower_private_get_value` constructs the actual
private read for both syntax access and ordinary private-call acquisition. It
derives result and effect information from that read's actual target and private
name. Proven data and absent-getter reads preserve their result facts. An
accessor or unknown private read observes possible source hooks and invalidates
caller facts before any following argument, optional choice or property tail is
lowered. Its target and result lose heap shapes that user code may mutate;
acquired value kinds and known function identities remain available.

Syntax access evaluates its base once, then consumes that read owner. An
ordinary private call first materializes its receiver, consumes the same read,
then refreshes receiver information from the post-Get target before arguments.
The callee and receiver still use the existing runtime roots and whole Throw
transport. Suspended member capture also reaches the syntax read owner, so its
former separate effect compensation is removed.

The generated auto-accessor backing read retains its existing direct known-data
construction. No getter can run when reading that backing slot. No second
interpreter, reference transport or continuation graph is introduced.

Initial private Value bases now enter the existing async and generator optional
Property/Call tails. The private Get completes before the optional nullish
choice. A nullish returned Value retains getter effects and skips its tail;
a getter or brand Throw occurs before that choice. The retained returned Value
supplies a later public property's receiver, independently of the original
private base. Grouping still preserves a terminal Property Reference or
publishes a terminal Call Value according to the actual source.

Private links within an optional tail, Super bases and nonordinary suspension
owners retain their explicit current boundaries. Existing ordinary-generator
base and argument admission rules remain unchanged.

Two Engine cohorts run strict and sloppy scripts through Wasm AOT. They cover
ordinary read effects, eager private-call argument order, acquired Proxy callee
and receiver roots during GC, whole getter/argument Throw, and a known private
method result. The optional-tail cohort covers async and generator getter
effects, skipped nullish tails, returned property receivers across GC, grouped
References and ordinary outer argument ordering. Existing IR positive/refusal
cohorts include the newly admitted private Value tails. These are authored
controls; compilation and execution remain pending for the combined checkpoint.
