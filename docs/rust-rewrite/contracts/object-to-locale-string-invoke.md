# Object.prototype.toLocaleString invocation

The private `builtins/object/object_to_locale_string_invoke.rs` owner implements
GetV(this, "toString"), IsCallable, and Call(method, this). GetV boxes for lookup
in the executing function's Realm while retaining the original receiver. The
Call receiver remains primitive when the original receiver was primitive.

A non-Copy GetV owner retains the original receiver, boxed lookup value and
whole method. Validation consumes it and publishes the only invocation owner
accepted by the final call phase. The boxed lookup root is cleared when it is
no longer needed. The call uses the actual generic GC call path and a non-null
empty argument vector; getter/call throws retain identity.

The historical source-spelling guard is retired. The authored borrowed-Realm,
primitive-this and getter-Throw control is meaningful semantic evidence only
after execution. No compile or runtime proof is inherited by the 2026-10-05
draft. Finish all-task source before confirmed-cap verification.
