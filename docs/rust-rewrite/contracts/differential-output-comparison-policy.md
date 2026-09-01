# Differential output comparison policy

Differential replay decides whether output is admissible before either backend
observation is projected. Schemas v1 and v2 require both captured transcripts
to be empty; schema v3 requires both transcripts to be captured so its later
comparison can compare their exact ordered events.

## Closed policy

`OutputComparisonPolicy` is a private, non-derived two-row domain. The
exhaustive protocol projection selects `RequireCapturedEmpty` for v1 and v2 and
`CompareCapturedPrintTranscript` for v3. The exhaustive consumer evaluates the
Wasm observation before the spec-exec observation and runs before dispositions
or projected backend observations are constructed.

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

The structure target passes `4/4`, both exact owner witnesses pass `1/1`, and
the package formatting check is green. Independent review found and corrected
an exhaustiveness wording overclaim; the executable invariant was clean. The
shared checkpoint passes `cargo fmt --all -- --check`, `cargo xc`,
`git diff --check`, the module-boundary check and the task-plan check.

## Nonclaims

This derive-only production change alters no corpus or report wire bytes,
fingerprint input, mismatch signature, verdict, backend execution or comparison
order. It adds no observation dimension, module replay, oracle or semantic
equivalence claim.
