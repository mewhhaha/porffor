# Static typeof kind domain

`compile_typeof_payload` owns one exhaustive `ValueKind` match at the point
where a singleton inferred kind may replace runtime tag observation. Undefined,
null, Array, Arguments, Boolean, Number, BigInt, Symbol and String project to
their exact `typeof` text. A known type replaces only runtime tag observation:
the operand still evaluates exactly once before its payload is discarded and
the type string is returned. Its effects and abrupt completions remain
observable, including numeric coercions and comma expressions. Function
retains its payload evaluation and HTMLDDA observation before returning
`"undefined"` or `"function"`.

Object and Dynamic deliberately project to no static result and continue into
the existing runtime tag path. The compiler therefore has no partial helper and
no `unreachable!` assertion for a kind the helper's parameter type admitted. A
new `ValueKind` cannot compile until this decision is extended.

The existing distrust of calls and runtime-backed Arguments storage, the
singleton gate, Function payload evaluation, HTMLDDA behavior and runtime tag
fallback remain unchanged.

```sh
cargo test -p lila-aot-wasm --test typeof_static_kind_structure
cargo test -p lila-engine tests::wasm_backend_supports_typeof_core -- --exact --test-threads=1
cargo test -p lila-engine --test aot_runtime_import_reachability
```

The earlier total-domain target passed `3/3`, and the exact core `typeof`
engine witness passed `1/1`. Those results predate the operand-evaluation
repair; its source guard and runtime regressions are included above.
