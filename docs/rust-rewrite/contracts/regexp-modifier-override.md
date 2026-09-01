# RegExp modifier override

Status: implemented and focused-verified for the `m` and `s` state carried by
RegExp modifier groups.

## Boundary

`RegExpModifierOverride::{Inherit, ForceOn, ForceOff}` is the sole Rust domain
for the local multiline and dotAll behavior selected by a modifier group.
`Modifiers` cannot spell that three-state policy as `Option<bool>` and cannot
be copied implicitly.

The initial parser state names `Inherit`. Adding or removing `m` or `s` names
`ForceOn` or `ForceOff`, while inline exhaustive matches carry unaffected
fields into a nested scope. The parser moves the outer state out, captures the
nested parse result, restores the outer state and only then propagates an
error.

One exhaustive `operand_code` projection owns the bytecode ABI:

| Override | `operand0` |
| --- | ---: |
| `Inherit` | `0` |
| `ForceOn` | `1` |
| `ForceOff` | `2` |

The inherent dot and start/end assertion constructors name `Inherit`, and the
modifier application rows replace it from the dotAll and multiline fields.
The Wasm matcher compares the encoded operand with the same typed `ForceOn`
and `ForceOff` codes, and otherwise retains the pattern's runtime flag.
Bytecode storage remains an integer ABI; no Rust producer writes an anonymous
override code.

## Durable evidence

`regexp_modifier_override_structure.rs` pins the non-copyable three-variant
domain, exhaustive nested-scope and ABI projections, initial/add/remove
producer census, restoration-before-propagation order, the three inherent
constructor sites, both IR encoder rows and the ordered typed Wasm decoder.

Focused IR tests compile enabled and disabled dotAll and multiline groups and
a nested dotAll group whose inner override, restored outer override and final
inherited state produce codes `2`, `1` and `0` in that order.
`wasm_regexp_modifier_overrides.js` exercises the same forced modes and nested
restoration through the product Wasm-AOT matcher.

## Verification

The structure target passes `5/5`; the forced-mode and nested-restoration IR
tests each pass `1/1`; the focused Wasm-AOT CLI fixture passes `1/1`; and
`cargo xc` is green. The exact pinned `add-dotAll.js` leaf was also attempted
and remains `0/2` with `NotImplemented:Runtime` because its broader
`RegExp.prototype.exec` pattern is unsupported. The other broad modifier leaves
were not rerun after that representative failure. No semantic golden was run;
the instruction operands and matcher instruction sequence are unchanged.

## Deferrals

This contract does not change top-level RegExp flag parsing, modifier-group
syntax or early errors, `i` case-folding, dynamic pattern compilation,
malformed bytecode handling or broader RegExp conformance.
