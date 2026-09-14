# Pinned assertion and property harness

The embedded Wasm-AOT named harness carries verbatim copies of the pinned
Test262 `assert.js`, `sta.js`, and `propertyHelper.js` sources, including their
copyright and license notices. Its `sta-preamble.js` section is the same pinned
`sta.js` source. Byte-identity tests compare every section with the vendored
file plus the materializer's terminating newline. The local `isConstructor.js`
bridge remains separate and uses the existing typed host intrinsic.

This replaces local adapters that accepted truthy assertion arguments, threw
primitive strings, omitted error messages and methods, and skipped property
checks or destructive probes. The confirmed main witness was
`harness/assert-samevalue-objects.js`: its assertion failure must be an object
whose constructor is `Test262Error`.

The original upstream source now defines strict-true assertions, SameValue
and array comparison, error constructor checks, diagnostic formatting,
descriptor validation, captured primordial methods, writability and deletion
probes, and the `restore` option. Assertions in regression tests verify these
observable results without relying on the harness assertion under test.

Ordinary materialization always retains the complete named assertion prelude.
Because that prelude uses `Test262Error`, cases without a full host include
the canonical `sta-preamble.js`; host-owning cases include `sta.js`. Included
property helpers are copied in full. The obsolete assertion omission and
compact property verifier are removed, along with their fingerprint gates
and plan fields. New canonical fingerprints describe the actual source;
they do not authorize replacing it with the old reduced implementations.

The canonical shortcut scanner observes two removals: the source-text
predicate in `typed_array_literal_helper_plan`, and the compact property
prelude publication in `materialize_test`. The latter shifts subsequent
materializer ordinals; unconditional assertion publication also changes its
reviewed selector fingerprints. Accounting changes from 112 to 110 entries:
30 legitimate harness adaptations, 39 diagnostic observations, and 41
remaining semantic shortcuts. This is selector accounting, not a Test262
pass count or a claim that all remaining runner shortcuts are removed.

The host transport and other existing materialization transformations retain
their separate owners. Harness changes can expose compiler failures that a
weaker adapter concealed; those must be fixed or reported using the actual
failure, rather than changing the pinned harness to recover a green result.
