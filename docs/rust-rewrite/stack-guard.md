# Catchable Wasm-AOT stack exhaustion

Status: focused integration verified, 2026-09-27. The final checkpoint
`target/watched/abi-final-v21.log` passes all 460 backend unit tests, 54 selected
engine integration tests, and ten engine stack library tests. The stack checks
include a forced Proxy-call guard refusal inside a foreign-Realm Promise job.
The subsequent complete engine library run passes 785/785
(`target/watched/engine-library-v25.log`), including the private module lifecycle
fixtures repaired to retain guarded entry points.
Earlier v13/v14 integration runs also cover all five recursion and all seven
prepared-script host tests. Guard wrappers use the prepared script's current
Realm instead of reading its lexical environment as a function object, and
ordinary calls restore the Realm active immediately before the call.
This work does not complete the original recursion Test262 case, which also calls
the unfinished semantic GC hook.

A native Wasmtime stack-overflow trap bypasses JavaScript completion handling.
The compiler therefore keeps each callable function's original index as a small
guard wrapper and appends its body after the original function section. Existing
direct calls and function-table entries continue to enter the wrapper. Source
functions, builtin functions, prepared scripts and runtime helpers that return
JavaScript throw completions need guards. The shared seven-argument/four-result
Wasm signature alone is insufficient: numeric conversion and RegExp helpers use
those result slots for other contracts. Pure conversion, RegExp, allocation and
mutation helpers remain in the verified bounded graph, together with the
error-object constructor. A refusal can therefore construct a fresh RangeError
without entering another guard or returning an error object as numeric data.

The pinned Wasmtime/Cranelift patches record each compiled function's native
frame bound and its active stack-pointer-to-frame-pointer distance after
register allocation. A leaf host callback recovers the calling wrapper's active
Wasm stack pointer from Wasmtime's saved exit state and compares it with the
runtime's soft Wasm stack limit. It rejects a mismatched active function or
module before dereferencing the saved frame. This is a runtime-specific
capability, not a portable Wasm API or a guessed recursion-depth limit.

The versioned `lila.stack_guard` section records the startup body, ordinary
wrapper/body pairs and all unguarded helper indices. Its original-function
space must partition exactly into startup, wrappers and helpers. Before instantiation, the engine checks that metadata
against the compiled module. It reads the actual helper bodies, rejects indirect
calls or edges outside the helper graph, rejects cycles, and computes the
longest simultaneously active helper chain from native frame bounds. Each
accepted entry reserves space for its body, the largest possible next wrapper,
and that helper chain, which includes RangeError construction. All size sums
are checked. Sequential helper branches are not counted as simultaneous frames.
A tiny main wrapper checks the relocated main body before its prologue or
bootstrap runs. Insufficient initial budget reports a host resource error: no
JavaScript handler or error intrinsic exists yet. The wrapper itself is the
unavoidable minimum entry frame; Wasmtime can reject entry before its callback
if even that frame cannot fit. No guessed host/trampoline byte allowance is used.

The callback is bound to its compiled Module and validates the wrapper/body
pair on every call; it does not execute JavaScript.

Wrapper emission distinguishes function contexts from runtime-helper arguments:
sharing a seven-integer signature does not make a helper's first argument a
function record. Function and prepared-script wrappers select their execution
Realm; runtime helpers inherit the active guarded Realm. Accepted wrappers tail-call their bodies so proper tail calls retain no guard
frame. Each relocated body captures the active Realm immediately before each
ordinary call and restores it afterwards. Capturing once on function entry is
insufficient: the Promise job scheduler temporarily enters each job's Realm,
and nested calls must restore that Realm rather than the scheduler's entry
Realm. The scheduler saves and restores both its current-Realm and guard-Realm
globals independently. Tail calls discard the caller frame and do not restore
it; bodies without ordinary calls need no capture local. The completion results
stay on the Wasm operand stack while restoration runs.
A refused call constructs a branded RangeError from that Realm's intrinsic
prototype. Native host callbacks use the existing execution-worker headroom
outside the configured 32 MiB Wasm stack allowance.

The coordinated patch spans `vendor/cranelift-codegen-0.136.1`,
`vendor/wasmtime-internal-cranelift-49.0.1`, `vendor/wasmtime-environ-49.0.1`, and
`vendor/wasmtime-49.0.1`. Each directory retains upstream provenance and a
`LILA-PATCH.md`. Native function/module cache formats are versioned for the new
compiled metadata. The program cache already incorporates compiler-artifact
identity.

Focused verification targets are `stack_budget_tests`, `stack_guard_host::tests`,
and the `aot_recursion_guard` integration target in `lila-engine`. They cover
native spills and shrinking stack budgets, malformed metadata and cyclic helper
graphs, JavaScript catch/finally behavior, repeated calls after unwinding,
generators, and cross-Realm error prototypes. Exact results belong in
[the failure backlog](../../tasks/README.md).
