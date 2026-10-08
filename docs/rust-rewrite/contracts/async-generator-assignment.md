# Mixed async-generator assignment References

Mixed `await`/`yield` ordinary, private and `super` property assignments consume the
existing Reference and native Get/Set operations. `RetainedMixedAssignmentReference`
is a closed, non-cloneable owner shared with identifier logical assignment.
An ordinary compound or logical assignment captures its original receiver,
boxed target and canonical property key through `OrdinaryPropertyGetCapture`.
Only its selected arm consumes the resulting captured write authority. The
raw key is never reconstructed or converted again after the RHS.

The condition phase of logical assignment includes the actual source base and
computed key. It uses the same mixed branch allocator as other conditional
values, followed by disjoint selected and skipped RHS ranges. Private operands
retain their original base and use the existing private-name identity and
brand/accessor operations. Plain private assignment does not introduce a Get
or move the brand check ahead of the RHS.

Get failures stop before the RHS. Eager arithmetic conversion follows both
operand values; Set follows the arithmetic result. A skipped logical arm
evaluates no RHS and performs no Set. Injected Throw/Return at a suspended RHS
uses the enclosing generator completion machinery and cannot execute PutValue.
Accessor effects discard stale shape facts while retaining whole runtime values.

The AOT fixture covers sloppy and strict property ordering, canonical keys,
receiver identity across suspension and GC, skipped/selected logical arms,
private accessors and brand failures, whole injected completions, and a
Number/BigInt abrupt completion before Set. These controls are authored but
have not run at this source checkpoint.

The existing `SuperPropertyMutation` carrier now also owns a checked capture
and a later consuming Put. Its private capture holds three activation cells:
the original receiver, original Super base, and referenced name. Role-specific
slot types and a non-cloneable lowerer authority prevent substituting the base
after the RHS. `ReadBeforeRhs` runs the existing native Get/canonicalization
before suspension; `WriteOnly` retains a raw key and permits a null base until
the original PutValue after the RHS. The shared native canonicalization and
Set helpers consume the saved base without reloading the HomeObject prototype.
Sloppy/strict controls change that prototype during the RHS, mutate a raw key,
exercise borrowed receivers and skipped logical arms, and distinguish plain
assignment's late null-base error from compound assignment's early Get error.

Sloppy Annex B Call assignment targets use the same staged Call and original
ReferenceError operation as Call update targets. The source tape owns only the
actual callee/arguments: the right side cannot run after that mandated error.
Controls include a suspended argument, compound assignment without value
coercion, an unreachable suspended RHS and an injected argument Throw. Strict
Call assignment targets and logical Call targets remain frontend early errors,
as required by [assignment evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-assignment-operators-runtime-semantics-evaluation).
