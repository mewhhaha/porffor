# RegExp finite string catalog

The existing immutable Unicode property catalog now owns complete sequence
payloads for all seven exact provider string-property names. Its one `OnceLock`
constructor consumes the closed `UnicodeStringProperty::ALL` row authority and
`unicode_string_property_sequences` from the committed regress tables. It checks
nonempty sequences and code points through U+10FFFF, deduplicates exact keys and
retains their code-point lexicographic order. It keeps singleton sequences in the
same borrowed payload. No source candidate selects a subset, and no provider
version, source data or provenance is changed or generated.

The private catalog entry fields remain the constructor boundary. The borrowed
`Strings` value now carries the validated sequence slice. Native static parsing
and StringPool serialization consume that same actual catalog payload. Static
`FiniteClassSet` still folds each operand before algebra, moves singleton keys
into ranges and retains multi-code-point and empty keys in its existing set;
static lowering retains its existing longest-string, singleton, empty priority.
The seven provider properties themselves have no empty key. Source `\q` can
separately supply one through its complete parser.

Every StringPool serializes the complete catalog unconditionally. The property
row remains 40 bytes: name pointer, name length, payload pointer, payload count
and closed kind, at offsets 0, 8, 16, 24 and 32. CodePoints retains the existing
8-byte u32 range pairs and exact alias/range semantics. Strings selects a table
of 16-byte descriptors: code-point pointer at 0 and code-point count at 8. Each
backing key is immutable u32 little-endian code points, not UTF-16 halves. The
serializer shares identical point keys across the seven properties, aligns key
and descriptor storage, checks addresses/counts within the existing 32-bit image
space, and exposes the image owner only after checking its complete end.

The emitted parser's real kind dispatch consumes these same payload words.
Operand-local simple folding, deduplication and bitmap/finite-key algebra belong
to its validated character mode and workspace. The completed finite atom and
lowerer own matcher ordering and checked instruction publication. Image rows
never become source-sized AST alternatives or a new matcher opcode/descriptor.
The catalog's lexicographic key order is set identity; matcher priority orders
complete strings by descending code-point length before singleton and empty
alternatives. A shared prefix retains each distinct complete key.

Source metadata checks of the committed provider show 1400 Basic_Emoji keys,
12 keycaps, 259 flag sequences, 665 modifier sequences, 3 tag sequences, 1614 ZWJ
sequences and 3953 RGI_Emoji keys. RGI is the exact union of the other six sets;
all seven have nonempty, unique full-code-point keys. The existing IR catalog
control now checks real members, strict nonmember prefixes, full set cardinality
and the distinction between astral code points and split UTF-16 units. Existing
provider-domain assertions follow its actual closed row macro and shared catalog
consumer rather than the retired pre-macro spelling.

Normative set/folding authority is [ECMA-262 2026 CompileToCharSet](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-runtime-semantics-compiletocharset)
and [UnicodeMatchProperty](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-runtime-semantics-unicodematchproperty-p).

This is a source implementation within the complete computed finite-string
batch. Rustfmt, hashes, source metadata and ordinary patch roundtrips check source
transport. No compiler, Cargo, tests, JavaScript parser, runtime, generator,
oracle, stress probe or status refresh ran. Compilation, focused semantic
execution and broad verification remain mandatory before runtime acceptance.
