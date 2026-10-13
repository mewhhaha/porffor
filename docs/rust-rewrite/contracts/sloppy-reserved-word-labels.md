# Sloppy reserved-word labels

The authority is ECMA-262 commit
`5345883164f463e87f8b40aca4956157ecba8783`, the `LabelledStatement`,
`LabelIdentifier` and `Identifier` grammar and their strict-mode early errors.
`let` and `static` are valid decoded identifiers in sloppy label positions.
Unicode escapes do not change the identifier or remove strict-mode restrictions.
Script directives, strict function bodies, class methods and Module code must
reject both plain and escaped spellings.

The actual `aee6bc109927c580393d59c0cf6e532e30efd4b3` historical replay finds
three Runtime/Bug records while preparing statically known Function bodies:

- `sloppy-script:staging/sm/syntax/escaped-let-static-identifier.js`
- `strict-script:staging/sm/syntax/escaped-let-static-identifier.js`
- `sloppy-script:staging/sm/syntax/let-as-label.js`

Their raw snapshot is under
`target/historical-native-replay-aee6bc109-20261013/snapshots/`.
The parser rejects a valid sloppy `let:` at its colon, before the normal label
grammar can run. Its statement lookahead handles IdentifierName, `await` and
`yield`, but omits the lexer's `Keyword::Let` token. Both plain and escaped
`let` have that token kind; `static` already follows IdentifierName.

The repair admits `Keyword::Let` only to the existing colon lookahead and then
uses the ordinary LabelIdentifier parser. That parser retains strict checks;
the preceding statement-list lexical-declaration lookahead remains unchanged.
No source rewrite, fixture exception or new dynamic source facility is needed.

Paired frontend controls cover decoded labels and break/continue targets,
strict directives with semicolons and automatic semicolon insertion, strict
function/class/Module contexts, lexical declarations, identifier assignments
and duplicate decoded labels. Existing Annex B labelled-function restrictions
remain part of the affected inventory. The three original Test262 executions
must be replayed unchanged after the coherent batch compiles. This prepared
repair has no native acceptance verdict yet.
