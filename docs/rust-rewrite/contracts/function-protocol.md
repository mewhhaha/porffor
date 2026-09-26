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
| `ModuleActivation` | arrow | generator | absent | none |
| `AsyncModuleActivation` | arrow | async | absent | none |
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

A synchronous module activation is compiler-private: its arrow flavor preserves
module lexical `this` and `arguments`, while the generator ABI owns persistent
environment cells and one instantiation suspension. Only exact trusted linker
positions can create this protocol; ordinary async arrows keep their source
protocol. Neither the activation function nor its generator object escapes to
JavaScript. AsyncModuleActivation uses the corresponding async ABI with closed
Allocate/Instantiate/Execute modes; it retains ordinary async source semantics,
while instantiation creates neither source effects nor await jobs. Its canonical
invocation environment remains distinct from the suspended current lexical chain.
See [module instantiation](module-instantiation.md).

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

## Legacy caller and arguments properties

Ordinary sloppy ECMAScript functions have own `caller` and `arguments` data
properties with `writable`, `enumerable`, and `configurable` all false. Their
values are `null` while inactive. During an invocation, `arguments` holds the
invocation's arguments object even when a parameter or declaration shadows the
`arguments` name; reassigning that binding does not replace the reflected
object. An unshadowed arguments binding initially refers to the same object.
The `caller` property holds the
immediately enclosing sloppy ordinary function object when that caller is
exposable. Recursive invocations save and restore both values. Reads through
ordinary property access and `Object.getOwnPropertyDescriptor` agree.

The compiler records source-function activation at body entry and restores it
on normal return, throw completion, and generator suspension. Strict functions,
arrows, methods, classes, generators, async functions, and native builtins
censor the enclosing caller. Bound-function invocation is transparent to this
stack; direct eval Script execution is transparent too. Indirect eval and Realm
Scripts are barriers. Cross-Realm reflection remains an implementation-defined
extension-policy limit and is not inferred from a shared function identity.

Sloppy source returns and escaping throws branch to one typed completion exit.
The shared epilogue restores reflection state and releases owned runtime roots,
so per-operation throw guards do not duplicate property scans. Explicit returns
skip fallthrough normalization; handlers and finally blocks retain their normal
completion routing. Dynamic proper tail calls restore the current activation
and release its owned runtime root after evaluating the callee and arguments,
before transferring to either the function or proxy call dispatcher.

Strict functions, arrows, methods, classes, generators, async functions, bound
functions, and builtins do not receive these properties and retain the inherited
`%ThrowTypeError%` accessors on `Function.prototype`. This is an
implementation-defined legacy reflection policy constrained by the
[Forbidden Extensions restrictions](https://tc39.es/ecma262/multipage/error-handling-and-language-extensions.html#sec-forbidden-extensions):
only ordinary sloppy functions may receive these properties, and reflection
must never expose a strict function. The protocol and strictness select the
allocation policy, including source-free ordinary Function construction. The
pinned Mozilla staging tests exercise the caller-frame policy described in the
[legacy-reflection proposal](https://github.com/claudepache/es-legacy-function-reflection/blob/master/spec.md),
which is not a normative ECMA-262 algorithm.
