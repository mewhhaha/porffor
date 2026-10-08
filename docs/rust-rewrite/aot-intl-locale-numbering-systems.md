# AOT Locale numbering-system information

This source draft adds `Intl.Locale.prototype.getNumberingSystems` through the
Rust IR, compiler, native Intl operation and Wasm result construction. Native
joining, eleven genuine identity/profile producers and their checks, 41 Python
admission controls, module boundaries and independent native source review pass.
Full compiler source review, compilation and runtime verification remain pending.
Hour cycles and numbering systems share this proposed ABI16 verification batch.
T23 stays open and published conformance counts do not change.

The method first checks the Locale receiver brand, then reads its immutable
canonical locale tag. Every present `nu` value becomes a fresh singleton array,
including unknown values, compound values and the empty value from a bare `nu`
keyword. Explicit values do not go through a known-name filter or native default
query. Constructor options continue to validate Unicode type syntax; an empty
`numberingSystem` option throws, while valid unknown types remain open.

When the slot is absent, a private checked native request queries the existing
NumberFormat available-locale prefix inventory and returns only the matched
profile's default numbering system. An unmatched locale returns `latn`.
Likely-subtag maximization, regional `rg` or `sd` preferences, the global
supported-system inventory and the NumberFormat resolver's `en-US` fallback do
not select this result. Existing profile data therefore gives `ar` and
`ar-u-rg-egzzzz` the default `latn`, while `ar-EG` and
`ar-EG-u-rg-uszzzz` retain `arab`.

Proposed ABI16 operation42 writes a checked canonical default system name into
an eight-byte output span. Wasm requires a returned length of three through
eight and lowercase ASCII alphanumeric bytes before packing the string and
allocating its result. A private consuming checked response owns the temporary
until materialization. Native code returns data; the user program constructs
the array in Wasm.

Both explicit and default branches use the same small singleton array helper
as the hour-cycle explicit branch. Results use the method's defining Realm
Array prototype, expose a normal own index zero and writable array length, and
have no mutable alias to another result or the receiver. The method ignores
extra arguments and does not inspect receiver properties or public Array and
Locale constructors.

Five Rust Engine controls cover profile defaults, open slots and option
precedence, metadata and freshness, receiver branding, and defining-Realm
results and errors. Each uses the product Wasm-AOT backend in both Script modes.
The four primary pinned Test262 files define eight selected execution modes.
These are written controls and a selected scope; they have not executed.

The backend assumes the experimental Wasmtime feature lower bound documented
by this project. This draft adds no interpreter or dynamic source parser to the
emitted user program.
