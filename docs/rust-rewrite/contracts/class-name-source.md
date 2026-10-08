# Class source name and inferred label

`SourceClassName` is the private AST evidence consumed by class analysis and
lowering. A display label inferred for an anonymous class does not create a
class-name binding. Class expression `name_scope` records the parser's actual
BindingIdentifier; NamedEvaluation changes the label without changing that
scope. Class declarations use their analyzed own name scope. The parser's
synthetic `default` label is display-only because that keyword cannot be a
source BindingIdentifier.

The owner has no public raw-name constructor. Its private domain distinguishes
anonymous classes, inferred labels and actual source bindings, retaining the
original Identifier span only in the binding case. Analysis derives class-name
environment and capture aliases from that case. Lowering establishes the same
inner binding only in that case and still passes the original display label to
constructor metadata. Static element inference may update class-name facts
only when that actual binding exists.

This preserves the separate `classBinding` and `className` arguments of
[ClassDefinitionEvaluation](https://tc39.es/ecma262/multipage/ecmascript-language-functions-and-classes.html#sec-runtime-semantics-classdefinitionevaluation).
Anonymous NamedEvaluation preserves the outer binding's value and TDZ;
explicit class names keep the immutable inner cell. Heritage and computed-key
suspension use the existing class evaluation plan and existing GC environment
owners. No new execution representation or name patching is introduced.

Three IR controls inspect real lowered classes, their method capture cells and
their resumable class-name environment ownership. One paired strict/sloppy
Engine cohort observes static initialization and display-name descriptors,
SingleName lexical default TDZ and abrupt cutoff, assignment defaults, explicit
inner bindings, live outer captures, heritage suspension and GC. These controls
are authored but unrun in this source batch; compilation and Wasm execution
remain required at the coordinated verification checkpoint.
