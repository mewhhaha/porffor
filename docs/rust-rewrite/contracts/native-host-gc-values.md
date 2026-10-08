# Native host GC values

The native host family uses the compiler's complete GC value and completion owners. Print evaluates each argument's ToString once in argument order, builds one exact UTF-16 string, and crosses the sole output boundary only after all conversions finish. Host GC invokes the registered native collector. Numeric parsers read UTF-16 units and share the exact ECMAScript leading-whitespace authority; parseInt retains string-before-radix conversion, while parseFloat accepts only the longest decimal/Infinity prefix.

Created Realms use the common completed intrinsic bootstrap and canonical global publisher. The new Global Environment belongs to the new Realm; caller declarations and entry-only host extensions are excluded. The host facade captures the new Realm through the actual callable factory. Duplicate raw created-Realm installers are retired. HTMLDDA remains a concrete callable with its retained exotic flag. Native AssertThrows accepts callable Proxies, retains whole abrupt values, and observes the thrown object's constructor exactly once; matching prototype ancestry cannot replace constructor identity.

Agent Start/Report text uses validated transient UTF-8 bytes, Sleep uses Number bits, and broadcast/receive transport a native shared-buffer resource with an Int32 id. A received message becomes a fresh real Array containing the fresh local SAB wrapper and id Number; the included harness supplies both entries to its callback. No JavaScript pointer or object graph crosses the agent host boundary. The private byte codec reuses the existing GC byte/String conversion owners.

The finite Engine target covers strict/sloppy source, exact structured Normal(262) and exact print chronology. Agent text/id abrupt controls stop before native threading operations; they are not a native threading claim. Root's separate included-harness component controls cover argument forwarding and message-pair unpacking. Earlier parser and created-Realm cohorts remain unchanged. All new source, types and controls are uncompiled and unexecuted while the full-task dry-source phase is active. Later verification requires the confirmed aggregate 4096 MiB limit, swap0 and serial execution.

The CLI collector witness returns a closure over function-local cyclic object,
Symbol and BigInt Array roots before collecting. A separate function throws a
distinct cyclic object; its catch binding is the only observable route while
another actual `gc()` runs. The control requires real Wasm-AOT execution, normal
undefined collector completion and unchanged reference identity through those
roots. This replaces the retired no-collector assertion; execution is pending.
