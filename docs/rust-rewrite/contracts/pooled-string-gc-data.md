# Pooled string initialization

The compiler interns each literal's exact UTF-16 units into one passive Wasm
data image. Private slice bounds come from that same producer; the GC string
constructor accepts those slices rather than caller-provided byte offsets.
Its `array.new_data` initializes the original `CodeUnitArray`, and consuming
`StringConstruction::publish` creates the original immutable `StringValue`.

The image is the first data segment and the module publishes its data count
before code. Existing active static data keeps its original linear-memory
addresses. The passive image is initialization data, not another JavaScript
string representation. NUL, paired surrogates and isolated surrogates retain
the existing decoder's exact units.

Literal length grows artifact data without expanding the main function into
one instruction sequence per code unit. The emitted-artifact control checks
that property, validates the Wasm and verifies both the passive payload and its
actual GC-array consumers. Native string and lifecycle regressions remain
required; source review does not establish runtime acceptance.
