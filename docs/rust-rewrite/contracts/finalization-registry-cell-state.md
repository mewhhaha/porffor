# FinalizationRegistry cell-state authority

`FinalizationRegistryCellState` is the private persisted lifecycle domain for
registry cells. Its declaration owns the complete stable wire mapping:
`Vacant = 0` and `Occupied = 1`. The domain derives no copying, comparison,
defaulting or formatting capabilities.

The typed serializer is the sole authority allowed to write the cell-state
word. Registration publishes `Occupied`, unregistration publishes `Vacant`,
and cell-array growth first admits the old word through the complete state
domain before serializing the selected state into the new record. The remaining
target, holdings and unregister-token words are copied only after that exact
state admission.

Unregistration likewise admits only the two declared words before routing
through an exhaustive Rust match. A `Vacant` cell skips token access. An
`Occupied` cell may compare and clear its token. Any invalid persisted word
reaches a Wasm `unreachable` invariant failure; it cannot silently acquire the
old nonzero-means-occupied behavior.

This contract hardens the current linear record representation. It does not
implement weak reachability, discover unreachable targets, or schedule
FinalizationRegistry cleanup jobs. Those remain blocked on the collector and
job integration described by T21.
