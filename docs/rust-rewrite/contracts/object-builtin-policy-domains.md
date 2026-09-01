# Object builtin policy domains

Status: implemented, capability-hardened and verified through Batch AN.

## Closed domains

`EnumerableOwnProperties`, `IntegrityTest` and `PrototypeLookup` are the exact
compiler domains for six Object builtins. They implement no clone, copy, debug,
equality, ordering, hashing or default capability. The standard-builtin
dispatcher sees only six fixed semantic operations. The three domains and raw
compilers live in the private `builtins/object/enumerable_own_properties.rs`,
`builtins/object/integrity_test.rs` and
`builtins/object/prototype_lookup.rs` owners, whose wrappers are the only
variant producers. Each compiler borrows its one policy through every exhaustive
semantic decision.

`EnumerableOwnProperties` has two independent exhaustive decisions. The first
selects the nullish diagnostic for `Object.entries` or `Object.values`; the
second emits either a key-value pair or the value itself. The shared key
snapshot, enumerable-descriptor recheck and live `Get` remain outside those
decisions, and the value is read before the result projection.

`IntegrityTest` exhaustively selects whether descriptor writability matters.
Both operations reject extensible objects and configurable properties;
`Frozen` additionally rejects a writable data property, while `Sealed` has no
writability branch. `PrototypeLookup` exhaustively selects the `get` or `set`
descriptor field before the shared prototype traversal.

Adding a variant therefore requires defining every semantic decision it
affects. The policy cannot be copied into a second decision path, and this
boundary has no Boolean projection, equality comparison, wildcard arm, debug
assertion or unreachable invalid-operation path.

## Durable regression

`object_builtin_policy_domains_structure.rs` owns the three exact
capability-free variant sets, six private semantic wrapper producers, four
borrowed exhaustive semantic projections and the absence of policy escape
hatches. Its exact parent, three children and dispatcher witness also excludes
all raw private domains and compilers from the Object parent and standard
dispatcher. The finite CLI fixture distinguishes Entries from Values, Sealed
from Frozen and getter lookup from setter lookup, including the two nullish
EnumerableOwnProperties paths.

```sh
cargo test -p lila-aot-wasm --test object_builtin_policy_domains_structure --quiet
cargo test -p lila-cli --test cli object::run_wasm_backend_preserves_object_builtin_policy_domains -- --exact --test-threads=1
cargo test -p lila-cli --test cli object::run_wasm_backend_succeeds_for_object_prototype_accessor_lookup_fixture -- --exact --test-threads=1
```

The shared semantic golden passes `2/2` in 722.99 seconds with 678 dumps. It
adds this witness plus the independent Array.fromAsync callback-Realm,
Promise-mode and Set-domain witnesses, removes none and leaves all 674 retained
dumps equal after accounting normalization. Broad Test262 verification remains
deferred.

The later capability hardening changes only Rust ownership and match borrowing.
At the 2026-08-28 Batch W checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, and the exact policy-domain CLI witness passes
`1/1`. The semantic golden was not rerun for that ownership-only hardening;
broad Test262 verification remains deferred.

Batch AL moved the exact four-line `PrototypeLookup` domain and 132-line raw
compiler into the private child. After normalizing only their former
`pub(super)` visibility, they retain SHA-256
`7ca467738a2dfd39524325c1fac34084715cbb79a46baf9a862dd54737778a57`
and `f6ba5e5158701597301fc843f203a2e3997665afc54bffefd908fbe9d866876f`.
The resulting 155-line child has SHA-256
`bf4ec5630203d7a10b6982ac101dcef9192f693f1e819e4c0e2bb79f0f06c2ec`;
the reduced 8,765-line Object parent has SHA-256
`82437af076110c4151c1a82943c93054d89c24ef0caf19b217ad946d77c0fa`.
The child contains six raw policy mentions, two qualified uses of each variant
and the raw definition plus two private calls. The parent and dispatcher contain
no raw type construction, import or emitter call and can call only the fixed
getter and setter wrappers.
The pre-extraction six-line dispatcher pair was
`868d40555f8ca71066083bb1482f33bab138dacc4cba71469f3117a11ae9d2ca`;
the equivalent fixed-wrapper pair is
`3836d2ef65fd1fa5b55647f87ff4c21b859c1ea292257e081883898881c9c259`.
At the Batch AL checkpoint, `cargo xc` is green, the structure target passes
`4/4`, and the two exact CLI witnesses pass `2/2`. No Test262 leaf or semantic
golden was required or run for this source-equivalent owner move.

