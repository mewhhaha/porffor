# F032: Lower await nested in export declarations and catch flow

- **Status:** open
- **Owner:** lila-ir module and expression lowering
- **Cause assessment:** confirmed
- **Disposition:** required
- **Baseline:** 13 executions across 13 physical files (Bug 0, NotImplemented 13, Crash 0)

[Backlog](README.md) · [Exact execution list](cases/F032.executions) · [Unmodified diagnostics](evidence/failures.json)

## Root cause

Top-level-await module source reaches ordinary expression lowering with raw Await AST nodes in exported variable initializers and catch-related expressions. That path only accepts already planned suspensions and emits unsupported expression form.

## Source evidence

- [crates/lila-ir/src/lowering.rs:7635](../crates/lila-ir/src/lowering.rs#L7635): General expression fallback proves these Await nodes were not transformed.
- [test262/vendor/test262/test/language/module-code/top-level-await/syntax/export-var-await-expr-literal-number.js:5](../test262/vendor/test262/test/language/module-code/top-level-await/syntax/export-var-await-expr-literal-number.js#L5): Await is in an exported variable initializer.

## Work

Stage these expressions through the module async continuation planner before ordinary expression lowering, preserving exported live bindings and rejection handling.

## Validation

Run all attached export-var await variants and catch-parameter fixture.

Replay all listed modes with a fresh compiler and retain the native snapshots:

```sh
python3 scripts/replay-test262-executions.py tasks/cases/F032.executions \
  --binary target/release/lila --suite-root test262/vendor/test262 \
  --output-dir target/test262-scratch/F032-replay --workers 2
```

Use a new output directory after rebuilding; resume only with unchanged inputs. See the backlog resource-limit and closure rules.

## Representative observations

- `module:language/module-code/top-level-await/syntax/catch-parameter.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Await(Await { target: Literal(Literal { kind: Int(42), span: Span((48, 27), (48, 29)) }), span: Span((48, 21), (48, 29)) })
```

- `module:language/module-code/top-level-await/syntax/export-var-await-expr-array-literal.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Await(Await { target: ArrayLiteral(ArrayLiteral { arr: [], has_trailing_comma_spread: false, span: Span((70, 24), (70, 26)) }), span: Span((70, 18), (70, 26)) })
```

- `module:language/module-code/top-level-await/syntax/export-var-await-expr-func-expression.js` — NotImplemented

```text
[origin:unknown] unsupported in lila wasm-aot first slice: unsupported expression form: Await(Await { target: FunctionExpression(FunctionExpression { name: None, parameters: FormalParameterList { parameters: [], flags: FormalParameterListFlags(IS_SIMPLE), length: 0 }, body: FunctionBody { statements: StatementList { statements: [], linear_pos_end: LinearPosition { pos: 2373 }, strict: false }, span: Span((76, 35), (76, 37)) }, has_binding_identifier: false, contains_direct_eval: false, name_scope: None, scopes: FunctionScopes { function_scope: Scope { outer: Some(Scope { outer: Some(Scope { outer: None, index: Cell { value: 0 }, bindings: RefCell { value: [] }, function: true }), index: Cell { value: 0 }, bindings: RefCell { value: [Binding { name: "name1", index: 0, flags: BindingFlags(MUTABLE | ACCESSED) }, Binding { name: "x", index: 0, flags: BindingFlags(MUTABLE | ACCESSED) }] }, function: true }), index: Cell { value: 0 }, bindings: RefCell { value: [Binding { name: "arguments", index: 0, flags: BindingFlags(LEX) }] }, function: true }, parameters_eval_scope: None, parameters_scope: None, lexical_scope: None, mapped_arguments_object: false, requires_function_scope: false }, span: Span((76, 24), (76, 37)), linear_span: Some(LinearSpan { start: LinearPosition { pos: 2360 }, end: LinearPosition { pos: 2373 } }) }), span: Span((76, 18), (76, 37)) })
```

## Closure

Keep the frozen failure inventory and task ID. Record the fixing revision, fresh replay evidence for every listed execution, and adjacent-family results before marking fixed. A suspected cause needs confirmation from a reduced reproducer or an observed compiler path; a matching error string alone is insufficient.
