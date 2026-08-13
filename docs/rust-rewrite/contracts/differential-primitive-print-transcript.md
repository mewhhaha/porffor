# Differential primitive-completion and print-transcript protocol

Status: normative for differential corpus and report schema v3.

## Bounded claim

Schema v3 compares three report dimensions that the engine already observes
without additional execution or coercion:

1. the top-level normal-or-throw completion kind;
2. its owned primitive value; and
3. the ordered `PrintLine` events captured by that execution's root output
   session.

A green result establishes equality only for those dimensions. It retains the
report's `semantic_equivalence: not_established` value and makes no claim about
object or Symbol identity, error realms, descriptors, prototypes, arbitrary
side effects, panics, host crashes or spec-exec timeout enforcement.

## Closed protocol domain

`DifferentialProtocol` is the only in-memory protocol authority. Its exhaustive
projection fixes the admitted wire pair and output policy:

| Corpus schema | Observation contract | Output policy |
| --- | --- | --- |
| v1 | `self_checking_no_output` | both captured transcripts must be empty |
| v2 | `primitive_completion_no_output` | both captured transcripts must be empty |
| v3 | `primitive_completion_print_transcript` | both transcripts must be captured and compared exactly |

Wire decoding rejects every version/contract cross-pair. The output policy is
a closed Rust enum selected by the protocol; replay has no boolean or caller
default that can silently turn transcript comparison off. Adding a protocol or
policy therefore requires updating exhaustive matches in decoding, projection,
comparison, fingerprinting and the schema-v1 arithmetic campaign.

Schemas v1 and v2 keep their existing JSON field order, bytes, fingerprints,
mismatch signatures, verdicts and fixtures. Schema v3 is strictly additive.

All three schemas currently admit only Scripts with an outer source closed over
module requests. Module goals and actual or conservatively possible outer
Script dynamic imports are rejected because the wire carries no dependency
graph. Imports synthesized by eval or Function construction in spec-exec meet
the mandatory reject-all loader rather than ambient files; Wasm-AOT retains its
dynamic-source diagnostic. The independent admission/runtime invariant and
future graph requirements are normative in
[`differential-source-closure.md`](differential-source-closure.md).

## Primitive completion observation

V3 reuses v2's primitive domain and canonicalization:

- `undefined` and `null` retain their distinct types;
- Boolean retains its value;
- Number retains canonical NaN bits and distinguishes signed zero;
- String retains its UTF-16 code units, including lone surrogates; and
- BigInt retains its signed decimal spelling.

Normal and throw are distinct completion kinds. Symbol and Object remain
type-only engine observations and are outside the protocol. Either backend
producing one makes the report `observation_contract_violated`, even if both
backends produce the same unsupported type. No coercion, description lookup,
property read, debug rendering or backend identity guess is permitted.

## Ordered print transcript

`Engine::observe_script` and `Engine::observe_module` already return owned
`HostOutputEvent::PrintLine` events. Differential replay projects those events
once, in their original order, to `OutputEventsObservation::Captured`. It does
not call `print` again or coerce an argument a second time.

V3 requires `Captured` from both backends. `Unavailable` is a contract
violation, not an empty transcript. Captured transcripts compare as exact
ordered sequences of Rust strings: event count, event boundaries, empty lines,
text and order all matter. Thus `[]`, `[""]`, `["ab", "c"]` and
`["a", "bc"]` are four different observations.

The transcript belongs to the root observed execution, including the root job
checkpoint or module evaluation performed by that execution. Agent-produced
output remains outside the v3 semantic claim: the current Wasm worker and
spec-exec capture boundaries do not provide a common agent ordering contract,
so corpus authors and report consumers must not treat agent lines as a
backend-comparable transcript.

## Verdict decision

After both backend observations have been projected:

- unavailable output or an unsupported completion is
  `observation_contract_violated`;
- two primitive completions with equal kind, value and ordered transcript are
  `primitive_completion_and_print_transcript_match`, the distinct v3 green
  verdict;
- two primitive completions that differ in any compared dimension are
  `mismatch`;
- one primitive completion and one engine failure are `mismatch`;
- two engine failures are `both_failed`; and
- a v1-only projected execution shape reaching v3 is
  `observation_contract_violated`.

Only the distinct v3 match verdict is green for schema v3. Shared failures,
contract violations and mismatches remain red.

## Stable mismatch signatures

V3 signatures never embed raw output text or backend diagnostics. Each backend
observation is reduced to an FNV-1a-64 digest under the
`lila-diff-v3-backend-observation` domain. Every variable-width component is
fed through the shared `fnv_field` operation, which prefixes the component with
its little-endian `u64` byte length. The typed execution marker, completion
kind, primitive type, canonical primitive signature, output-availability
marker, encoded event count and every UTF-8 event string are separate fields.
Engine failures include only their stable phase, never diagnostic text.

The final mismatch digest uses the
`lila-diff-v3-primitive-completion-print-transcript` domain and independently
length-prefixes the case ID, case fingerprint, parse goal and both labelled
backend digests. This makes event boundaries and all enclosing fields
unambiguous while keeping reports useful for triage through their full
structured observations.

## Durable witness and nonclaims

The committed v3 foundation case prints two distinct lines in order and
completes with the primitive Number `3`. Unit gates pin its wire round-trip and
fingerprint, the distinct green verdict, output ordering, boundary-sensitive
signatures, completion mismatches, unavailable output, unsupported values and
backend failures. A feature-gated end-to-end gate replays the fixture through
Wasm-AOT and spec-exec.

This slice does not change either engine, runtime output capture, the product
host surface, v1 generation/reduction, Test262, snapshots, fuzzing, object or
Symbol comparison, external-engine support, panic isolation, performance
budgets or CI scheduling. It does not provide module replay; current corpus
protocols reject outer module requests and deterministically reject requests
created at runtime until an embedded-graph protocol exists.
