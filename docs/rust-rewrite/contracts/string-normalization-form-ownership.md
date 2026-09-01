# String normalization form ownership

Status: implemented for the existing normalization-form authority.

## Boundary

`StringNormalizationForm::{Nfc, Nfd, Nfkc, Nfkd}` is the sole compile-time
authority for normalization spelling, runtime branch code, decomposition table
and composition policy. It derives no cloning, copying, comparison, debugging
or default capability.

The normalization emitter owns one form. Both decomposition passes borrow that
form, and the final composition projection consumes it. The normalize
validation loop borrows each form for its accepted spelling before consuming it
for the runtime branch code. Adding a policy projection that moves the form
before its final owner now fails to compile instead of silently duplicating the
authority.

## Durable evidence

`string_normalization_form_structure.rs` pins the attribute-free four-row
declaration, the borrowed and consuming projection signatures, both borrowed
decomposition calls, the borrowed lookup boundary and the existing exhaustive
maps. It also retains the validation, dispatch, locale-compare and String-pool
authority checks.

## Verification

The focused structure target is the cheap gate for this source-equivalent
ownership change. No runtime behavior, emitted Wasm value, normalization data,
Test262 status or published conformance count changes.

## Deferrals

This contract does not complete Unicode normalization data, locale/options-aware
collation, the String API, T18 or full Test262 conformance.
