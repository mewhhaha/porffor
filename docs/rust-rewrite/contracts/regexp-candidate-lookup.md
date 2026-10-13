# RegExp candidate lookup

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`, RegExpInitialize,
Pattern early errors, Unicode strings and RegExp matching. The backend requires
the existing Wasm GC and typed reference capability set.

The actual `aee6bc109` compiler fails the historical
`sloppy-script:intl402/default-locale-is-canonicalized.js` execution at native
compilation. An isolated diagnostic using the existing emitted-body budget
measures `helper::regexp_program_candidate` at 27,330,651 bytes against a
7,654,321-byte budget. That diagnostic is not a replacement conformance verdict;
the unmodified historical failure remains in its replay cohort.

The prepared lookup emits one loop over the complete compiler-owned candidate
table. Candidate source and flags are pooled GC-string ordinals, compared by
their UTF-16 values. The writer checks address, count and extent and shares all
four word indices and the derived row stride with the reader. Every candidate
and outcome remains present; candidate selection and flag expansion are unchanged.

A Program hit copies the validated descriptor's active static-memory span into
one unpublished GC construction and publishes its typed RegExp program once.
This retains descriptors deduplicated into either the runtime or program data
prefix. Rejected means SyntaxError; Unsupported and a miss enter the existing
runtime compiler. An impossible entry discriminant reports CorruptProgram.
The string table and constructed bytes remain rooted throughout lookup/copy.

Controls decode the actual wire rows for all three outcomes, validate the actual
emitted module and compare encoded helper sizes for 16 and 512 source strings.
Native controls exercise many candidate sources/flags, NUL, non-BMP and lone
surrogate strings, syntax rejection, empty patterns and a computed cache miss.
The complete existing RegExp inventories and the unchanged failing historical
IDs must pass on the actual new compiler before these repairs are accepted.

The row scan remains linear at execution time. Large array literals and pooled
string-table initialization are separate emitted-body limits; this change does
not establish that those cases pass or that the full pinned suite is green.
