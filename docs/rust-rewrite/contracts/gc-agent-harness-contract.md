# Agent harness protocol and host effects

The Wasm-AOT Test262 broadcast wrapper forwards both the SharedArrayBuffer and
message id. The native body validates the shared buffer before applying
`ToInt32` to the id once. The host message carries the native resource and the
signed id together; it does not use a global last-message slot.

The native receive body returns a fresh GC Array containing a fresh
SharedArrayBuffer wrapper retaining that resource and the signed id as a
Number. Lowering describes this result as an Array. The ordinary JavaScript
harness unpacks both entries and invokes its supplied callback with two
arguments. That callback is a source-function effect; the native receive body
itself does not synchronously invoke user code.

The closed host effect match classifies Print, realm Script evaluation, agent
start/report/sleep and broadcast as synchronous user-code effects because their
actual bodies perform observable coercion. This keeps caller knowledge from
surviving coercion hooks that can mutate its dependencies. Existing distinct
caller-flow invalidation remains unchanged.

Two authored Engine controls include the actual harness asset and replace its
native dependencies with ordinary functions. They check wrapper argument
forwarding, absence of extra coercion, callback receiver, results and exact
Throw identity in both Script modes. These are component controls; native
resource delivery and coercion controls belong to the complete host batch.

This is source-only progress. Compilation and runtime verification are pending
until the whole task source batch is complete. Later execution must use the
confirmed 4 GiB aggregate cgroup cap and serial verification launcher.

The native Engine regression prelude consumes the receiver's exact two-member
`[SharedArrayBuffer, id]` result and calls the Test262 callback with those two
members. Passing the pair itself as a typed-array buffer loses the shared
resource and cannot exercise worker notification. Product execution and raw GC
ABI observations now link the same private `wasm_agent_host::agent_call`
dispatcher. Its operations, resource ownership, and failure paths are unchanged
by that extraction. Verification of this repair remains pending until the
frozen workspace sweep finishes and the complete batch is applied.
