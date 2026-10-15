# Labelled functions in lexical declaration lists

Authority: ECMA-262 `5345883164f463e87f8b40aca4956157ecba8783`,
LexicallyDeclaredNames and LexicallyScopedDeclarations,
TopLevelVarDeclaredNames and TopLevelVarScopedDeclarations, and labelled
FunctionDeclaration evaluation.

A Block or switch CaseBlock instantiates the lexical declarations of its
statement lists. One label whose immediate LabelledItem is FunctionDeclaration
contributes that declaration. A label whose immediate item is Statement
contributes an empty list, including when that Statement is another label.
Therefore `a: function f() {}` receives the ordinary block/case lexical binding,
while `a: b: function f() {}` does not create a lexical binding there. Reading
`f` in the latter case follows the surrounding live environment chain and
throws ReferenceError when the reference is unresolvable. Evaluation of the
function declaration itself returns an empty completion; it does not create
a missing lexical binding.

Script and function body top-level declarations use different operations.
TopLevelVarDeclaredNames and TopLevelVarScopedDeclarations explicitly recurse
through a LabelledItem containing another labelled Statement. Their normal
hoisting of arbitrary label chains must remain intact.

The compiler uses separate selectors for lexical Block/CaseBlock declarations
and general labelled-declaration traversal. The lexical selector also drives
capture aliases and named environment records, so a closure or a finite direct
eval observes the same source binding that ordinary code resolves. A chain
cannot mask a surrounding lexical, catch or with-object binding by inventing
a block cell. An ordinary direct declaration sharing the chain's spelling
retains its own initializer.

The storage-plan inventory follows the same lexical selection. A rejected
outer-variable copy is not by itself sufficient: retaining a nonlexical
chain's synthetic block-storage name in the activation would still publish an
uninitialized cell to direct eval. The actual outer binding would then share
its source name with that cell. Block and CaseBlock plan collection therefore
exclude nonlexical label chains before assigning storage or named-record slots.

This repair does not extend Annex B's optional outer-variable copy scan through
labels. Outer-copy eligibility and copy evaluation are unchanged. The separate Mozilla staging fixture expectation conflict remains
recorded in `target/annex-b-labelled-normative-disposition-0a18df332-20261014`;
neither held copy patch is applied. HostSurfacePolicy selects exposed host
globals and does not select a declaration-semantics dialect.

Focused native controls cover block/case lexical depth, top-level chains,
single-labelled declarations, ordinary redeclarations, closures, finite direct
eval, catch and with records, and the unmodified pinned labelled syntax fixture.
The preparation packet records which controls actually ran. Source preparation
and noncompiling checks alone are not native or full-suite acceptance.
