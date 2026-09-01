# Annex B direct-function collection

This contract fixes the compiler-private decision to collect direct function
declarations while planning Annex B block-level function semantics.

## Closed domain

`AnnexBDirectFunctionCollection` has exactly two legal values:

- `Skip` leaves direct declarations to their existing owner;
- `Record` scans the current statement list and records its direct function
  declarations before recursively visiting nested statements.

The domain is private and non-`Copy`. The collector consumes it in one
exhaustive match with no Boolean, equality, wildcard, default or unreachable
fallback.

There are exactly six producers. The outer owner body selects `Skip`, because
its direct functions are ordinary owner declarations. An ordinary nested block
selects `Record`. A switch first groups direct declarations from all cases and
records that group once, then each case selects `Skip` while recursion finds
deeper blocks. The try body, catch body and finally body each select `Record`.

## Ordering invariants

1. The non-strict owner eligibility check still precedes all nested
   collection; strict owners return without Annex B plans.
2. `Record` scans and records direct functions before recursively visiting
   nested statements.
3. A switch aggregates all case-direct functions before visiting any case.
   Case recursion cannot record those declarations a second time, so duplicate
   declarations continue to share the switch's last block binding.
4. Try, catch and finally are visited in source execution-region order, with
   each region's direct declarations recorded before its nested statements.

The migration changes only the Rust representation of the existing decision.
The planned IR and emitted Wasm must remain byte-identical.

## Durable regressions

The structural guard pins the closed domain, sole exhaustive consumer, exact
six-producer census and the recursive ordering above. Existing IR witnesses are
`annex_b_block_functions_create_undefined_owner_bindings_and_copy_when_selected`,
`annex_b_switch_declarations_share_one_case_block_binding` and
`annex_b_copy_bypasses_a_same_named_catch_binding`. The existing CLI witness is
`run_wasm_backend_supports_annex_b_block_functions`.

Focused pinned Test262 witnesses are:

- `annexB/language/global-code/block-decl-global-init.js`;
- `annexB/language/global-code/switch-case-global-init.js`;
- `annexB/language/global-code/block-decl-global-no-skip-try.js`.

The dedicated structure target passes `3/3`, as do the three existing IR
witnesses in aggregate. The CLI witness passes `1/1`, and the three exact Wasm-AOT Test262
witnesses pass `3/3` with every failure bucket at zero. `cargo xc`, the full
formatting check and the repository boundary checks are green. Independent
review found the final capability census, switch aggregation and recursive
ordering guard clean.

## Nonclaims

This invariant does not add Annex B syntax or runtime behavior, change
eligibility or strictness rules, alter block-function initialization or global
instantiation, close the Annex B tree, refresh a conformance count, or change
any emitted artifact.
