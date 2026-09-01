# Reflect descriptor Object prototype Realm

Status: implemented with focused structural and semantic verification,
2026-08-28.

## Scope

This contract owns the ordinary descriptor object that
`Reflect.defineProperty` supplies to a Proxy `defineProperty` trap after
`ToPropertyDescriptor`. It does not own descriptor validation, Proxy invariant
enforcement, the target mutation, other Reflect methods, or general Realm
bootstrap.

## Rust invariant

The Reflect descriptor Object prototype is represented by the private,
non-cloneable `ReflectDescriptorObjectPrototypeLocal`. Its only producer emits
exactly two routes:

1. an entry-Realm Reflect function, identified by its zero environment handle,
   loads the entry `%Object.prototype%` global;
2. a self-backed created-Realm Reflect function loads its defining Realm, that
   Realm's intrinsic table and that table's populated `%Object.prototype%`
   slot. Missing state traps instead of falling back to the entry Realm.

The descriptor allocator consumes the prototype proof, installs the carried
local as `[[Prototype]]`, stores the new descriptor object and releases the
local. `compile_reflect_define_property_builtin` cannot pass an unrelated raw
local to that allocator, clone the authority for a second allocation, or use
the entry global at the guarded allocation site.

Created-Realm Reflect publication is part of the boundary: every method is
self-backed before it is exposed, so `current_env_local` identifies the active
method object and its recorded defining Realm rather than a lexical
environment.

## Observable behavior

The focused engine witness calls both entry-Realm and created-Realm
`Reflect.defineProperty` on Proxy targets. Each trap observes the completed
descriptor object's prototype. The entry call receives the entry
`Object.prototype`; the borrowed created-Realm method receives that Realm's
distinct `Object.prototype`.

## Verification and non-claims

The Rust-lexical guard ignores comments, nested comments, normal/raw/byte/C
strings, character literals and raw identifiers. It pins the exact authority
declaration and recursive census, both producer routes, three required-state
traps, the consuming allocator, the sole guarded call pair and the self-backed
created-Realm Reflect publication dependency.

This does not claim general Realm isolation, complete Proxy conformance,
descriptor algorithm completeness, cross-Realm error correctness, or broader
T06 completion. It changes no intrinsic registry row, Wasm ABI, host surface,
published conformance count or unrelated object allocation.

The carrier, producer and consuming allocator now belong to the private
`builtins/reflect/descriptor_object_prototype.rs` owner. Rust requires the
carrier name and both methods to be `pub(super)` because the parent retains the
inferred factory/consumer call pair, but the tuple field stays child-private.
The parent therefore cannot construct or destructure the raw proof, and the
recursive source policy forbids explicit naming, import or re-export.

The move selected the exact two-line carrier, 53-line producer and 12-line
consumer at SHA-256
`b1c715d874f23c0d210ee092b547457eead1cb42557eaff40124f4fe59ba68a0`,
`0ed08206648a1d4f58e9aa3683448dd738d85ea9000d48402e66eed4b34d74f9`
and
`ed599d485865a47e4b425a5ed23630ca3b4c1e3c5c01e18a14869dc39fac1bf2`.
Normalizing only the required `pub(super)` visibility restores each original
hash. The unchanged six-line parent call pair retains SHA-256
`770f6319489eb9aa746a3e1147f7484a550413db9c6c4bb4e8bc2da018fd40e5`.
The 73-line child has SHA-256
`30522774764563257635779bbb9c4f59639af31b98136d119cc42ca9dd38688f`
and reduces the concurrent `reflect.rs` snapshot from 2,415 to 2,347 lines.
The retargeted structure target passes `4/4`, the engine Realm witness passes
`1/1`, and the shared `cargo xc` checkpoint is green. This source-equivalent
owner move changes no published conformance count.
