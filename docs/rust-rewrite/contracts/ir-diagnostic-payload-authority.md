# IR diagnostic payload authority

`IrDiagnostic` stores one private `IrDiagnosticPayload`:

- `Rejected(EarlyErrorCode)`;
- `Unsupported`;
- `UnsupportedFeature(UnsupportedFeature)`;
- `UnsupportedParserFeature`;
- `Lowering`.

This replaces the public mutable `kind` field and the independent optional code
and feature fields. Those fields allowed a coded rejection to be relabeled as
unsupported, a lowering failure to carry a spec rejection code, or a typed
capability to drift away from `Unsupported`. None of those states can be
constructed from the new carrier.

The payload is intentionally private and non-`Copy`. Constructors are the only
producers. `kind()`, `code()`, and `unsupported_feature()` borrow it and match
all five variants without a catch-all. `phase()` and `error_type()` project
through `kind()`, retaining the single rejection-stage map owned by
`rejection_kind`. Adding a payload variant therefore fails compilation until
every classification projection chooses its semantics.

All callers use `diagnostic.kind()`. There is no public classification field to
mutate after construction, and no consumer reconstructs kind from code,
feature, or message text.

Focused verification:

```console
cargo test -p lila-ir --test ir_diagnostic_payload_structure -- --test-threads=1
cargo test -p lila-ir modules::early::tests:: --lib -- --test-threads=1
cargo test -p lila-ir modules::graph_tests:: --lib -- --test-threads=1
cargo check -p lila-ir --quiet
```

Prepared source candidates may discard an ordinary capability gap when the
runtime callable is unknown. `can_omit_for_runtime_source_candidate()` derives
that decision from the private payload. A caught parser abort retains its own
payload, keeps the frontend's unsupported classification without claiming a
SyntaxError, and survives nested optional source preparation.
