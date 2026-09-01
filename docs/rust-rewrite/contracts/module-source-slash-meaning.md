# Module-source slash meaning

The module-syntax rewriter must distinguish division punctuators from
regular-expression literals without treating an unrecognized scanner state as
either one.

## Closed domain

`SlashMeaning` has exactly two private rows:

- `Divide` means the preceding significant token can end an expression;
- `Regexp` means it cannot, so `/` begins a regular-expression literal.

The scanner constructs that state at each significant token boundary. It has no
derived capabilities or default: the only semantic consumer borrows the state
and matches both rows exhaustively.

Line and block comments are recognized before this decision. In `Regexp` state,
the scanner consumes the complete literal and enters `Divide` state. In
`Divide` state, it consumes only the slash punctuator and enters `Regexp` state,
matching the operator transition used before the domain was closed. Both paths
clear the property-name context after consuming the slash.

## Durable regressions

The structure guard fixes the private two-row declaration, its exact 23 owner
mentions beside the dynamic-source scanner's separate 18-mention domain, the
exact nine `Divide` and ten `Regexp` producers, every producer mapping,
comment-before-slash ordering, comment-state preservation, and both exhaustive
semantic bodies. There is one additional `Regexp` producer because the
divide-slash arm makes explicit the operator transition that previously fell
through the generic punctuator arm. Owner unit tests exercise a regexp
containing a quote, a division expression followed by an export declaration,
line and block comments in both slash contexts, and ECMAScript line
terminators in comments. The dedicated structure target passes `3/3`, all four
focused owner units pass, the full formatting check is green, and independent
review found no remaining invariant gap.

## Nonclaims

This closure changes no JavaScript lexical grammar, module edit, byte-offset or
line-terminator behavior. It does not replace the module rewriter with the
parser, broaden supported module syntax or close the remaining T12 module
graph work.
