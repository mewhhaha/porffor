# Fresh Script restricted global lexical admission

Source proposal prepared on 2026-10-03. Compilation and runtime verification
remain pending.

Fresh Script global declaration instantiation must reject a lexical name when
the pre-existing global object's own property is non-configurable. The closed
`GlobalPropertyInitializerIr` descriptor projection makes `Infinity`, `NaN`
and `undefined` restricted. A matching global `let`, `const` or class
declaration emits an ordinary runtime `SyntaxError` through the existing
global declaration failure completion. Parsing and IR lowering still admit
the Script.

The fresh-entry check runs after runtime roots and native error prototypes
exist, before allocation or publication of global lexical entries and before
source global variable/function installation or body evaluation. Global
source variable/function and lexical names cannot overlap in
`GlobalBindingPlan`; therefore an overlapping object binding at this point
is a pre-existing descriptor, not a newly installed declaration. Error
construction uses the realm intrinsic, not a replaceable public constructor.

Configurable global properties may be shadowed by global lexicals. Block,
function, Module and eval-local bindings may use the restricted names.
Prepared realm Scripts retain their existing runtime descriptor validation,
including custom non-configurable properties; this proposal does not change
prepared Script or eval admission.

`aot_fresh_script_global_lexicals` retains the untouched pinned
`language/global-code/decl-lex-restricted-global.js` runtime-negative fixture
in both Script modes. Its controls cover body/initializer effects, the three
declaration forms, safe names, configurable builtin shadows, realm error
identity, prepared descriptor admission, eval scope and Module scope.

Future batch verification must compile the integrated source, run this target
and the existing `aot_prepared_global_declarations`,
`aot_declaration_completion`, `aot_execution_global_environment` and Module
scope/completion regressions, then rerun both pinned fixture modes and the
required broad checks. This proposal changes no published conformance counts
and claims no runtime pass or T08 completion.
