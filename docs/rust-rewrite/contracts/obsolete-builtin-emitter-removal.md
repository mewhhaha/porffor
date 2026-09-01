# Obsolete builtin emitter removal

Status: implemented as a source-equivalent T02 reachability closure.

Three crate-visible builtin emitters had no call site in the backend:

- Date's `emit_date_time_within_day` wrapper;
- Binary Data's `emit_throw_if_shared_array_buffer` path; and
- String's zero-start
  `emit_string_match_all_global_ascii_word_iterator_from_string_locals`
  wrapper.

Their names now have zero recursive Rust-source occurrences. The live Date
positive-modulo and make-time emitters remain, immutable ArrayBuffer rejection
remains on its active paths, and the regexp path continues to call the
start-index-aware ASCII-word iterator directly.

The deleted Date method's exact original ten-line source has SHA-256
`e69fe8ffc2517b72e18a85800ae0556736ede49cf01cd12c29a563008d7d3767`.
The deleted Binary Data method's exact original 30-line source has SHA-256
`df9bc99017d1ab0080f962469ea29e263e3d59c15ba720e2eacfe099dacca563`.
The deleted String method's exact original 20-line source has SHA-256
`934f091b5e4b1e04057b0a56b51a7897dc1c2537057b748d4e4f01f411198471`.

This deletion has no new JavaScript behavior and changes no emitted Wasm: code
without a call site could not contribute instructions to an artifact. It adds
no Test262 materialization, capability claim or published count.

At the Batch BS checkpoint, `cargo xc` is green without the three corresponding
dead-code diagnostics, the focused absence target passes `3/3`, the active
ASCII-word match-all CLI fixture passes `1/1`, and the exact Date plus
SharedArrayBuffer leaves pass all four Wasm-AOT executions with every failure
bucket at zero.
