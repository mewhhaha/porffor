# Differential output comparison policy

## Current worker failure priority — 2026-10-04 dry source

The supervisor now publishes completed observations only after worker retirement.
A WorkerFailure from either backend returns the distinct red worker verdict
before output-policy or semantic comparison. Its transcript is `Incomplete`,
with only validated committed frames, and it receives no mismatch signature.
Missing capture cannot acquire `Captured` authority. Completed workers retain
the exact two-row `OutputComparisonPolicy` and protocol comparison order below.

The existing owner controls and updated source guards remain authored and
unexecuted. The historical pass counts below do not accept this source. See the
[worker lifecycle](differential-worker-lifecycle.md).

Differential replay decides whether output is admissible before protocol-specific
observation comparison. Schemas v1 and v2 require both captured transcripts
to be empty; schemas v3–v7 require both transcripts to be captured so their later
comparison can compare their exact ordered events.

## Closed policy

`OutputComparisonPolicy` is a private, non-derived two-row domain. The
exhaustive protocol projection selects `RequireCapturedEmpty` for v1 and v2 and
`CompareCapturedPrintTranscript` for v3–v7. V7 adds the native rooted completion
graph contract; its capture/budget refusals remain red even when they match.
The exhaustive consumer evaluates the
Wasm observation before the spec-exec observation and runs before the protocol's
completed-observation comparison, after worker-failure classification.

The policy has no debug, clone, copy, equality or default capability. A new
protocol row must select a policy in the exhaustive projection; a new policy
row must define its behavior in the exhaustive consumer. There is no caller
Boolean or fallback that can silently weaken output comparison.

## Durable regressions

The recursive structure guard fixes the private declaration and seven-mention
census, both protocol projections, both exact comparison rows, and the policy
check's placement before backend projection. Existing owner witnesses prove
that output from either backend makes a v1/v2 report red and that equal ordered
v3 transcripts admit the distinct green verdict. Run:

```sh
cargo test -p lila-test262 --test output_comparison_policy_structure -- --test-threads=1
cargo test -p lila-test262 differential::tests::either_backend_output_makes_a_no_output_case_red -- --exact
cargo test -p lila-test262 differential::tests::v3_matches_primitive_completion_and_exact_ordered_print_transcript -- --exact
```

At the earlier checkpoint, the structure target passed `4/4`, both exact owner
witnesses passed `1/1`, and the package formatting check was green. Independent review found and corrected
an exhaustiveness wording overclaim; the executable invariant was clean. The
shared checkpoint passed `cargo fmt --all -- --check`, `cargo xc`,
`git diff --check`, the module-boundary check and the task-plan check.

The current projection and its existing structure guard include v5 selected
object probes and v6 explicit Test262 host authority, with the same two policy
rows and ordering. V6 adds no output dimension: the new Realm grammar prints
its selected primitive identity comparisons through the same observed contract.
Their current executable checks remain pending; earlier passes are not evidence
for the new source.

## Nonclaims

The earlier derive-only production change altered no corpus or report wire bytes,
fingerprint input, mismatch signature, verdict, backend execution or comparison
order. It adds no observation dimension, module replay, oracle or semantic
equivalence claim. The additive v4 graph path reuses that comparison policy;
its separate source-authority contract defines the new protocol. V6's explicit
host authority is defined by the [generated scenario contract](differential-generated-scenarios.md).
