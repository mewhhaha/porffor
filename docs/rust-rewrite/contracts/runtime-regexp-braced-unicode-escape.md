# Computed RegExp braced Unicode escapes

The computed Pattern compiler now admits braced Unicode code-point escapes in u
and v through its existing validated character-mode owner. The pre-scan already
validated the body; this batch removes only the complete braced run's taint
assignment and implements its actual parser consumer. The follow-on code-point
property extension shares that parser; nested v classes, set algebra and class
strings retain their separate source gates. No new runtime helper, table service, matcher instruction,
program representation, descriptor, data pool or ABI is introduced.

The private Unicode-escape decoder borrows CompilerCharacterMode and owns the
fixed/braced dispatch. Its braced arm checks a nonempty hexadecimal body, closing
brace and value at most 0x10FFFF, then consumes the complete escape as one atom.
Only the fixed arm invokes the existing fixed/fixed surrogate pairing helper.
Raw/fixed pairing is unchanged; braced surrogate values never pair with raw,
fixed or braced neighbors. Unicode bitmap/literal/range consumers use the same
full character domain already owned by the previous source batch. Legacy still
uses its identity-u fallback and subsequent ordinary literal/quantifier parsing.

The grammar permits zero, leading zeros, the maximum code point and surrogate
code points. Separators, signs, empty bodies, nonhexadecimal characters, missing
closure and larger values reject. Each accepted prefix is bounded, so the next
multiply/add remains within u64 even with arbitrary leading zeros. The cursor
advances past the brace before ordinary quantifier or class-range handling.
Decoded syntax characters are character values; they do not become raw v syntax
or reserved-double punctuation. Existing endpoint classification and scalar
ordering still own range errors. The current ordinary v pre-scan's existing
range-policy limits remain unchanged; this does not claim complete v grammar.

The same taint removal exposes braced spellings in Unicode GroupSpecifier names.
That consumer already existed in the emitted name decoder and remains byte-identical:
it validates identifier start/continuation membership, rejects surrogate names,
canonicalizes the name payload and publishes it through the completed capture
inventory. The follow-on [named Unicode reference extension](runtime-regexp-named-unicode-reference.md)
admits references following these names through the same completed capture
inventory. Neither extension adds a name service or alternate capture authority.

[ECMAScript 2026 Patterns](https://tc39.es/ecma262/2026/multipage/text-processing.html#sec-patterns)
defines braced escapes as a separate Unicode production whose character value is
the code-point value, and pairs only the fixed lead/trail production. Its shared
[CodePoint grammar](https://tc39.es/ecma262/2026/multipage/ecmascript-language-lexical-grammar.html#prod-CodePoint)
uses hexadecimal digits without separators and bounds the value inclusively by
0x10FFFF. Existing static atom/class parsing follows these distinctions too.

Two finite Engine sources use code-unit arrays with the existing fromUnits prelude,
then ordinary constructor/recompile/exec/test paths in both sloppy and strict
contexts. They cover null, leading zeros, mixed hex case, max/lone surrogates,
quantifiers and UTF-16 indices, class/range/case/complement consumers, all new
cross-form separations, encoded syntax characters, braced names and invalid
identifiers, body/range syntax errors, failed receiver rollback and successful
recompile. Property controls now observe matching; class-string controls remain; the former named-reference
gap control is replaced by the follow-on extension's finite normal/syntax controls.
No source selector or alternate materializer is added to production.

All changes are authored source. Formatting, hashes and ordinary forward/inverse
patch application do not establish typechecking, Wasm validity or semantic results.
No Cargo, compiler, runtime, tests, validator, oracle, stress/reproducer, generator
or status refresh ran. Focused Engine/runtime compiler regressions, existing mode,
name/class/folding and capability controls, then combined broad verification remain
mandatory at the later checkpoint.

The subsequent [code-point property batch](runtime-regexp-codepoint-property.md)
admits complete property aliases and ranges through this same parser. Its
separate v string/algebra capability boundaries remain explicit.
