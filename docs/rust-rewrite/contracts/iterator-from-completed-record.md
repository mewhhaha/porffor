# Iterator.from completed-record ownership

The complete 2026-10-04 dry source family implements
[Iterator.from and its wrapper methods](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-iterator.from)
through three fixed standard-builtin entry delegations and one private
builtins/iterators/from.rs leaf. Existing Iterator protocol, layouts, planning
and intrinsic prototype owners remain in use.

[GetIteratorFlattenable](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-getiteratorflattenable)
first admits only Objects and primitive Strings. Acquisition gets the actual
Symbol.iterator once, retaining the original receiver even when a String must
be boxed for lookup. A nullish method selects the original input; otherwise
IsCallable and the shared Proxy-aware Call run with zero arguments. The final
Object guard follows that join. GetIteratorDirect then reads next once without
validating its callability yet.

The private non-Copy completed record is minted only after that final Object
guard and next Get. Its fields are inaccessible outside the private leaf.
Identity/prototype traversal and wrapper publication consume that record,
preventing unchecked primitive or merely observed method locals from reaching
publication. The existing defining-Realm Iterator and wrapper prototypes select
the identity walk and wrapper allocation. Prototype traps run after next Get,
and a matching intrinsic ancestry returns the original iterator.

Wrapper receiver validation uses the existing raw own-data private-field reads
and propagates native failure without observable Get or Proxy traps on an
invalid receiver. Next invokes the cached arbitrary method value, so a later
next replacement cannot change it and noncallability is deferred until Call.
Return performs a fresh ordinary GetMethod on every call. A nullish return
produces a fresh own-value/own-done result through the method's defining-Realm
Object prototype. A present return method uses the same zero-argument Call
path as next.

Both wrapper methods forward arbitrary Call results, including primitives.
They implement direct Call forwarding; the separate IteratorNext operation's
Object-result requirement remains with its other consumers. The two obsolete
wrapper-result Object diagnostics are retired after their sole producers move.
Shared general IsCallable accepts callable Proxies, including revoked Proxies
whose failure belongs to Call. Original getter, apply and prototype-trap throws
retain their identity. Algorithm-created TypeErrors use the executing intrinsic
function Realm, independently of the input, wrapper or method values.

Two finite strict/sloppy WasmAot Engine cohorts extend the existing Iterator
consumer target and preserve all five preceding controls. They cover original
object/String receivers, single lookup and cached next, valid/revoked Proxy
methods, nullish fallback, acquisition and prototype ordering, deferred
noncallability, primitive next/return results, fresh return lookup and fresh
done-object Realm, wrapper identity, invalid receivers without traps, original
foreign abrupt values and both borrowed Realm directions with saved poisoned
public constructors.

The module inventory attaches the actual private leaf and three live entry
joins. Code, types, controls and documentation are authored. The ref97 combined
all-target Rust type checkpoint passed; emitted-Wasm validation and runtime
execution remain pending.
This batch does not close other Iterator helpers, full T15 or pinned conformance.
