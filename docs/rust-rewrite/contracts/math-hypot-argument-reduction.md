# Math.hypot argument reduction

The native reduction reads the callable's private GC argument vector. It
coerces every argument in order, including arguments after Infinity or NaN;
any whole Throw branches to native cleanup. Infinity dominates NaN only after
all required coercions succeed.

The consumed `CompletedMathHypotReduction` contains typed scalar scale,
normalized sum and classification locals. Only the finisher reads that record
and selects positive zero, finite scaled magnitude, NaN or positive Infinity.
The argument vector retains GC references through each user hook.

The T05 draft is source only. Existing CLI controls and new Engine coercion and
classification controls are unrun against this source. No compilation, runtime
or conformance status is claimed.
