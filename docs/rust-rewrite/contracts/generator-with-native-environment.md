# Native ordinary-generator With environment

`OrdinaryGeneratorWithIr` consumes the complete source head and body ranges,
the retained ToObject result and the original analyzed Object Environment
Record. The head belongs to the outer environment. Its operand captures and
yields use the existing StatementList completion owner with source-value
checkpointing suppressed only for that head region. The prior active state is
restored before body evaluation; no caller manufactures an Empty-source proof.

Fresh body entry allocates the original Environment and hidden BindingCell
once, publishes the completed ToObject value in that cell, and supplies
Undefined for With's empty completion. A resumed body reattaches the same
physical record from `InvocationFrame.LEXICAL_ENVIRONMENT`; the existing
parent-chain reattachment accounts for deeper saved lexical scopes. Closures
and captured Identifier References retain the real record and original object.
No JavaScript environment encoding or alternate environment model is used.

A completion cleanup Block exists before the complete body can inject a
resumed Return or Throw. Inner finally scopes and retained Array patterns
finish first. Outward Throw, Return, Break and Continue reach this cleanup;
branches to targets inside With keep its environment active. Ordinary Yield
and delegated done:false publish the existing saved lexical record and return
before cleanup. Fresh and resumed normal body completion use the same tail.

The tail advances the source exit, leaves the original lexical record and
saves the restored outer environment before dispatching the complete body
Completion. Pure environment operations preserve its kind, value and target.
The enclosing catch/finally observes the outer environment, while closures
created inside With continue to retain the original object cell. Native entry,
exit and direct lexical-instantiation walkers consume the checked body.

Two authored native artifact controls validate the actual Wasm GC environment,
object-record and hidden-cell topology and saved frame field operations. One
uses the full semantic Engine cohort; the other combines nested With, lexical
closures, retained Array patterns, delegation and outward loop finalizers.
Semantic Engine controls own fresh/resumed identity, dynamic unscopables,
selected Identifier Put, boxing, TDZ and whole-completion behavior. Compilation
and tests remain deferred until the coherent batch is complete.
