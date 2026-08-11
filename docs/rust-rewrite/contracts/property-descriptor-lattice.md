# Contract: the Property Descriptor lattice (ECMA-262 6.2.6)

This is the short-name pointer the formalizer's document asked the encoder to
write. The full contract — measurement table, spec basis, invariant index
I1–I15, ledger LN1–LN10, mistake-class table, retrofit map, deviations, dry-run
corpus and acceptance criteria — is:

> `docs/rust-rewrite/contracts/The Property Descriptor lattice: one closed 6.2.6 type and one derived ValidateAndApplyPropertyDescriptor, replacing a raw u64 bitfield re-derived at eight sites.md`

Read that document before touching anything in this area. In particular read its
**§5.2**: the obvious fix to the `data: None` + `data_present_local: Some(..)`
pair deletes a required 10.1.6.3 step-6.a check unless the two kind-change
obligations are first split apart, which is what
`FunctionBuilder::emit_descriptor_kind_change_throw` now does.

## Where the types live

| Item | File |
|---|---|
| `DescriptorField`, `DescriptorSide`, `TO_PROPERTY_DESCRIPTOR_ORDER` | `crates/porffor-ir/src/property_descriptor.rs` |
| `Presence<T, R>`, `KnownPresence` | same |
| `DescriptorCarrier`, `SourceText` | same |
| `PartialDescriptor<C>`, `ValidatedDescriptor<C>`, `ValidateError`, `BothDataAndAccessor` | same |
| `PropertyDescriptorKind`, `DescriptorClassification<C>`, `KindTerms<C>`, `classify` | same |
| `CompleteDescriptor<C>`, `CompletionDefaults<C>`, `complete_property_descriptor` | same |
| `DescriptorSourceText<DataSide \| AccessorSide>` | same |
| `DescriptorBit`, `DescriptorWord`, `DescriptorMask`, `DescriptorFlags`, `MappedSlot` | `crates/porffor-aot-wasm/src/heap.rs` |
| `TaggedLocals`, `WasmLocals`, `StoredDescriptorKind`, `AttributeBit`, `DescriptorKindLocal<K>`, `DescriptorKindWord` | `crates/porffor-aot-wasm/src/objects.rs` |

## The six rules a later change must not break

1. **`classify` is the only derivation of 6.2.6.1/6.2.6.2/6.2.6.3.** There is no
   second `if data.is_some()`, no second `accessor: bool`, no second `bool`
   named after a kind. The seed decision in
   `emit_object_define_entry_validated` derives from the two `KindTerms`, never
   from `[[Value]]` alone — 6.2.6.2 says a descriptor carrying only
   `[[Writable]]` is a data descriptor, and a `[[Value]]` test gets that wrong.
   The one residual tiebreak, for a descriptor whose *both* sides are
   run-time-possible, is ledger row **LN9**.
2. **No `_ =>` arm in any `match` over `PropertyDescriptorKind`,
   `DescriptorClassification`, `KnownPresence`, `DescriptorField`,
   `CompleteDescriptor`, `StoredDescriptorKind`, `DescriptorSide` or
   `Presence`.** Adding a case must be a compile error at every consumer.
3. **A mask is not a word.** `DescriptorMask` and `DescriptorWord` have no
   conversion in either direction. `ACCESSOR | WRITABLE` is a legal mask and an
   illegal value, and both facts are load-bearing.
4. **Absent, Present and Runtime are three states, not two.** `Present` means
   "the field is there and the compiler knows it"; `Runtime` means "presence is
   decided when the program runs". Collapsing them re-creates mistake class M2
   and M5′. Note the asymmetry that is **not** yet typed: every 10.1.6.3 step-4
   arm treats `Present` like `Absent`, which is sound only for the internal
   defines that chose the value as well as the presence. Open ledger row
   **LN10**.
5. **`[[Writable]]` is not an attribute in the kind-agnostic sense.** The
   kind-agnostic word builders take `AttributeBit`, whose two inhabitants are
   `[[Enumerable]]` and `[[Configurable]]` — 10.1.6.3 steps 6.b and 7's
   surviving set. Widening them back to `DescriptorBit` re-opens mistake class
   M1 through the side door: `set_bit_if_nonzero(DescriptorBit::Writable, …)` on
   an accessor-seeded word stores the illegal word `ACCESSOR | WRITABLE`.
6. **A statically-true side is not an absent side.** 10.1.6.3 step 6.a/7.a's
   antecedent is `statically_true OR (run-time terms)`. Deciding whether to emit
   the check from `runtime_flags().is_empty()` alone conflates "the antecedent is
   false" with "the antecedent is unconditionally true"; the four cases are
   matched exhaustively in `emit_descriptor_kind_change_throw`.

## What remains runtime-checked

`LN1`–`LN10` in the full contract, plus the lane's own additions, are recorded in
`target/lane-notes/The Property Descriptor lattice: one closed 6.2.6 type and one derived ValidateAndApplyPropertyDescriptor, replacing a raw u64 bitfield re-derived at eight sites-theory-integration.md`,
which also carries the per-site retrofit instructions for the files this lane
does not own.
