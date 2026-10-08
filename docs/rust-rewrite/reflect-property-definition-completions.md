# Reflect property definitions in the GC compiler

Source authoring refresh: 2026-10-04. The atomic T05 draft replaces scalar native
Reflect with whole GC values and shared internal-method owners. Object admission
precedes ToPropertyKey; descriptor conversion follows and occurs once. Proxy
invariants, exotic storage and compatibility stay with the object owner. Normal
false Boolean and whole Throw completions remain distinct.

Private native algorithm calls retain the executing function Realm. OwnKeys
materializes a completed key List into a fresh Array in that Realm, retaining
String/Symbol order. Get/Set distinguish omitted receivers from explicit
Undefined. Apply/Construct validate call capability and newTarget before list
access. All thirteen entries remain reachable through the native dispatcher.

Existing completed-descriptor, Proxy and Realm semantic controls are retained.
Three new GC Reflect controls cover identity, order, admission and borrowed
Realms. Five obsolete scalar spelling guards and the duplicate descriptor
prototype loader are retired. These sources have not been compiled or executed.
Complete all task source before verification. Later checks require the confirmed
4096 MiB aggregate memory launcher, zero swap and serial workers.
