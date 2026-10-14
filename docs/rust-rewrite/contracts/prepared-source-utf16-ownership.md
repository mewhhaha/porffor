# Prepared source owns exact UTF-16 units

`eval`, Realm Script evaluation, ShadowRealm evaluation and the four Function
constructor grammars receive ECMAScript strings. Their compilation candidates
use the same lossless string codec as emitted string data. A candidate's codec
spelling is transport data; it must never enter a UTF-8 source lexer unchanged.

`PreparedSourceCache` decodes at its typed input boundary. Its cache key owns the
actual UTF-16 units, the Script/eval/Function grammar, the Function kind, and each
DirectEval parse context. Equal semantic units reuse one successful or rejected
parse in that context. The cache remains local to one host compilation and
retains the actual AST/interner allocation identity. Constructor argument tuple
order and all isolated fragment parses are preserved.

Front prepares those inputs through `Source::from_utf16`. Boa's source collector
is retained beside its AST/interner and supplies the exact UTF-16 coordinate
owner for callable source slices. Linear spans are never applied to encoded
marker text. Slice publication and Function constructor source metadata encode
exact semantic units once. Literal marker characters are escaped by the codec
as ordinary user data, including when source candidates are concatenated.

`SourceUnit.encoding` distinguishes ordinary UTF-8 source bytes from encoded
UTF-16 metadata. Ordinary host Script and Module files retain their exact UTF-8
bytes and original module byte-range maps. The new UTF-16 parse entry is
Script-only, so an encoded module cannot be passed to UTF-8 module-link owners.
For UTF-16 parse failures, line/column traversal uses actual source code points;
diagnostic byte spans then identify the corresponding encoded metadata range.

This changes compilation-local source preparation. It adds no parser, VM or
interpreter to emitted Wasm. Runtime argument evaluation, conversion, tuple
matching, candidate budgets and deferred SyntaxError ownership remain intact.
The RegExp parser and matcher are unchanged by this repair.

The source-bound 32c197968 diagnostic failed original raw-Unicode assertion 7
in both modes: a fixed escaped lead plus raw trail matched null in legacy mode.
Its paired matrix first failed source-unit case 4, path 0 (prepared eval). The
original 56 assertions, all four original execution IDs, the existing paired
matrix and runtime-only native control remain unchanged. The new controls cover
UTF-16 lexical ownership and diagnostic spans; all four Function fragment goals;
callable source slices after lone-unit comments; semantic cache reuse; native
callable metadata and repeated activations; nested eval; and marker user data
across eval, Function, ShadowRealm and host Realm Script evaluation.

Native acceptance belongs to the subsequent source-bound verification owner,
not formatting, private apply checks or the earlier diagnostic probe.
