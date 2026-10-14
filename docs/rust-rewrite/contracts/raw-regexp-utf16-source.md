# Raw UTF-16 RegExp source ownership

ECMAScript Pattern input is a sequence of UTF-16 units. A raw adjacent lead/trail
pair becomes one PatternCharacter in Unicode mode, but mixed raw/escaped units
remain separate atoms. Legacy mode consumes both units independently and a
following quantifier belongs to the trailing atom. Class membership follows the
same grammar domains; a legacy class searches the first input unit it contains.

RegExp literal lowering uses the Interner's semantic units, through the ordinary
lossless StringPool codec. Interner Display is diagnostic text and cannot supply
an executable Pattern. Static literal/constructor lowering and finite runtime
candidate tables decode that codec before calling `RegExpProgram::compile_utf16`.
The existing `compile(&str, flags)` entry remains literal UTF-8 user input, so a
real U+F0000 followed by hexadecimal letters cannot become a marker instruction.

`Utf16PatternSource` owns UTF-8 scalar bytes and WTF-8 lone-surrogate bytes,
constructed only from semantic UTF-16. Source decoding reads exactly one encoded
character rather than validating its entire remaining tail as UTF-8. It does not
rewrite raw units into fixed or braced RegExp escapes. Fixed escaped-pair
admission and braced-escape atom ownership remain in the existing grammar.
Property names retain their ASCII-only grammar; a raw surrogate in a property or
capture name is an actual syntax rejection rather than a source-decoding miss.

The runtime parser and matcher, which already consume real code units, are
unchanged. No source-shape exception, candidate-budget change, limit increase,
VM, or harness bypass is used. IR controls check atom/quantifier/class domains
and the lossless codec; native controls compare direct scalar literals, prepared
UTF-16 literals, constructors and separate runtime-only misses. The original
staging bodies and canonical harness are copied byte-for-byte into a control
fixture, with their original source files and hashes retained separately.

Acceptance still requires the unchanged original Test262 execution IDs in both
modes, the actual Engine controls, and retention of accepted RegExp-family IDs.
A diagnostic transcript or a private unexecuted patch is not native acceptance.
