# Active legacy function reflection

Source-authored on 2026-10-14 against `32c19796850336aaa03e5962f2e9dbeb78b5a4ec`.
This contract describes the optional policy and its authored controls. It does
not claim a native, pinned-suite or full-suite passing checkpoint.

## Policy and specification boundary

The earlier policy published an immutable own `caller` with value `null` on
every ordinary non-strict function. It published no own `arguments`, so reads
of that property reached the restricted Function.prototype accessor. This
satisfied a permitted minimal policy but left the original legacy staging
caller/arguments cluster failing.

The chosen optional extension publishes own non-writable, non-enumerable,
non-configurable data properties `caller` and `arguments` on ordinary non-strict
function declarations/expressions and corresponding dynamic ordinary Functions.
Their inactive value is `null`. During the innermost active invocation,
`arguments` is that invocation's real mapped or unmapped ArgumentsObject and
`caller` is its actual ordinary non-strict JavaScript caller, or `null` when
there is none or the immediate caller is opaque. The engine updates these
private descriptor values; user assignment or incompatible DefineProperty
still fails under the ordinary property rules.

This is an implementation-selected extension, not a requirement that every
ECMAScript implementation expose active legacy reflection. The specification's
[Forbidden Extensions](https://tc39.es/ecma262/multipage/conformance.html#sec-forbidden-extensions)
restrict adding these extension properties to the eligible legacy function
forms. Strict functions, arrows, methods/accessors, class constructors,
generators, async functions, async generators, bound functions, builtins and
Function.prototype continue to inherit the same restricted accessor behavior.
The defining realm's canonical ThrowTypeError getter/setter and the existing
strict/unmapped ArgumentsObject `callee` poison remain unchanged. In particular,
no strict function is returned through `caller`, and opaque callers are not
skipped to reveal an older ordinary caller.

Actual Function.prototype.call/apply, Reflect.apply and eval intrinsics are
transparent forwarding operations. Bound dispatch resolves the target before
entry. The exceptions use the actual ExecutableCode intrinsic identity, not
public names or the source property used to retrieve a callable. Prepared eval
Scripts have environment execution roles rather than FunctionObject entries
and therefore create no fake callable frame. User replacements for a public
`call`/`apply` property remain ordinary source functions with their own identity.
Other native calls and native constructors are opaque boundaries; their user
callbacks do not expose a builtin or an older source frame as `caller`.

## Ownership and exit ordering

One typed nullable GC global retains the actual innermost call-boundary
FunctionObject. Each source entry owns private Wasm locals saving the previous
identity. Eligible ordinary sloppy entries also retain the exact own caller
and arguments descriptors and their prior StoredValue references. Recursive
entries therefore save the outer activation's values and restore them on
completion, instead of sharing one mutable argument list or discarding an
outer activation.

The entry installs its identity before instance initialization or parameter
initializers can call user code. Arguments construction publishes the private
actual object before parameter binding or body writes can replace the source
`arguments` binding. Existing mapped environment cells, duplicate-parameter
selection, index aliasing and non-simple unmapped construction are reused.
The feature never reconstructs arguments from a JavaScript binding, a name,
a source shape, a property currently containing the function, or an estimated
stack depth.

Normal fallthrough, explicit return, uncaught throw, suspension and initializer
returns restore the saved descriptor values and caller identity. Proper tail
calls evaluate all operands and conversions first, then retire the current
source activation before dispatching the target. Native entries save/restore
the boundary around the complete typed call, including all abrupt builtin
leaves and constructors, rather than adding scattered builtin return hooks.
Caught throws and finally bodies remain in their still-active source
invocation until it actually exits. Restored global and descriptor values retain
no ended activation; an ArgumentsObject explicitly saved by user source remains
a normal live user value, with its original mapped-cell semantics.

## Authored controls and required replay

The existing native test ID
`aot_function_caller::legacy_caller_is_null_only_on_ordinary_sloppy_functions`
is retained. Its inactive-null and strict/non-ordinary poison assertions remain,
while its obsolete always-null active assertions now expect actual callers.
The added native controls cover exact original identities after property
replacement/deletion, recursive argument/caller restoration, caught throws and
finally execution, source `arguments` reassignment/shadowing, mapped writes,
non-simple unmapped poison, strict and non-ordinary barriers, native callbacks,
user replacements for forwarding properties, and nested prepared eval.

Ten unchanged pinned staging bodies are copied byte-for-byte as test fixtures
with the canonical pinned sta/assert harness. They run in their original sloppy
mode, plus the original strict execution of regress-584355, preserving all
eleven execution inputs. Required acceptance remains the actual unchanged
pinned execution replay and complete native inventory/pass retention; fixture
copies and source checks cannot establish that acceptance.
