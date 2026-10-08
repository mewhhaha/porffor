# Boa rooted completion snapshots

`lila_spec_exec::observe_script_graph_with_module_loading_policy` and
`observe_module_graph` execute the original source through the existing host
materialization and job checkpoint. Their scalar observation counterparts use
the same retained execution owner. The entry Realm is retained before execution;
the actual normal value or opaque throw stays rooted through job draining and
graph capture. A queued failure cannot replace an established primary throw.
Parse and loader failures remain execution errors.

The adapter reads Boa's retained own descriptors, extensibility, prototype,
callable kind and Realm directly. Vendored inspection methods expose bounded
raw property enumeration, private-element presence, borrowed Symbol descriptions
and registry keys, and allocation-free BigInt decimal bounds. These methods
invoke no JavaScript internal method, getter, Proxy trap, coercion or public
reflection operation. Exact decimal admission accepts a fitting BigInt after
at most one bounded scratch digit. The complete entry/created-Realm inventory
is capped before graph copies and scans, including unreferenced created Realms.

The shared runtime driver owns graph identities, ordering, limits and wire
admission. Ordinary Objects, raw ordinary Error objects, Arrays and ordinary or
native Functions expose their public property graph. Intrinsic anchors match
retained entry/created-Realm tables after public constructor-property mutation.
Private brands/fields, HTMLDDA and other unsupported exotic values reject
explicitly. No graph claims equivalent callable behavior or closure state.
Normal/Throw is retained even when capture rejects.

Seven control families in `lila-spec-exec/src/rooted_graph/tests.rs` cover exact
number/UTF-16/BigInt boundaries; cycles, Symbol identity and raw accessors; sparse
Array length; foreign intrinsic mutation; post-job roots and primary throws;
unsupported exotics/private brands; and budget/parser failure distinctions.
They are authored but unrun pending the joined checkpoint.
