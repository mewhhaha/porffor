# `Intl.DateTimeFormat` reserves its result before initialization

## Scope

This contract owns the observable `OrdinaryCreateFromConstructor` boundary of
`Intl.DateTimeFormat`. Resolving `NewTarget.prototype` and allocating the
ordinary result happen before locale-list canonicalization, options coercion,
or any option read. The allocated object remains unreachable until its current
DateTimeFormat record and brand are installed.

This slice deliberately preserves the existing `CurrentGlobal`
default-prototype fallback. Created realms do not yet install `Intl`, so the
complete cross-Realm fallback and active-function identity rules remain
outside this contract.

## Required order

ECMA-402 `Intl.DateTimeFormat` first substitutes the active function object
when `NewTarget` is undefined. It then performs
`OrdinaryCreateFromConstructor` before `CreateDateTimeFormat` observes
`locales` or `options`. Therefore the Wasm constructor performs these
transitions in order:

1. Resolve `NewTarget.prototype`, including Proxy `[[Get]]`.
2. Allocate an ordinary object whose prototype retains the resolved value's
   complete payload and representation tag.
3. Keep that reserved object private and unreachable.
4. Canonicalize the locale list, coerce options, and perform every ordered
   option read and validation.
5. Allocate and completely populate the represented DateTimeFormat record.
6. Install the record and DateTimeFormat brand on the reserved object.
7. Publish only that initialized object.

Consequently, an abrupt prototype lookup wins over every locale or options
operation. An abrupt locale coercion happens only after prototype reservation
and prevents all later option observation. Either abrupt path can leave an
unreachable allocation behind; JavaScript cannot observe that allocation.

## Load-bearing representation

`builtins/intl_datetimeformat/construction_lifecycle.rs` owns a private-field
two-state lifecycle:

```text
ReservedIntlDateTimeFormatObjectLocal
    -> InitializedIntlDateTimeFormatObjectLocal
    -> published result
```

Both states are non-`Copy` and hide their raw local. Rust requires the carrier
names and three transitions to be `pub(super)` so the parent constructor can
pass inferred states across locale/options initialization, but the parent
cannot construct or project either state. The recursive source policy also
forbids explicit parent naming, imports and re-exports. The reserve transition
is the only constructor of the first state. It uses the shared
`emit_new_target_prototype_to_locals` operation and tagged ordinary-object
allocation, then releases the temporary prototype tag and payload in LIFO
order while retaining the result local.

The initialization transition consumes the reserved state only after the
record has been populated, installs both the record pointer and the internal
brand, and returns the only state accepted by publication. Publishing a
reserved or partly initialized object is therefore a Rust type error.

`functions.rs` classifies `Intl.DateTimeFormat` in the closed direct-returning
constructor domain. That dispatch enters the builtin body and leaves the
construct block before the generic constructor path can read
`NewTarget.prototype` or preallocate a receiver. Consequently the reserve
transition above is the sole prototype `Get` and sole result allocation for an
explicit `NewTarget`; removing the classification makes the structural guard
fail instead of silently restoring the former two-Get path.

## Owned files

This slice is closed over eight files:

- `crates/lila-aot-wasm/src/builtins/intl_datetimeformat.rs`;
- `crates/lila-aot-wasm/src/builtins/intl_datetimeformat/construction_lifecycle.rs`;
- `crates/lila-aot-wasm/src/functions.rs`;
- `crates/lila-aot-wasm/tests/intl_date_time_format_construction_order_structure.rs`;
- `crates/lila-cli/tests/fixtures/wasm_intl_date_time_format_construction_order.js`;
- `crates/lila-cli/tests/cli/intl.rs`;
- this contract; and
- `tasks/23-intl402.md`.

The dispatcher file is required: without its direct-returning classification,
generic construction would perform another prototype `Get` before entering the
builtin body.

## Durable evidence

`wasm_intl_date_time_format_construction_order.js` pins:

- a Proxy `NewTarget.prototype` read before locale `length`, element access and
  coercion, and before options access;
- unchanged propagation of a throwing prototype getter while locale and
  options remain unobserved;
- locale coercion after successful reservation and before options access;
- successful initialization behind a custom prototype; and
- exact Function, Array and Arguments prototype identity across tagged
  allocation.

The adjacent pinned Test262 obligations are
`intl402/DateTimeFormat/proto-from-ctor-realm.js`,
`intl402/DateTimeFormat/subclassing.js`, and the constructor options-order and
throwing-getter tests.

## Verification boundary

The static freeze consists of exact-file `rustfmt --check`, `node --check` for
the fixture, the recursive structural guard proving one tagged prototype
lookup, sole carrier/transition ownership and the one-way
reserved/initialized/published transition, `git diff --check`, module boundary
checks, task-ledger checks, file-scope review, and independent review.

At the 2026-08-28 Batch W checkpoint, `cargo xc` is green, the recursive
structure target passes `1/1`, and the registered construction-order CLI
fixture passes `1/1`. The adjacent pinned Intl Test262 witnesses, semantic
snapshot and broad batch ladder were not rerun; no result is claimed for them.

## Nonclaims

This contract does not add created-realm `Intl` bootstrap, the complete
cross-Realm intrinsic fallback, or the legacy `ChainDateTimeFormat` behavior.
It does not add locale, CLDR or time-zone data, another Intl service, or a
green DateTimeFormat/Intl subtree. It also does not change the current
`CurrentGlobal` fallback used when `NewTarget.prototype` is primitive.
