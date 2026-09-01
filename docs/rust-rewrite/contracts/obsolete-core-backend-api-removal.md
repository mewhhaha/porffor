# Obsolete core backend API removal

Status: implemented as a source-equivalent T02 reachability closure.

Five names across four backend owners had no external call site:

- String's self-recursive but uncalled `static_number_expr_value` projection;
- `emit.rs`'s unused 32-bit and 16-bit buffer-memory argument wrappers;
- `functions.rs`'s unused TypeError-only realm-prototype store; and
- `module.rs`'s unused standard-constructor prototype lookup.

All five identifiers now have zero recursive Rust-source occurrences. The live
String call emitters, 64-bit and 8-bit buffer memory arguments, typed
message-error store, and constructor/function global-index authorities remain
on their existing paths.

The deleted String projection's exact original eight-line source has SHA-256
`f3bc9cf6043c6d927bf0d51a9f600cf28f1c2e86291f623c47ba9406b35bc0c7`.
The two deleted buffer-memory wrappers' exact original seven-line source has
SHA-256
`6af38235bb977a2b2673f8424ea1bfa1b4fb4b958df5f4a06b9490bb8e270b48`.
The deleted TypeError store's exact original 13-line source has SHA-256
`7860a2a85f440682f332a7be0a6bee8d1a7f92eaa2de78025329ae026dd699fb`.
The deleted constructor-prototype lookup's exact original 72-line source has
SHA-256
`ceac7d89945f7aeaeff7721ff47901b0c9980405f35d05aa33396ec25aab608b`.

This deletion has no new JavaScript behavior and changes no emitted Wasm: none
of the removed Rust APIs had a product call site. It adds no Test262
materialization, capability claim or published count.

At the Batch BT checkpoint, `cargo xc` is green without the five corresponding
dead-code diagnostics, the focused absence target passes `3/3`, and the exact
String substring plus ArrayBuffer transfer leaves pass all four Wasm-AOT
executions with every failure bucket at zero.
