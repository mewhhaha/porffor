# Selected object probes — differential schema 5

Schema 5 is `selected_object_probe_print_transcript`. Its `source` is an
ordinary FunctionBody that returns `{roots, anchors}`. Both arrays contain
own data records `{name, value}`; roots may hold any ECMAScript value, while
anchors require an Object or Symbol identity. Names are unique within their
array and contain 1–64 lowercase ASCII letters, digits, `-` or `_`.

The worker admits the body with the existing frontend's isolated FunctionBody
parser, then checks the generated Script's outer dependency closure. Body
text cannot close the callback and replace the capture invocation. The parent
still decodes only native corpus data before spawning. Module loading stays
`RejectAll`, and the existing two oracle gates, pre-spawn deadline, bounded
journal and cleanup-before-publication rules apply.

The worker compiles and executes the same Rust-owned capture harness plus
probe body through the ordinary AOT and Boa Engine paths. This is compiled
JavaScript fixture/harness data, not a host interpreter or compiler special
case. Primordials are captured before the probe runs. Fresh capture records
and arrays have null prototypes so probe mutation cannot insert inherited
setters or `toJSON` hooks into graph construction. Captured reflection and
primitive conversion operations produce the JSON String completion; there
is no serialization of Rust pointers or opaque Engine object categories.
The corpus fingerprint includes the complete capture harness bytes.

Object and Symbol IDs are assigned at their first encounter: named roots,
named anchors, then a breadth-first object walk. Each object records its
observable object/Array/callable category, extensibility, prototype reference
and every own key in returned order. Complete data descriptors retain value
and all three attributes; accessor descriptors retain getter/setter identity
and both attributes without invoking the accessor. Primitive values retain
UTF-16 units, canonical Number bits including signed zero and NaN, and canonical
decimal BigInt. Symbols retain identity, description, registry key or the
closed well-known-symbol name.

`Reflect.ownKeys`, `Object.getOwnPropertyDescriptor`, `Object.getPrototypeOf`,
`Object.isExtensible` and `Array.isArray` deliberately operate on selected
values. Proxy traps and their ordered print events are observable probe work.
A trap throw, disappearing key, invalid root plan or exceeded graph budget
becomes a red `observation_contract_violated` report. It is never bypassed or
converted into object-category equality. Probes can catch the exceptions they
intend to compare and include those values among their roots.

Named Realm anchors are actual identities supplied by executed JavaScript,
such as `TypeError.prototype`. A caught Error's prototype edge is compared
against that identity, and a probe can also include the executed Boolean
comparison. The harness does not infer a Realm from a constructor name or
claim to enumerate all Realms. IDs are local to each traversal; equivalent
graphs receive equivalent numbering regardless of backend allocation order.

The producer and wire admission both enforce 16 roots, 16 anchors, 512 object
nodes, 256 Symbols, 4096 total own keys, 32768 total UTF-16 units and a 128 KiB
graph encoding. `SelectedObjectProbeGraph` is the only comparison/report
owner. Its private wire validator rejects unknown fields, dangling/noncanonical
IDs, detached nodes, duplicate own keys, inconsistent Symbols, invalid
accessors and out-of-domain values, then normalizes Number payloads once.
Worker terminal admission accepts this graph only for schema 5.

Both graphs and exact captured print transcripts must match for
`selected_object_probe_and_print_transcript_match`. Mismatches have a versioned,
length-framed stable signature that excludes backend diagnostic text. Engine
failures remain red; incomplete workers remain worker failures with no semantic
mismatch signature. Semantic equivalence remains `not_established`.

Schemas 1–4 retain their wire, fingerprints and comparison contracts. General
arbitrary Script/Module object completions remain category-only. Arbitrary
post-execution graphs, unselected Realm identities, object internal slots and
non-print side-effect logs remain explicit observation gaps. This selected
probe slice does not close all T25 generation, reduction, fuzz or performance
requirements.

The committed `differential/v5/t25-object-graph-descriptors-and-realm.json`
fixture is consumed by `selected_object_probe_uses_actual_backends_and_observable_realm_anchors`.
It compares cycles/aliases, numeric own-key order, complete attributes, a
noninvoked getter, Symbol identity, negative zero and the actual caught TypeError's prototype identity.
`selected_probe_reflection_traps_remain_observable` requires the exact Proxy
reflection transcript. `selected_probe_captures_primordials_before_intrinsic_mutation`
replaces the public reflection/JSON functions, an inherited array setter and
`toJSON` hook after capture, then requires the complete graph from both backends.
Eight graph/admission controls cover FunctionBody isolation, canonical admission,
descriptor/key/prototype/alias mismatches, Realm anchors, Number normalization,
Symbol sharing and origins, bounds, accessors, red capture failures and stable
report signatures.

The source batch has not executed these controls. After adoption, the bounded
verification cohort includes the feature-enabled selected-worker tests and
`differential::object_probe::tests`, followed by the retained differential
protocol/worker controls. The CLI entry is the existing command:

```sh
lila differential replay crates/lila-test262/tests/differential/v5/t25-object-graph-descriptors-and-realm.json --oracle spec-exec
```
