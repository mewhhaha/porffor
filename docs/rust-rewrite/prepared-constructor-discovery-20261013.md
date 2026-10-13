# Prepared constructor discovery — 2026-10-13

The unfiltered frozen baseline exposes three ordinary Function sources whose
hoisted `var` arguments become `"undefined"`, and two literal source tuples
reached through declared async/async-generator functions' `constructor`
properties. Each source runs in sloppy and strict mode, for ten executions.

Candidate discovery retains `"undefined"` when the declaration prepass visits a
`var` without an initializer. These optional candidates survive invalidation of
flow facts; repeated declarations still perform no write, and previously found
source candidates remain. A known function target's source execution protocol
also supplies a possible Function-constructor kind for property projection.

Both changes feed the existing bounded prepared-source registry and ordinary
parser, early errors, IR and Wasm emitter. They establish no runtime value or
callable fact. Actual property resolution, replacement constructors, argument
evaluation and ToString still run. The intrinsic checks the converted complete
source tuple before selecting a prepared unit. No runtime parser, new source
grammar or path-based admission is introduced.

Controls cover all four declared function kinds, forward declarations across
calls/try statements, source-comment preservation, existing-value/parameter
redeclarations, effectful source conversion, prototype replacement and an own
constructor getter. Focused and broad verification are pending; neither this
source patch nor the earlier family checkpoints close full pinned acceptance.

The fourteen stateful object-to-source constructor executions and four BMP eval
executions retain their explicit unsupported ownership under T13. Their source
values require further analysis; this patch supplies no verdict for them.

The first linked-native focused run passed forward-var and both Iterator
controls, but the constructor mutation control had an invalid expectation.
`AsyncFunction.prototype.constructor` is non-writable and configurable under
`sec-async-function-prototype-properties-constructor` in the pinned standard.
Sloppy assignment correctly left it unchanged. The corrected control checks
that descriptor and uses DefineProperty for replacement/restoration; the old
failed native owner and trace remain at
`target/constructor-iterator-focus-linked-36d111e86-20261013/receipt.json`.
This failed test establishes no product regression.
