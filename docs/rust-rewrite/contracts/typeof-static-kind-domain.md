# typeof over runtime values

`compile_typeof_payload` evaluates its operand exactly once into `ValueLocals`,
then calls `emit_typeof_value` with that same complete value. Operand effects and
abrupt completions precede type observation. Inferred `ValueKind` precision,
including Object or Dynamic, does not replace this evaluation or discard the
operand's GC reference.

`emit_typeof_value` starts with `"object"` for null and non-callable heap values,
then projects Undefined, Boolean, Number, BigInt, Symbol and String runtime tags
to their exact text. It tests actual callability before publishing `"function"`:
Function objects, bound functions and the Proxy's retained call capability use
the shared callable owner. The HTMLDDA observation runs last and selects
`"undefined"`, including when that value is otherwise callable. No public
property lookup or inferred source kind substitutes for these runtime facts.

The focused source guards retain one operand evaluation, exact primitive
spellings and object default, whole-value callability, and HTMLDDA precedence.
The contract now follows the runtime owner that replaced the earlier static
`ValueKind` match; it does not require that retired optimization to return.

```sh
cargo test -p lila-aot-wasm --test structure_language -- typeof_static_kind_structure::
cargo test -p lila-engine tests::wasm_backend_supports_typeof_core -- --exact --test-threads=1
cargo test -p lila-engine --test aot_realm_modules -- aot_runtime_import_reachability::
```

The earlier static-domain target passed `3/3`, and the exact core `typeof`
engine witness passed `1/1`. Those are predecessor results, not verification of
the current guard updates. Their joined checkpoint remains pending.
