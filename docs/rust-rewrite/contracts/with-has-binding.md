# With Object Environment HasBinding

`with` evaluates its head and applies the canonical `ToObject` operation once,
before publishing the Object Environment Record. Null and undefined therefore
throw even when the body is empty. Primitive wrappers retain their identity for
the lifetime of that record, including captured records. The hidden binding and
its captures carry an Object-only kind domain. The `ToObject` emitter preserves
the executing realm and routes errors to the active catch or finally target.

`ObjectEnvironmentBindingObject` produces the typed
`SpecOperationIr::WithEnvironmentHasBinding` operation. Its two operands are the
binding object and String identifier name. It returns a Boolean or throws; it
can run arbitrary user code and cannot preserve source call-flow facts. Global
Object Records continue to use their separate, unscopables-free HasProperty
operation.

The static reference path and named environment lookup call the same Wasm
runtime helper. Its single body implements the ordered protocol:

1. Perform HasProperty on the binding object. If false, stop.
2. Get `Symbol.unscopables` with the binding object as receiver.
3. If the result has ECMAScript Type Object, get the identifier name from it
   with that object as receiver. Apply ToBoolean without coercion hooks.
4. Return false for a truthy exclusion, and true otherwise.

The Type Object check uses runtime object tags. Callable and HTMLDDA exclusions
participate; primitive exclusions are ignored without boxing. Every getter and
Proxy trap can throw its original value. The helper receives the trusted caller
realm projection in ABI slot 6 and returns the standard completion tuple.
Normal calls preserve the caller's previous StatementList value.
Non-callable Proxy `get` traps use that same executing-realm projection,
including transparent nested proxies and proxies found in a prototype chain.

HasBinding selects the reference before an assignment RHS or selected getter
runs. GetBindingValue and SetMutableBinding retain their independently
observable HasProperty rechecks. Mutating unscopables or deleting a property
does not restart resolution or change an already selected object.

The old per-reference lowering expanded generic HasProperty, property access,
temporary bindings, typeof, equality, and Boolean expressions. Frozen-main
measurements added 92,151 bytes for nine extra reads with fully local object and
fallback bindings. The emitted-body regression checks the same one-versus-ten
probe against a 45,000-byte growth ceiling. Exact admission of the five large
baseline With fixtures remains a native validation result, not an inference
from this bound.

Nested writes also previously expanded array index and length dispatch at
every generic property write, even though the shared object-write helper
already owns those semantics. The generic dynamic-key path now calls that
helper after evaluating the key and RHS. Array setters, descriptors, length
conversion, Arguments, typed arrays, and Proxy writes retain one dispatch
owner. An abrupt write copies the thrown value into the expression's output
locals before propagation. A separate nested-assignment size regression guards
the frozen 538,272-byte growth for nine extra writes; its 180,000-byte ceiling
does not substitute for native validation of the large baseline fixtures.

Validation lives in `aot_with_has_binding.rs`, the With entry IR test, the
existing reference-plan ordering tests, the deep temporary-local planner test,
and the emitted-body growth test. This change removes no Test262 cases and adds
no harness omission or source-shaped optimization.
