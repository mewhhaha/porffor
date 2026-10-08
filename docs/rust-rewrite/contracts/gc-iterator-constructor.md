# Iterator constructor GC entry

This source belongs to the atomic semantic GC draft. MAIN is unchanged; Rust
types, emitted Wasm, fixture parsing and runtime semantics remain unverified.
All task source precedes verification, which requires the confirmed aggregate
4096 MiB cgroup cap, swap disabled, and one worker.

The actual standard dispatcher enters a private native constructor body. The
body receives NewTarget as a whole value and borrows the actual FunctionObject
from its declared callable entry. Undefined NewTarget and SameValue with that
actual callable reject before any prototype Get. Captured handles and borrowed
Realm copies retain their own identity after global replacement.

The complete GetPrototypeFromConstructor owner supplies the prototype. It
retains the original Throw, actual semantic prototype kind, Proxy revocation,
and saved Iterator.prototype from the NewTarget function Realm when the public
prototype is primitive. Only a Normal result may allocate the existing strong
GC OrdinaryObject. Normal and abrupt returns publish one whole completion and
clear the entry's common temporary roots.

The algorithm follows the [ECMA-262 Iterator constructor](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-iterator-constructor).
Three unrun paired strict/sloppy controls cover abstract rejection and actual
identity, subclass/Proxy prototype access and whole Throws, and borrowed Realm
identity with saved fallback intrinsics. Iterator helper algorithms, prepared
Script/host ports, sparse indexed storage, and weak reachability remain separate
open source obligations. This is no task acceptance or conformance claim.
