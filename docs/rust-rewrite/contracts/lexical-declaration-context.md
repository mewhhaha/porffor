# Lexical declaration context

The parser must distinguish an ordinary lexical declaration from an ambiguous
`for` head before it decides initializer and terminator requirements or emits
declaration-level early errors.

`LexicalDeclarationContext` is the private two-row authority:

- `Statement` requires an ordinary declaration terminator, requires initializers
  for `const` and resource declarations, and immediately runs the lexical
  bound-name validators;
- `ForHead` permits the initializer and terminator to remain absent and defers
  the validators until the surrounding `for` parser resolves classic versus
  iterable grammar.

The domain and its `LexicalDeclaration` and `BindingList` carriers derive no
cloning, copying, debugging, equality or default capability and have no manual
implementation of those traits. Their existing inherent and `TokenParser`
implementations remain. `BindingList` borrows the context, and the three
grammar decisions match both rows directly and exhaustively. There is no
Boolean projection, equality test, wildcard or default behavior.

Exactly one declaration parser constructs `Statement`; the three `let`,
`const`, and resource-declaration branches in the `for` parser call the single
`ForHead` constructor. The recursive structure guard fixes all thirteen owner
mentions, both constructor mappings, those external call counts, all four
borrowed binding-list entries, and the exact error and validation order of all
three decisions. It also fixes the initializer decision before the semicolon
probe and the missing-terminator decision after that probe.

This is source-equivalent parser control flow. It changes no grammar,
diagnostic wording, token consumption, error precedence or accepted program.
The focused structure and parser witnesses are green. The direct format check
of the vendored parser file remains red on pre-existing formatting outside this
lane; the three touched exhaustive-match regions are clean. This contract does
not claim that the complete vendor file is format-clean.

Independent review confirmed the exact adjacent items, manual-capability
exclusion, recursive four-route producer census and all three decision orders.
The coordinated workspace checkpoint passes `cargo fmt --all -- --check`,
`cargo xc`, `git diff --check`, the module boundary check and the task-plan
check; the compile retains the repository's existing warnings.
