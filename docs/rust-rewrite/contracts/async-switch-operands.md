# Plain async Switch operands

A plain async Switch with an awaited discriminant or selector consumes the
same checked complete Switch carrier and native CaseBlock pipeline as the
existing complete owners. The actual source factory allocates the discriminant,
ordered lazy selectors, fallback decision and source-order fallthrough bodies
once. Its typed Await tape is compared against the lowered regions before the
carrier is published. Generator and mixed regions cannot satisfy this proof.

The discriminant publishes one original-invocation cell outside CaseBlock.
CaseBlock TDZ cells and hoisted functions are instantiated once before any
selector. Conditional and logical assignments, optional property and Call
tails keep their branch continuations in the selected operand region. Matching
reads each terminal value while that region's temporary scope remains active;
operand prefixes cannot overwrite an enclosing statement-list completion.
Plain async Await checkpoints an active retained statement-list value before
suspension and preserves its received value for an actual source expression;
generated operand and Empty declaration prefixes remain suppressed.

Default may appear between selectors: all reachable later selectors still run
before fallback enters its body. A successful match skips subsequent selectors;
normal fallthrough evaluates only subsequent bodies. Await rejection and whole
Break, Continue, Return and Throw use the original lexical cleanup and finalizer
transport. Body-only async switches retain their existing continuation owner.

The retained refusal literal is now a supported control. New actual source
constructor controls and strict/sloppy JS-to-Wasm fixtures cover conditional
References, optional receivers and skipped operands, default placement, captured
TDZ cells, terminal temporary lifetime, labels and awaited finalizers. They are
dry-written; no formatting, compilation or execution has run for this batch.
