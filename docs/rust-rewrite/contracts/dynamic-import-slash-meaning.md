# Dynamic-import slash meaning

The dynamic-import call-site scanner must distinguish division punctuators from
regular-expression literals without treating an unrecognized scanner state as
either one.

## Closed domain

`modules::dynamic::SlashMeaning` has exactly two private rows:

- `Divide` means the preceding significant token can end an expression;
- `Regexp` means it cannot, so `/` begins a regular-expression literal.

The scanner constructs that state at each significant token boundary. It has no
derived capabilities or default. Its semantic consumer borrows the state and
matches both rows exhaustively.

Line and block comments are recognized before the decision. In `Regexp` state,
the scanner consumes the complete literal and enters `Divide` state. In
`Divide` state, it consumes only the slash punctuator and enters `Regexp` state,
which is the same operator transition previously supplied by the generic
punctuator arm. Both paths clear the property-name context.

## Durable regressions

The structure guard fixes the private declaration, its exact 20 owner mentions
beside the module-source scanner's 23-mention domain, the exact nine `Divide`
and seven `Regexp` producers, every producer mapping, comment-before-dispatch
ordering, comment-state preservation and both semantic bodies. Focused units
exercise a regexp containing an apparent import call, a division expression
followed by an import call, a call in a template substitution and the complete
linker rewrite.

Focused commands:

```sh
cargo test -p lila-ir --test dynamic_import_slash_meaning_structure --quiet
cargo test -p lila-ir --test module_source_slash_meaning_structure --quiet
cargo test -p lila-ir modules::dynamic::tests::import_calls_inside_literals_and_comments_are_left_alone -- --exact
cargo test -p lila-ir modules::dynamic::tests::division_slash_does_not_consume_the_following_import_call_as_a_regexp -- --exact
cargo test -p lila-ir modules::dynamic::tests::a_call_site_inside_a_template_substitution_is_rewritten -- --exact
cargo test -p lila-ir modules::link::tests::dynamic_import_is_desugared_into_a_dispatcher_call -- --exact
```

Independent review confirmed the complete scanner-body and lexical-state
census. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the
module-boundary check and the task-plan check. The compile retains the
repository's existing warnings; broader Test262 module verification was not
rerun.

## Nonclaims

This closure changes no JavaScript lexical grammar, rewritten source bytes,
module resolution, dynamic-import scheduling or emitted Wasm. It does not
replace the scanner with the parser, broaden supported dynamic-import syntax or
close the remaining T12 module graph work.
