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
constructor getter. Actual `9ddc57a2c` passes all 4,489 affected native IDs,
retaining all 4,485 previous passes, and the complete original Function893 family
has 879 Success/14 NotImplemented: ten limitations removed, all 869 prior passes
retained. Its Iterator1308 replay has 1,308 Success, eliminating all 112 Bugs
and retaining all 1,196 prior passes. These complete family comparisons and
native scopes do not establish full pinned acceptance.

The fourteen stateful object-to-source constructor executions and four BMP eval
executions retain their explicit unsupported ownership in that checkpoint.

A separate actual-worker boundary probe catches a real new regression before
publication: `b486503d2` executes all 256 literal body candidates in an Array
callback, while `9ddc57a2c` loses them when a parameter has a `var` redeclaration.
The raw completed owner is
`target/source-candidate-retention-regression-9ddc57a2c-20261013/receipt.json`.
Its first nested Cartesian probe is unsupported on both workers; the corrected
conditional 16-by-16 probe passes both. Neither is reported as a second measured
regression. The 256-body regression prevents accepting/pushing that source batch.

The verified `4a3968e5e` repair tags additive hints as `SupplementalText`. Primary
candidates retain their original all-or-none 256-value limit. Supplemental
values fill spare slots; concatenation preserves the tag, and an overflowing
expanded complete tuple product falls back to every original primary tuple.
The tag travels through the existing binding/parameter/child-lowerer maps.
It is excluded from eager module-request discovery and scalar source facts.

For unknown object-to-parameter conversions, matching-grammar parsing of finite
bodies can supply IdentifierReference strings as optional parameter hints.
The visitor includes expression and shorthand references, excluding binding
names, property names and labels. Existing arguments and body bytes remain
unchanged. AST `+` composition carries supplemental status. Discovery bounds
total body bytes and new composed hints to 64 KiB, names/complete tuples to 256;
exceeding a hint budget preserves every original complete tuple. Speculative
parsing uses Front directly without recording host requests. Added hints require
call-free bodies: Call/New/ImportCall/tagged templates and optional calls are
declined so preparing a guessed tuple cannot discover nested source or host
requests. Declined bodies are excluded from added tuples, including mixed bodies;
all existing tuples still retain their ordinary admission.
Actual ToString order, exceptions, call/apply forwarding and exact converted
tuple matching remain the only execution authority.

Actual replay removes twelve of the fourteen remaining executions: A2_T4/T5/T6,
call T3/T4 and apply T3, each in both modes. The initial prediction mistakenly
grouped both T4 bodies as last-formal-only. Call T4 is
`this.shifted=a3+a2+a1.length;`: all three IdentifierReferences supply the exact
live parameter tuple, and its assertion checks the executed result `"24"`.
Apply T4 is `this.shifted=a3;`, so its two unused parameter names are not inferred
from spelling or test shape. Those two executions retain unchanged unsupported
details. The original prediction/comparison owner stays red and unchanged; a
separate post-run comparison verifies the complete original inventory, every
earlier pass, all twelve eliminated IDs and the two remaining failure records.
Four IR and four native controls cover all source grammars, both candidate
boundaries, eager request exclusion, live conversions, abrupt order, forwarding
and unmatched-source refusal. All eight focused controls and 4,497 complete
affected native IDs pass, retaining every 4,489 earlier pass. The exact saved
256-body source passes on the immutable successor worker. Complete original
families are Function891/893 with two explicit NotImplemented and Iterator1308/1308
Success; all four previous RegExp/ArrayBuffer Bug IDs remain green. See the
[complete checkpoint](constructor-iterator-checkpoint-20261013.json).

The first linked-native focused run passed forward-var and both Iterator
controls, but the constructor mutation control had an invalid expectation.
`AsyncFunction.prototype.constructor` is non-writable and configurable under
`sec-async-function-prototype-properties-constructor` in the pinned standard.
Sloppy assignment correctly left it unchanged. The corrected control checks
that descriptor and uses DefineProperty for replacement/restoration; the old
failed native owner and trace remain at
`target/constructor-iterator-focus-linked-36d111e86-20261013/receipt.json`.
This failed test establishes no product regression.
