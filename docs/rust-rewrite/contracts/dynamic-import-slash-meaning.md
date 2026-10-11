# Dynamic-import slash meaning

Valid dynamic imports are identified and ranged by the original parser. Script
discovery walks the retained AST; source rewriting maps the recorded keyword/phase
ranges through prior edits. Property and method names cannot become call sites.
The remaining scanner supplies conservative evidence only after Script parsing
has failed. A possible import or an uncertain scan produces `Indeterminate`; it
cannot admit source as dependency-free. Its slash-state contract follows.

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

The structure guard fixes the private declaration, its exact 20 owner mentions,
the exact nine `Divide` and seven `Regexp` producers, every producer mapping, comment-before-dispatch
ordering, comment-state preservation and both semantic bodies. Focused units
exercise a regexp containing an apparent import call, a division expression
followed by an import call, a call in a template substitution and the complete
linker rewrite.

Focused commands:

```sh
cargo test -p lila-ir --test dynamic_import_slash_meaning_structure --quiet
cargo test -p lila-front --test module_source_ranges --quiet
cargo test -p lila-ir --lib modules::dynamic::tests::import_calls_inside_literals_and_comments_are_left_alone -- --exact
cargo test -p lila-ir --lib modules::dynamic::tests::division_slash_does_not_consume_the_following_import_call_as_a_regexp -- --exact
cargo test -p lila-ir --lib modules::dynamic::tests::a_call_site_inside_a_template_substitution_is_rewritten -- --exact
cargo test -p lila-ir --lib modules::link::tests::dynamic_import_is_desugared_into_a_dispatcher_call -- --exact
```

Independent review confirmed the complete scanner-body and lexical-state
census. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the
module-boundary check and the task-plan check. The compile retains the
repository's existing warnings; broader Test262 module verification was not
rerun.

## Nonclaims

The original two-state scanner repair changed no JavaScript lexical grammar,
module resolution or dynamic-import scheduling. The subsequent parser-range
repair removes this scanner from valid-source rewriting and discovery. Neither
repair establishes completion of the remaining T12 module graph work.

Static and dynamic module rewriting now consume ranges retained by the original
parser. Controls preserve regex, method names, nested calls, UTF-16, phase-head
line terminators and ASI boundaries without repeating its lexical decisions.
The scanner above remains only in the conservative parse-failure probe.
