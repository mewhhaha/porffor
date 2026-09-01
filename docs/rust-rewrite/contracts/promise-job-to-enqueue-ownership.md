# Promise job enqueue ownership

Status: source-equivalent Wasm-AOT ownership invariant, focused-verified on
2026-08-28.

## Single enqueue token

`builtins/promise/promise_job_to_enqueue.rs` privately owns
`PromiseJobToEnqueue`, the two complete producers and the sole Promise-job FIFO
append. Its two shapes carry either the reaction record plus argument
payload/tag or the thenable-job record plus callback payload/tag. The parent
cannot name, import, re-export, construct or project the authority; its retained
callers can only invoke the two `pub(super)` producers. The type derives no
cloning or copying capability. Once a selected token is passed to the private
`emit_enqueue_promise_job`, Rust ownership prevents that same selection from
being enqueued or otherwise used again.

The two named producers construct their token only after their inputs are
ready. In particular, the thenable producer writes all six fields of its heap
record before selecting `ResolveThenable`; it releases the temporary record
local only after the owned enqueue call returns.

The consumer matches the owned token once and exhaustively. Each arm writes the
callback and argument fields, derives the job Realm and selects the matching
`PromiseJobKind`. The common tail then stores Realm, null next link and kind
before performing the one FIFO append: initialize the head only for an empty
queue, otherwise link the old tail, then publish the new tail. Its three locals
are released in reverse order.

This is a Rust ownership boundary, not a unique-local-index type. The underlying
Wasm local numbers remain copyable, and two separately constructed tokens may
intentionally name the same inputs when the ECMAScript algorithm schedules two
jobs. The invariant prevents implicit reuse of one already-selected token; it
does not infer whether two explicit enqueue operations are semantically
duplicates.

## Durable evidence

`crates/lila-aot-wasm/tests/promise_job_to_enqueue_structure.rs` uses a
Rust-lexical recursive census that excludes comments and every Rust
string/character literal form. It pins the one private child module, zero
imports/re-exports, exactly six child-only authority mentions, two producers,
one owned parameter and two exhaustive arms; bans derived/manual capabilities,
casts, catchalls and alternate raw owners; and exact-normalizes the complete
consumer through payload selection, FIFO append and reverse local release.

The pre-extraction 15-line domain, 17-line reaction producer, 52-line thenable
producer and 122-line consumer retain SHA-256
`372060fd20211269848fa8ff18be925ee15bcc771ec2d2be4f2460b1d138b58d`,
`4086b791c4da713c743617447fd8e75448fe02eb76091e0bc61b3a8caf3081e7`,
`6d814a02b3ac71f0dcee2a8ca94574d2e91246d2dec826478b76c6775e973c6d`
and
`351e4c06208555795e68495874a346fb4aca978690c237a06e0e89b6757e6f4b`.
Their combined 206 selected lines retain SHA-256
`1d8d89b234378b015344445320aabbd5787ff0494aa7da4194457f46fd5850e9`.
The formatted 210-line child has SHA-256
`c4587437a74e5788055bb21104c1f5096518a7933feba487be7619287c3416c6`
and reduces the concurrent `promise.rs` snapshot from 8,923 to 8,717 lines.

Focused verification:

```sh
cargo test -p lila-aot-wasm --test promise_job_to_enqueue_structure --quiet
cargo test -p lila-engine tests::wasm_backend_promise_reactions_run_after_synchronous_code_in_registration_order -- --exact --test-threads=1
cargo test -p lila-engine tests::wasm_backend_promise_thenable_jobs_are_asynchronous_and_settle_once -- --exact --test-threads=1
cargo test -p lila-cli --test cli functions::run_wasm_backend_preserves_created_realm_promise_internal_callbacks -- --exact --test-threads=1
./target/debug/lila --jobs 1 test262 run built-ins/Promise/resolve-thenable-immed.js --suite-root test262/vendor/test262 --snapshot-dir /tmp/lila-promise-job-resolve-thenable --snapshot-name promise-job-resolve-thenable --execution-backend wasm-aot --threads 1 --timeout-ms 180000
./target/debug/lila --jobs 1 test262 run built-ins/Promise/prototype/then/resolve-settled-fulfilled-non-thenable.js --suite-root test262/vendor/test262 --snapshot-dir /tmp/lila-promise-job-fulfilled-non-thenable --snapshot-name promise-job-fulfilled-non-thenable --execution-backend wasm-aot --threads 1 --timeout-ms 180000
```

The post-extraction structure target passes `3/3`. After the extraction, the
reaction-FIFO and thenable-settle-once engine witnesses pass `1/1` each, the
created-Realm internal-callback CLI witness passes `1/1`, and both exact
Test262 leaves pass sloppy and strict execution for an aggregate `4/4`, with
every failure bucket at zero. The shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. No semantic golden or broad
suite was run for this source-equivalent ownership move.

Independent review is clean after the guard was strengthened for the private
child, exact producer bodies and alternate method/UFCS route closure.

## Nonclaims

This source-equivalent lane changes no job record word, callback Realm,
argument, enqueue/drain order, local lifetime or emitted instruction. It does
not change the main-job checkpoint or unhandled-rejection reporting contracts,
add a job kind, prove all Promise scheduling, establish full Promise/Test262
conformance, or refresh a Wasm semantic golden or README status.
