# Annex B unescape publishes exact GC UTF-16

[Annex B unescape](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-unescape-string)
recognizes `%XX` and lowercase `%uXXXX`, yielding one UTF-16 code unit per valid
token. Unrecognized or incomplete spellings copy the next original unit and
resume scanning. It performs ToString once before decoding and never throws
merely because percent syntax is malformed.

The native owner is the complete six-function family in `builtins/uri.rs`.
`unescape` now counts and writes units into an exact GC `CodeUnitArray`, then
consumes `StringConstruction` to publish one immutable `StringValue`. The
former pending-lead/WTF-8 output coordinator and packed byte payload are gone.
Pairing is already inherent in the ECMAScript unit sequence: decoded/decoded,
decoded/raw and raw/decoded lead/trail boundaries all produce the same UTF-16
String. Lone surrogates and U+0000 remain exact units.

The parser cannot publish an intermediate String, and the construction's
mutable code-unit storage has no write capability after publication. The shared
pure traversal counts first and fills second without any extra coercion hooks.
The signed I32 output extent is checked before narrowing; an unsupported resource
extent traps rather than silently shortening the String.

The finite GC Engine cohort retains exact pair/lone/raw-token cases, malformed
suffix rescanning, byte escapes, astral boundaries, original ToString abrupt
identity and called-Realm Symbol conversion errors. The existing CLI Annex B core
fixture remains. The raw `annexb_unescape_output_structure` mirror retires; its
historical payload lifecycle assertions are not evidence for the GC model.

All new controls remain unrun during the all-task source phase. Type checks,
Wasm execution, pinned unescape leaves and the broad verification ladder are
pending. This change does not claim complete Annex B/T24 or alter published counts.
