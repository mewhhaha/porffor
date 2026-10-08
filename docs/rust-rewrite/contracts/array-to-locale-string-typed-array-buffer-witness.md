# Generic Array toLocaleString observable length

Status: current Wasm-GC source authored on 2026-10-05. Compilation, runtime
regressions and conformance verification have not run.

TypedArray locale formatting retains initial brand/read-view admission and live element operations. No private address or scalar Realm fallback crosses the common invocation owner.

Values and completions retain typed GC references. Arrays use the single sparse
indexed storage owner; observable length never sizes a hole allocation. Old raw
payload/local-offset and direct-dispatch source-string mirrors are retired.
Actual Engine and CLI behavior controls remain the verification evidence; their
authorship does not establish a passing result.

See [native Array GC methods](native-array-gc-methods.md) and
[Array indexed storage](gc-array-indexed-storage.md) for the complete current
owners, lifecycle rules and unrun finite controls. Historical beforeimages and
source-only receipts remain in the atomic migration packet.
