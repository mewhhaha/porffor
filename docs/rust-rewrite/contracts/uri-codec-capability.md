# GC URI codec ownership

The six global entry methods in `builtins/uri.rs` consume one source argument
through the whole-value ToString owner, then operate only on its immutable GC
UTF-16 code-unit array. The standard dispatcher retains the six fixed wrapper
names. Private `UriBuiltin` and `UriCodecKind::{Uri, Component}` matches select
all native algorithms; no payload codec, host codec or secondary String model
remains.

The normative owners are current [Encode and Decode](https://tc39.es/ecma262/multipage/global-object.html#sec-uri-handling-functions)
and [Annex B escape/unescape](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-escape-string).
Encoding leaves the exact ASCII sets unchanged, rejects unpaired surrogates,
and emits uppercase percent-encoded UTF-8 octets. Decoding checks every percent
triplet, continuation octet, shortest form, scalar range and surrogate exclusion.
`decodeURI` preserves the original spelling and hex case of reserved ASCII
escapes; `decodeURIComponent` decodes them. Raw input units are copied directly,
including lone surrogates that are outside percent-encoded UTF-8.

One pure traversal first validates and counts output units, then fills one
exact-length `StringConstruction`. The immutable input makes the two passes
identical and introduces no repeated user hooks. No result is published on
malformed input. Output extents are checked against the selected runtime's
signed I32 GC array domain before narrowing; exceeding it remains a resource
trap, without truncation or a fabricated URIError.

Argument coercion and every native URIError preserve a complete completion.
Malformed-input errors use the executing builtin's defining FunctionContext
Realm, including borrowed methods after public error constructors are replaced.
Original getter/call throws retain identity. Source argument evaluation,
including ignored operands, remains in the shared invocation owner.

`aot_gc_uri_entries.rs` contains three finite paired strict/sloppy actual Wasm-AOT
controls for all six entries, UTF-16/UTF-8 boundary cases, malformed sequences,
unchanged prior assignment on Throw, callable Proxy coercion, complete argument
order and both called-Realm directions. Existing CLI URI/Annex B fixtures remain.
The two obsolete raw spelling/output structure guards retire with this cutover.
The earlier 2026-08-28 URI guard/CLI/type receipts describe the former payload
implementation and do not verify this GC source.

Current status is source-only: formatting and exact byte/path checks are allowed;
compilation, runtime, focused pinned tests and broad conformance remain pending
until the complete atomic GC batch is authored. No conformance counts change.
