# Static typeof kind domain

`compile_typeof_payload` owns one exhaustive `ValueKind` match at the point
where a singleton inferred kind may replace runtime tag observation. Undefined,
null, Array, Arguments, Boolean, Number, BigInt, Symbol and String project to
their exact `typeof` text. Function retains its payload evaluation and HTMLDDA
observation before returning `"undefined"` or `"function"`.

Object and Dynamic deliberately project to no static result and continue into
the existing runtime tag path. The compiler therefore has no partial helper and
no `unreachable!` assertion for a kind the helper's parameter type admitted. A
new `ValueKind` cannot compile until this decision is extended.

The source-equivalent change preserves the existing distrust of calls and
runtime-backed Arguments storage, the singleton gate, Function payload
evaluation, HTMLDDA behavior and runtime tag fallback.

```sh
cargo test -p lila-aot-wasm --test typeof_static_kind_structure
cargo test -p lila-engine tests::wasm_backend_supports_typeof_core -- --exact --test-threads=1
```

The total-domain target passes `3/3`, and the exact core `typeof` engine witness
passes `1/1`. The shared `cargo xc`, formatting, diff, module-boundary and
task-plan checks are green.
