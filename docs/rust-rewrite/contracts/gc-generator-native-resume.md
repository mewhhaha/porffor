# Native generator resume

The source draft dispatches next, return and throw through a closed
GeneratorResumeKind domain. A concrete GeneratorObject owns its typed
activation. Brand checks reject proxies; the Executing state rejects reentry.

Plain and delegated yield publishers mark STATUS as SuspendedYield. The
resume consumer uses that state to distinguish suspension from normal body
fallthrough. DELEGATE presence preserves the original delegated IteratorResult
without reading its value again. No completion target or signed auxiliary
integer carries an identity marker.

Terminal dispatch marks Completed and clears DELEGATE and RESUME_POINT before
publishing a result or propagating a whole Throw. Abrupt entry before the first
resume completes without running the body. Plain yield results use the saved function Realm. Terminal and already-completed
results use the restored caller context.

This follows [GeneratorResume, GeneratorResumeAbrupt and GeneratorYield](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-generatorresume).
Six authored Wasm-AOT engine controls cover lifecycle, pending finally
completions, GC retention, reentry, delegated identity, close ownership and
borrowed cross-Realm methods. Async entries allocate intrinsic capabilities before brand validation, retain\nwhole requests in a typed GC queue, and settle through the existing capability\nCall owner. Return resumption retains its Await and finally completion.\nNone has run. Compilation, runtime and
conformance proof remain null until the entire task source batch is complete.

## Synchronous Await setup failure

PromiseResolve can throw before reactions exist or Await suspends. The whole
setup completion now remains visible to the existing body handler. Plain Yield
and suspended-Yield Return consume that Throw into the retained generator resume
value/kind in the same invocation; they retain the active request and continue
the body. A setup Throw does not become a deferred rejected Promise, an outward
native-method Throw, or a suspended status. The helper restores its caller's
completion after the body has handled the retained resume.

A seventh authored generator control checks same-call catch ordering for plain
Await, generator Yield and Return, subsequent yielded values, GC roots and
terminal completion. It is unrun, as are the original six controls. The prior
six-control receipt remains preserved by this source successor.
