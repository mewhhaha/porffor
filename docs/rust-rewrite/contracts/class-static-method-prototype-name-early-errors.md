# Class-static-method literal `prototype` early errors

## Decision

A public static method definition whose literal property name is `prototype`
is one closed pre-evaluation condition:

`EarlyErrorCode::ClassStaticMethodPrototypeName`

The code covers ordinary, generator, async, async-generator, getter and setter
method definitions because all six consume the same ClassElement early-error
rule. Its sole wire spelling is `E_CLASS_STATIC_METHOD_PROTOTYPE_NAME`.

## Measured parser boundary

Pinned `boa_parser-0.21.1` has six producer branches in
`parser/statement/declaration/hoistable/class_decl/mod.rs`. Every branch emits
the exact, case-sensitive message:

```text
class may not have static method definitions named 'prototype'
```

One complete-message classifier row owns those producers. Script and Module
goals report `Early`, `SyntaxError`, the typed code, and the parser source span;
retained dependency parsing projects the same diagnostic.

## Name boundary

The restriction uses the syntactic literal property name. Non-static methods
named `prototype` remain valid. Computed public static names such as
`static ['prototype']() {}` also remain parse-valid because their key is
evaluated later. If that computed key resolves to `prototype`, the separate
class-definition runtime guard rejects installation against the constructor's
non-configurable own `prototype` property with `TypeError`; that T09/T10
evaluation rule must not be reclassified as an early error.

Private names and static fields are separate productions. The existing static-
field code owns literal `prototype` fields; this code owns only public method
definitions and accessors.

## Verification boundary

Front-end tests cover all six method forms as declarations and expressions
under Script and Module goals. Positive controls cover non-static literal names
and all six computed static forms. A retained Module parse must carry the same
typed code, early phase, `SyntaxError`, and source span.

The exact pinned Test262 cohort is the twelve
`grammar-static-{meth,gen-meth,async-meth,async-gen-meth,get-meth,set-meth}-prototype.js`
files under both `language/expressions/class/elements/syntax/early-errors` and
`language/statements/class/elements/syntax/early-errors`. Their metadata expands
to 24 sloppy/strict Wasm-AOT executions. This bounded diagnostic family does
not claim class-method execution, computed-key installation, or broad T07/T09
closure.

At `2026-08-23`, capped serial verification passes the complete front-end gate
at `51/51`, the focused IR early-error gate at `3/3`, and the exact twelve-file
cohort at `24/24` Wasm-AOT executions. Every failure and non-success bucket is
zero. The workspace `cargo xc` check is also green.
