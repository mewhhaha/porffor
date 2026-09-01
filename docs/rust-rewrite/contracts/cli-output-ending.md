# CLI output ending authority

`CliOutputEnding::{None, Newline}` is the private compile-time authority for
whether the Lila CLI capture macros terminate one write with `\n`. The plain
`print!` producer selects `None`; both `println!` forms and both `eprintln!`
forms select `Newline`.

The domain derives no cloning, copying, debugging, equality, ordering, hashing
or default-construction capability. Both captured-output sinks accept only the
typed ending and exhaustively emit the same `write!` or `writeln!` operation.
Adding an ending therefore requires both stdout and stderr semantics instead of
inheriting one side of a boolean branch.

`crates/lila-cli/tests/cli_output_ending_structure.rs` recursively pins the
private two-row declaration, absent capabilities, exact five producers, both
typed exhaustive consumers, identical byte-ending projections and retained
lock-before-write order.

The structure target passes `3/3`, and the exact CLI help/output witness passes
`1/1`. Independent dry review found the strengthened whole-macro producer table
clean. `cargo xc`, the full formatting and diff checks, and repository boundary
checks are green.

This is source-equivalent. It does not change formatting, output routing,
capture lifetime, locking, exit status, host printing or CLI command behavior.
