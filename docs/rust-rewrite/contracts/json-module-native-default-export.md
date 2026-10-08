# Native JSON module records

The host declares JSON independently of JavaScript syntax. Filesystem `.json`
records require the sole `type: "json"` import attribute. Embedded policies use
`EmbeddedModuleInput::Json` and the checked `try_new_typed` graph factory. Exact
Source Text host attribute rows keep their existing authority. Record-kind/type
contradictions reject; omitted attributes never reinterpret JSON as JavaScript.
Kind, original bytes, metadata URL and exact resolution rows participate in
compilation identity. Aliases resolving to one canonical host key share one
record, default value and namespace.

`lila_front::parse_json` owns the strict JSON grammar and its retained result.
It preserves UTF-16 code units, binary64 number bits, ordered duplicate object
entries and original source. Source-sized explicit frames handle parsing and
destruction; shared graph clones retain the admitted tree through `Arc`.
JavaScript comments, expressions, trailing commas and extra JSON roots reject.

`ModuleRecordIr` holds a private Source Text/Script/JSON data variant. JSON has
one initialized mutable default binding, no named exports and no dependency
requests. The existing canonical module activation allocates its environment
and namespace before evaluation. Compiler-private scaffold reserves the default
cell and evaluation boundary; it contains no JSON source or rewritten JSON
expression. Only the actual parsed record mints `JsonModuleValueIr`, which the
trusted position map lowers into `ExprIr::JsonModuleValue`.

Wasm evaluation uses the record's actual Realm and the original JSON object,
array and CreateDataProperty operations. Compile-time iterative traversal emits
literal data construction, with live parent/child roots retained through each
property publication. Duplicate keys overwrite data properties without moving
their original property order; `__proto__` remains an ordinary own data property.
Strings use the existing lossless UTF-16 StringPool encoding. Public JSON.parse,
Object and Array constructor bindings are not consulted. There is no emitted
JSON parser, JavaScript wrapper, alternate object model or evaluator.

Static malformed JSON rejects the importing graph. Dynamic-only malformed JSON
retains the original import-job ModuleLoad rejection, while unknown request
rows use the existing host-load TypeError route. JSON offers no proposal source
representation: static source imports reject and dynamic import.source rejects
with the existing intrinsic SyntaxError. Evaluation and namespace imports remain
distinct from that source phase.

The independent oracle uses Boa's genuine JSON synthetic module in its cached
owner Realm. Parser, IR, filesystem, embedded identity, native architecture and
Engine AOT controls are authored and typechecked. The first native attempt
rejected a private loop-cell lookup; the repaired attempt executes but observes
an undefined JSON default. Diagnostics retain the original exception and show
the default cell was initialized without reaching data evaluation. Private
module resume was overwriting the generator's committed resume point with a
completion branch target, repeating instantiation. The written repair consumes
the actual activation status and preserves its resume point. Source Text module
live-binding/error controls now pass in `tasks-scope-native5`. The unchanged JSON
fixture passes its first nine data/namespace assertions in a separate diagnostic,
then fails `instanceof SyntaxError` for malformed JSON. Trusted FunctionValue
references allocated fresh constructor functions with generic prototypes. They
now load installed constructors from the current Realm, keeping explicit builtin
demand and immunity to global replacement. `tasks-iterator-intrinsics3` passes
the original JSON control and both existing source-import intrinsic/error-category
controls, including replaced global constructors, fresh rejection identities and
distinct missing-target/parse errors. These controls do not claim conformance
counts or close other non-JavaScript source loaders.
