# Error stack accessor contract

The source authority is the stage-three
[Error Stack Accessor proposal](https://github.com/tc39/proposal-error-stack-accessor/blob/4d0a932d27eb45dd44e416c33b26eb4d555f1e22/spec.emu),
commit `4d0a932d27eb45dd44e416c33b26eb4d555f1e22`. Its 5,560-byte normative
source has SHA-256
`84d9f20fc633b592a033de8d6b158299a4664b0afa193051c002f891dd33f19d`.
The pinned ECMA-262 revision does not yet contain these accessor clauses.
Pinned Test262 contains 35 physical tests, expanded to 70 execution IDs.

Only `%Error.prototype%` installs the configurable, non-enumerable accessor.
Its non-constructable getter and setter have lengths zero and one, and names
`get stack` and `set stack`. Native error prototypes inherit that accessor.
Error constructors create no own `stack` property.

The getter rejects primitive receivers in its function's Realm. A direct native
ErrorData brand test recognizes native errors, subclasses, foreign new targets
and cross-realm errors. Proxies, Error prototypes and objects inheriting from
errors do not acquire that brand. The getter invokes no receiver traps or
observable `name`/`message` reads. Its implementation-defined text is captured
internally and survives message/name mutation and ordinary property replacement.

The setter checks its receiver before accepting an actual String, without
coercion. It rejects its own Realm's exact `%Error.prototype%`, then performs
`SetterThatIgnoresPrototypeProperties`. An absent own descriptor creates a
writable, enumerable, configurable data property; an existing descriptor uses
throwing `[[Set]]`. Descriptor access, setters and Proxy traps retain their
ordering, abrupt identity and function Realms. A proxy around the home prototype
passes the identity check and reaches its own internal methods.

Wasm AOT stores captured text in a typed, immutable GC reference on the native
Error record. The unpublished constructor owner retains both the ordinary
header and text until brand publication. Every runtime error allocation also
supplies text. The optional Boa oracle captures text in its native Error data
and uses its ordinary descriptor operations. No object-property fast path or
runtime-negative exception filter substitutes for either algorithm.

Assigning an own `stack` property leaves captured text intact. Deleting that
property exposes the inherited getter again. IR prototype facts model the
accessor and its actual result kinds; the setter's catalog records synchronous
user-code effects so descriptor hooks invalidate caller facts normally.
