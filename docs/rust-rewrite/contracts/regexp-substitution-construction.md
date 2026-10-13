# RegExp substitution construction

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`,
GetSubstitution and RegExp.prototype [ @@replace ]. Completed UTF-16 Strings
and undefined capture values reach GetSubstitution only after their prescribed
conversions. Named capture Gets and conversions remain observable per occurrence.

The actual historical `aee6bc109` compiler records Crashes in both modes of
`staging/sm/String/replace-math.js`: a 532,676,648-byte GC allocation fails
outside the JavaScript try/catch. The fixture requests a replacement far beyond
the engine's existing u32 string extent. Growing substitution concatenates each
piece with the entire previous prefix, so it allocates enormous intermediate
strings before its old wrap check can detect the eventual oversized result.

The repair keeps completed pieces in a private list, accumulates their lengths
in i64 arithmetic, and returns a catchable RangeError when the existing extent
is exceeded. For an admitted result, it allocates one unpublished backing,
copies exact code units once and publishes one StringValue. User operations
remain in their original order and happen once; final copying runs no capture
Gets or conversions. Retained pieces and their code-unit arrays stay GC roots
across reentry. This removes repeated copying of the growing prefix.

Native controls cover the oversized expansion followed by a normal replacement,
repeated named Gets/conversions, reentrant replacement, surrogate units and an
abrupt later getter. Existing literal/function replacement controls and complete
String/RegExp inventories remain required. The original two Test262 executions
are unchanged. Prepared source does not establish a pass or measured speedup.
The length check does not guarantee that every representable allocation fits
the current Store's remaining memory; other resource failures remain explicit.
