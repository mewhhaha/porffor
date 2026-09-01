# String well-formed operation owns its result shape

Status: implemented for the Wasm-AOT `String.prototype.isWellFormed` and
`String.prototype.toWellFormed` dispatch seam.

## Boundary

The private, capability-free
`StringWellFormedOperation::{Check, Repair}` domain owns the choice between
testing a String's UTF-16 well-formedness and replacing its unpaired
surrogates. Standard builtin dispatch cannot construct or inspect that domain.
It can call only the two named entry points that fix `isWellFormed` to `Check`
and `toWellFormed` to `Repair`.

One consuming exhaustive match owns both consequences of the choice:

- `Check` invokes the existing well-formedness scan and tags its payload as a
  Boolean;
- `Repair` invokes the existing surrogate-repair materializer and tags its
  payload as a String.

The algorithm and result tag are therefore one closed projection. Adding an
operation without defining both consequences fails to compile, and a caller
cannot pair a repair payload with the Boolean tag or a check payload with the
String tag.

Receiver validation, receiver coercion, active-function-Realm errors and the
existing UTF-16 scan and repair algorithms are unchanged.

## Durable evidence

`string_well_formed_operation_structure.rs` pins the private two-row domain,
its lack of incidental capabilities, the two exact producers, the consuming
exhaustive algorithm/tag projection and the standard dispatch boundary. It
also excludes direct low-level well-formed emitters from standard dispatch.

## Verification and nonclaims

This is source-equivalent lifecycle and result-shape hardening. It adds no
runtime mode word, fixture, Test262 claim or published conformance count. It
does not prove the complete Unicode data set, other String methods, the pinned
String tree or T18 closure.