Batch AM moved the exact four-line `IntegrityTest` domain and 198-line raw
compiler into its private child. After normalizing only their former
`pub(super)` visibility, they retain SHA-256
`0f81ace14c7caea6494f3c6ac21f2b0bba61ba10bb10637eb79e10a22b0f2d64`
and `7263b51a1dcfdcd4eb0bc1a1bcb6569516652347ec3b8bfe773c187bddb7bf79`.
The resulting 221-line child has SHA-256
`ad029d42fc1fdeb65ae03ac765c7186a7bd7efa8cbe7da51e932f9733ad53d93`;
the reduced 8,562-line Object parent has SHA-256
`67232c7c756062fa9eb24d83506a750e147744b087464ce77deccd4243b27cee`.
The child contains six raw policy mentions, two qualified uses of each variant
and the raw definition plus two private calls. The parent and dispatcher contain
no raw type construction, import or emitter call and can call only the fixed
isSealed and isFrozen wrappers. The pre-extraction six-line dispatcher pair was
`07007e4238218ea733b7c5396e46187bd3c835dffe91fab29cf63f237dd0eaf5`;
the equivalent formatted two-line fixed-wrapper pair is
`e3aa1143b0df626eccbb1d37f76528bd30ada2f0a3e245696e136d997e30aa04`.
At the Batch AM checkpoint, `cargo xc` is green, the structure target passes
`4/4`, and the exact policy-domain CLI witness passes `1/1`. No Test262 leaf or
semantic golden was required or run for this source-equivalent owner move.

Batch AN moved the exact four-line `EnumerableOwnProperties` domain and
309-line raw compiler into its private child. After normalizing only their
former `pub(super)` visibility, they retain SHA-256
`791b3ae06c58f2ed8ca870d44a823882e4a7a3262c0eb528d323169116c54dc4`
and `2feb0f6ab4e8fa5c68e75311a45f637fcebc811ad26aad38bd2abb7b5db7ce06`.
The resulting 338-line child has SHA-256
`8d47fee7765fbcb0691be6b4f1df1de876e662db8552b29f8599a9a2a37d7777`;
the reduced 8,248-line Object parent has SHA-256
`3229401d4da5d26395572f184167c246442d67b2c7121d79adce385b33c7b3b1`.
The child contains eight raw policy mentions, three qualified uses of each
variant and the raw definition plus two private calls. The parent and dispatcher
contain no raw type construction, import or emitter call and can call only the
fixed entries and values wrappers. The pre-extraction ten-line dispatcher pair
was `7a19a898dd952f8c90da9e61fb34d3c207f891a6bd54381267eac663dcbf09cb`;
the equivalent formatted two-line fixed-wrapper pair is
`531e8ccf0c457ec36d0ed5273dd9eb0832b81a4033c966871092987337da39dc`.
At the Batch AN checkpoint, `cargo xc` is green, the policy and Realm
structure targets pass `4/4` and `1/1`, and the exact policy-domain CLI witness
passes `1/1`. No Test262 leaf or semantic golden was required or run for this
source-equivalent owner move.

This closure does not claim the complete Object or descriptor Test262 trees.
The existing Object.entries pair-Realm guard, Proxy traversal tests and full
prototype-accessor fixture remain their respective owners.
