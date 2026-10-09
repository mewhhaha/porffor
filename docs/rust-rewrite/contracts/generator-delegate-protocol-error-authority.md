# Generator delegate protocol-error authority

`GeneratorDelegateProtocolError` is the private four-row authority for the
TypeErrors raised directly by synchronous and asynchronous `yield*` protocol
checks: non-iterable target, iterator method result not an object, subsequent
iterator result not an object, and missing `throw` method. The domain derives
no cloning, copying, equality, debugging or default capability.

Eight current producer sites name one row. The object-result check accepts the
domain, and the direct failure paths invoke the same typed emitter. That
emitter owns the exhaustive row-to-message projection and the sole raw runtime
error emission in the delegation module. Adding a row without a diagnostic
fails exhaustiveness, and a producer cannot supply an arbitrary diagnostic.

Method callability, callable proxies and their Realm-correct TypeErrors flow
through `emit_function_or_proxy_call_with_argv`. The GC migration consolidated
those checks into the ordinary checked Call owner. They are not four unused
protocol-error variants or independently emitted callability checks.

The source guard pins the exact four-row declaration, eight-producer census,
typed object-result boundary, canonical Call edge, and one occurrence of each
diagnostic in a wildcard-free projection. Current compilation, artifact and
native verification is pending. The restoration changes Rust authority only:
it retains messages, completion routing and emitted instruction order.

At the earlier pre-GC checkpoint the eight-row/eighteen-producer version passed
its standalone dependency-free structure executable 4/4, rustfmt and whitespace
checks. No native or Test262 cohort ran for that historical invariant-only
follow-up. Those results do not verify the current four-row GC implementation
or imply broad T15 completion.
