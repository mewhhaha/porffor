# Intl.DateTimeFormat mode capability

Status: implemented as a source-equivalent T23 invariant closure.

## Closed output policy

`DtfFormatMode::{String, Parts}` is the sole Rust authority for the shared
DateTimeFormat field walk's output representation. `Clone` and `Copy` remain
required because one mode is stored in the sink and projected repeatedly.
Equality is not part of that authority: the domain derives neither
`PartialEq` nor `Eq` and has no manual capability implementation.

The former `if mode == Parts` allocation decision is now exhaustive. `String`
emits no array allocation, while `Parts` preserves the exact existing capacity
selection, array allocation and buffer load. A future mode must therefore
choose its allocation policy before the compiler builds instead of inheriting
the String path from an equality default.

## Ownership and verification

The dedicated structure target pins the exact two-variant declaration and its
required derives, the recursive twenty-mention ownership census, both variant
censuses and the complete normalized allocation projection. The neighboring
range-mode structure target continues to pin the exact two range producers,
receiver-operation mapping and result-tag mapping.

This is source-equivalent capability closure. It changes no emitted
instruction, local reservation, receiver validation, argument conversion,
result tag or output bytes, and it does not claim broader DateTimeFormat,
Intl402 or T23 conformance.

The dedicated and neighboring range-mode structure targets pass `3/3` each.
The exact
`intl402/DateTimeFormat/prototype/formatToParts/main.js` witness passes both
sloppy and strict Wasm-AOT executions (`2/2`); every failure bucket is zero and
both outcomes are `Success`. Targeted formatting of the new guard and the lane
diff check are clean, and the package-wide format check is green.

Independent review confirmed the retained `Clone`/`Copy` requirement, removed
equality capabilities, recursive census and complete allocation projection.
The coordinated workspace checkpoint passes `cargo fmt --all -- --check`,
`cargo xc`, `git diff --check`, the module boundary check and the task-plan
check; the compile retains the repository's existing warnings.

## Owner-private formatter selection

Batch BG makes the owner-private `DtfFormatMode` and owner-private `DtfFormatTimes`,
all three carrier fields, and the raw
`emit_intl_dtf_build_format_with_kind` formatter inaccessible outside
`intl_datetimeformat.rs`. The three fixed format, format-to-parts and range
paths remain the complete producer set. Other backend modules can no longer
construct a mode/carrier combination or invoke the large field walk directly.

Restoring only the former visibility reproduces the exact original seven-line
mode declaration with SHA-256
`2a3472c7bf2f6ea58fdca16c0e3aa06afb5e8d6dcbaaab9f96f5991939d8ab70`,
the original eight-line times carrier with SHA-256
`672ab8ac8cb31e3e580d22705dadd2707e3b06f4c0d7e567d56ec08c1c42cafb`,
and the original 932-line raw formatter with SHA-256
`76bff3098cbab0ded80001f3b2a4687a927045c5992f8cd31ac3f9976a471ae8`.
At the Batch BG checkpoint, `cargo xc` is green, the mode, range and receiver
structure targets pass `3/3`, `3/3` and `4/4`, and the exact `formatToParts`
leaf passes both Wasm-AOT executions with every failure bucket at zero.
This source-equivalent hardening has no new Intl behavior and does not close T23.
