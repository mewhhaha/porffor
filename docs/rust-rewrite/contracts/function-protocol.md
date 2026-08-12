# Function protocol: one reachable call/construct/class domain

## Decision

Function metadata carries one `FunctionProtocolIr`. It does not independently
store a function flavor, an execution kind, a constructable flag and a class
role.

Those four fields described an 80-row Cartesian product even though source and
generated lowering use only these rows:

| Protocol | Flavor | Execution | `[[Construct]]` | Class role |
| --- | --- | --- | --- | --- |
| `OrdinaryCallOnly` | ordinary | ordinary | absent | none |
| `OrdinaryCallAndConstruct` | ordinary | ordinary | present | none |
| `Arrow` | arrow | ordinary | absent | none |
| `Generator` | ordinary | generator | absent | none |
| `Async` | ordinary | async | absent | none |
| `AsyncArrow` | arrow | async | absent | none |
| `AsyncGenerator` | ordinary | async-generator | absent | none |
| `ClassConstructor` | ordinary | ordinary | present | constructor |
| `ClassMethod(k)` | ordinary | any execution kind `k` | absent | method |
| `ClassGetter` | ordinary | ordinary | absent | getter |
| `ClassSetter` | ordinary | ordinary | absent | setter |

The enum is the stored truth. Its exhaustive projections provide the older
flavor, execution, constructability and class-role views where an algorithm
needs one axis. Adding a new function family therefore requires choosing all
four properties in one match; it cannot accidentally inherit a raw default.

This rejects combinations that have no ECMAScript source or compiler-generated
meaning: constructable arrows and resumable functions, generator accessors,
arrow class methods, non-constructable class constructors, and a class role on
an unrelated ordinary function.

## Boundaries that stay separate

`FunctionSignature::callable` is a lowering capability, not the ECMAScript
`IsCallable` result stored on a function object. Accessor definitions use it to
prevent a property-call fast path from treating the accessor body as the
method's value, while the accessor function object itself remains callable
when obtained through its descriptor. It therefore does not belong in
`FunctionProtocolIr`.

`ClassElementExecutionKind` is also orthogonal. Field initializers and static
blocks execute in class context but are not constructors, methods or
accessors. They use `OrdinaryCallOnly` plus their existing class-element
execution witness.

## Prototype materialization is not constructability

The semantic `[[Construct]]` capability controls the function-object runtime
flag. It must not double as a request to allocate the default ordinary
function `prototype` object.

The GeneratorFunction, AsyncFunction and AsyncGeneratorFunction constructors
are semantically constructable, but realm bootstrap supplies their
`prototype` properties explicitly. The Wasm emitter therefore carries a
backend-private, two-state materialization policy:

- automatic materialization follows the semantic protocol;
- bootstrap-supplied materialization skips only the automatic property work.

The second state does not alter the runtime constructable flag. This removes
the prior sequence that temporarily marked those constructors
non-constructable and repaired their runtime flags after allocation.

## Construction and consumption

- Syntax analysis selects a protocol variant directly from the closed AST
  function kind. It never assembles the four projections.
- Generated class lowering accepts the protocol selected by the class element
  classifier. Constructor, method and accessor tuples cannot be passed as
  independent arguments.
- `FunctionIr`, lowering signatures and `WasmFunctionMeta` carry the same
  protocol value.
- Call, construct, resumable and class consumers use exhaustive protocol
  projections. Raw compatibility fields are not retained beside it.

The existing call/construct algorithms, runtime flag values and function
prototype identities are preserved. This seam changes which metadata states
can be built; it does not broaden supported dynamic Function construction,
proxy behavior, async/generator execution or class semantics.
