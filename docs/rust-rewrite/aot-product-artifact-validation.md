# Product Wasm artifact validation

The product_artifact gate now validates emitted Wasm types and function bodies
with wasmparser::Validator before inspecting imports and instructions. A binary
can decode successfully while containing an invalid operand stack or function
result type; Parser alone does not establish a valid executable artifact.

The numeric source-marker fixture remains. Five additional source families
exercise loop branches, closure capture, throw/catch/finally completion, heap
aggregates and BigInt/string values. Every emitted artifact runs through the
same type-validation, compiled-code and no-evaluator-import assertions.

A positive minimal Wasm control validates. Its negative counterpart changes an
i32.const into i64.const without changing the function's i32 result signature;
all instructions still decode, but validation must reject it. This proves the
new gate distinguishes parsing from type correctness.

```sh
cargo test --locked -p lila-aot-wasm --test product_artifact
```

The validator enables the declared GC/reference/exception/thread/tail-call
capabilities explicitly rather than allowing every experimental proposal. This
is not a replacement for running the artifact in the configured Wasmtime engine,
checking JavaScript output semantics, or publishing a full raw-source Test262
run. No Test262 status or denominator is changed.
