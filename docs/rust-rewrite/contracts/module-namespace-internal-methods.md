# Module namespace internal methods

The module linker emits a real namespace constructor in spec IR. An exact span
inventory records only namespace initializers from the generated prelude, before
user module bodies are appended. `LinkedScriptDefinitions` shifts those spans
through the strict asynchronous or Script-graph wrappers, and asserts that every
recorded initializer is consumed after parsing. Matching source text, names, or
array shapes in user code does not authorize this constructor.

`ExprIr::ModuleNamespace` carries `ModuleNamespaceModeIr::{Eager, Deferred}` and
a lowered private export table. The first table element is an evaluation closure
for deferred namespaces and undefined for eager namespaces. Remaining elements
are export-name/live-reader pairs in UTF-16 code-unit order. Ordinary compiler
closure capture and function reachability handle those readers. Construction
never invokes them and does not call observable `Object` or `Symbol` globals.

The runtime uses the canonical Object header, with a reserved namespace kind and
an Array-tagged private boxed payload. The existing tagged-pointer header layout
therefore traces the export table and its closures. There is no separate heap or
object model. Namespace objects have a null prototype, are non-extensible, and
store only their ordinary non-configurable, non-enumerable, non-writable
`@@toStringTag` property in the ordinary property table. Export readers are not
JavaScript accessors and cannot be extracted through descriptors. Eager and
deferred tags are `Module` and `Deferred Module`, respectively.

| Internal operation | Namespace behavior |
| --- | --- |
| GetOwnProperty | Reads the live exported binding and returns a data descriptor with writable/enumerable true and configurable false; binding TDZ propagates. |
| Get | Reads the same binding, including when the namespace is encountered in a prototype walk. |
| HasProperty | Tests export presence without reading an eager binding. |
| DefineOwnProperty | Reads the current descriptor, accepts only compatible attributes and SameValue values, and never writes the export binding. |
| Set | Returns false for every key. An ordinary superclass setter still runs directly; an ordinary data write with namespace receiver first performs that receiver's GetOwnProperty. |
| Delete | Rejects present exports without reading eager bindings; absent keys succeed. |
| OwnPropertyKeys | Returns UTF-16-sorted export names followed by `@@toStringTag`, preserving namespace order even for numeric-looking export names. |

Deferred key-specific methods invoke the module's existing evaluation thunk
before looking for any ordinary string key, including missing exports. Symbols
and the deferred `then` key use ordinary namespace storage without triggering
evaluation. OwnPropertyKeys always triggers evaluation, including when its caller
only requests symbols. Set, prototype and extensibility operations do not trigger
evaluation on their own.

The direct descriptor projection and HasProperty consume the closed object-kind
dispatch. Proxy Get/Set/Has/Delete/descriptor invariants therefore observe the same
namespace TDZ and descriptor values. Proxy own-key validation obtains namespace
keys and descriptors before comparing the trap's result with the non-extensible
target. Object.keys, Object.values and Object.entries share
EnumerableOwnProperties through the intrinsic Reflect own-key and descriptor
routes; for-in uses those same operations lazily.

Namespace identity is keyed by `(module, ModuleNamespaceModeIr)`, independently
of body materialization. Static imports, namespace reexports and dynamic imports
preserve the request phase. Reexporting an imported namespace produces an
indirect export, so converging export-star paths agree on the same resolved
namespace identity. Transparent Proxy definitions dispatch again after unwrapping,
and successful Object.defineProperty returns the original Proxy receiver.
Public own-key array results use the invoked builtin's realm.

Module evaluation lifecycle remains a separate boundary: deferred dependency
activation, instantiation before evaluation, ready-for-sync checks and sticky
abrupt completions require the module environment and scheduler changes tracked
by the lifecycle cohort. Separate namespace caches do not establish those
semantics on their own.

Regression coverage lives in `lila-ir/tests/module_namespace_construction.rs`
and `lila-engine/tests/aot_module_namespace.rs`. The completed baseline's 38
namespace executions and import-defer cohort remain separate verification
boundaries; implementation of these internal methods is not a claim that every
module/linker failure is fixed.
