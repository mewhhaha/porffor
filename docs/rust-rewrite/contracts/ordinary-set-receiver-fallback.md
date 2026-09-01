# OrdinarySet receiver fallback

Status: implemented and structure-verified on 2026-08-27.

## Closed helper policy

The private `OrdinarySetReceiverFallback` domain owns the body-construction
distinction between the two outlined OrdinarySet helpers. `Allowed` selects
`RuntimeHelperId::OrdinarySet` and permits the final generic receiver write;
`Denied` selects `RuntimeHelperId::OrdinarySetWithoutReceiverFallback` and
forbids that write.

Exactly two runtime-helper construction sites produce those variants. Their
single typed consumer projects the helper identity and receiver behavior
together in one exhaustive match before beginning the helper body. A new
variant therefore cannot silently inherit either half of an existing helper's
policy. The later `helper_bodies` filing table is independent of that value; the
structure regression pins its two exact insertion rows in runtime-helper ID
order so a source transposition is detected.

The domain is private and derives no `Clone`, `Copy`, `Debug`, equality or
default capability. It has no projection method, `matches!` shortcut or
equality observation.
The downstream OrdinarySet emitter still accepts the projected Boolean because
that larger object-write boundary also serves independent inline callers; this
contract closes only the paired policy used to compile the two outlined helper
bodies.

## Source equivalence and witness

This migration changes only Rust compile-time policy representation. The same
two helper ids and the same `true`/`false` values reach the same calls in the
same order, so emitted Wasm is expected to remain byte-identical.

`wasm_ordinary_set_outlined_receiver.js` remains the focused product witness.
It covers inherited setter receiver identity, an explicit Symbol receiver
write, mapped arguments receiver storage and the calling Realm of an Array
length error reached through foreign `Reflect.set`.

The recursive structure regression owns the exact two-row declaration, absence
of derived capabilities, complete source mention/call census, paired projection
table, producer order, final helper-body filings, argument order and existing
CLI registration:

```console
cargo test -p lila-aot-wasm --test ordinary_set_receiver_fallback_structure
cargo test -p lila-cli --test cli object::run_wasm_backend_preserves_outlined_ordinary_set_receiver_semantics -- --exact --test-threads=1
```

The dedicated structure target passes `4/4`, and the exact CLI witness passes
`1/1`. Independent dry review found the strengthened filing-table guard and
narrowed claim clean. `cargo xc`, the full formatting and diff checks, and
repository boundary checks are green. Broad Test262 and semantic-golden
verification remain deferred.

This seam does not claim complete OrdinarySet, Reflect, Proxy, Arguments or
Array descriptor conformance.
