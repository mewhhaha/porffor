# Realm ID allocation

T06 reserves integer zero as the absence of a runtime Realm identity. A live
`RealmId` contains `NonZeroU64`, and its raw representation remains private, so
neither callers nor persistent Realm views can construct the absent identity.

`RealmBuilder::build` is the sole identity producer. It advances the shared
atomic counter with checked addition and fails with the exhausted value before
the counter can wrap. A successful allocation is therefore nonzero and cannot
reuse an earlier identity through integer overflow.

This boundary does not add Realm teardown, host-resource lifecycle, or complete
intrinsic allocation. Those remain separate T06 work.

## Evidence

- `crates/lila-runtime/tests/realm_id_allocation_structure.rs`
- `lila_runtime::tests::realm_builder_assigns_unique_realm_ids_in_one_agent`
