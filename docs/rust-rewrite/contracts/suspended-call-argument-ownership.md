# Suspended call Reference and argument ownership

Status: source implementation and regression fixtures authored; compilation and runtime verification pending. No PASS or Test262 count change is claimed.

For admitted eager expressions containing `await` or generator `yield`, Call, `new`, and tagged-template lowering owns an `EvaluatedCallReference` before it lowers arguments. Acquisition stores GetValue's callee and Reference-derived receiver in the existing suspension-owned activation bindings. The owner is private, neither Clone nor Copy, and is consumed by the invocation. Acquisition does not check IsCallable or IsConstructor; the existing actual call/construct emitter performs rejection after ArgumentListEvaluation.

Property acquisition uses ordinary GetV/private/super lowering, preserving computed-key conversion, getter/Proxy effects, and abrupt completion. Plain `with` resolves the existing ordered HasBinding/unscopables selection once, stores its selected base, then performs Object Environment GetBindingValue's independent HasProperty recheck. Runtime environments use `CaptureCallReference` to consume the actual ResolveBinding/GetValue/call-base result and save the receiver through existing BindingStorage. Neither path reconstructs resolution after await. Original syntax and acquired callee identity retain the direct-eval capability guard and prepared-source context.

Tagged-template acquisition precedes GetTemplateObject, which precedes substitutions. T13's actual parsed-unit/defining-Realm cache remains the template owner. Constructors acquire the callee while discarding the Reference's call receiver at invocation.

Arguments are staged in source order. Ordinary argument values are saved before subsequent arguments. A spread runs the existing backend ArgumentListEvaluation immediately, including iterator acquisition, cached NextMethod, done/value reads, append, and abrupt completion. The argument spread protocol does not add IteratorClose. A non-Clone `ArgumentListCapturePlan` constructs both the stored capture and its private `CapturedArgumentListIr` use. Payload fields are private; no constructor accepts an ordinary JS array. The real `emit_call_args_vector` consumer copies complete own vector values without iterator/prototype/accessor lookup. Captured vectors remain compiler-private and use the existing argv representation, without a second object model or new heap-record schema.

Exhaustive visitors cover both capture and captured-list expressions; runtime receiver binding is an explicit operation operand. Collection roots, builtin iterator dependencies, throw inference, effect analysis, global property collection and temporary planning traverse the actual producers. Dynamic-source preflight distinguishes template arguments from already-accounted invocation facts so a runtime string-source gap still produces its typed unsupported diagnostic.

Invocation staging consumes the existing checked conditional, logical and
optional Property/Call value plans. Call-bearing tails consume their
[checked source owner](optional-call-await-ownership.md) outside all loops and
reuse this invocation owner after their optional nullish guard. Grouped optional
callees/tags ending in a property now use the consumed
[terminal Reference owner](grouped-optional-reference-await-ownership.md) outside
all loops. Intermediate Gets retain their values, while the terminal actual
chain captures callee and raw receiver together before outer operands. The
exhaustive Call/Construct purpose keeps constructors on their value path.
Grouped terminal Calls now consume a checked completed Value with no outer
receiver through the [Call Value owner](grouped-optional-call-value-await-ownership.md),
including target-only suspension while preserving an inner grouped method
Reference. Mandatory preflight validates the corresponding terminal kind before
states. Target-only awaited terminal Property References and Call-bearing tails
in loops remain refused. For synchronous chains, a private `OptionalCallReferenceCaptureIr` consumes the existing lowered optional-chain owner. Its actual emitter publishes the final Reference receiver or undefined from the existing reference/short-circuit locals before the outer arguments. Parenthesized optional calls and tags retain admission: internal nullish branches skip their keys/calls, while the outer unconditional arguments still run before callability rejection. A final inner Call resets the receiver to undefined. Constructors discard the saved receiver at invocation. `super()` argument staging, mixed Await/Yield invocations and unowned branch
operands retain their explicit unsupported boundaries. Bounded ordinary
generator values now compose through their checked structural
[source plan](plain-generator-value-branch-ownership.md); suspended loop and
async-generator branches remain separate work. This packet does not add a parser, interpreter, VM, source execution fallback, or broad conformance claim.

Pending focused selector: `cargo test -p lila-engine --test aot_generators -- aot_suspended_call_references::`. Seventeen source fixtures cover method replacement and key conversion, getter/key/Proxy abrupt completion, noncallable/nonconstructor argument order, spread side effects and iterator replacement, spread abrupt completion, constructor identity, private/super methods, tag receiver/frozen arrays, direct eval's original identity, plain/runtime with-environment References, and grouped optional admission, conditional receivers, nullish outer argument evaluation, getter/key abrupt completion, post-call receiver reset and tags. No fixture has executed.


The shared private `InvocationSuspension::{Await, Yield}` choice selects the
existing suspension prefix/state producers. Source-plan preflight checks the
complete staged operand shape before an invocation consumes generator states;
literal prototype setters rejected by the staged object lowerer are refused
using the parser's canonical symbol at admission. Direct Yield targets stage
nested invocation operands before their outer suspension. Consuming pin
restoration retains the enclosing map on failure. Later argument effects clear
earlier retained values' heap-shape facts, preserving saved identity, and
private/super getter effects invalidate caller facts before arguments.

The ordinary-generator optional chain producer consumes the same completed
Reference, evaluated Value/Spread argument capture and final Call owners as
Await staging. An explicit Await/Yield choice governs first-Call target capture.
Several yielded operands use flat scalar guarded steps in source order; a
completed constructor chain supplies only its callee Value. A grouped yielded
terminal Call now consumes the full checked chain Value before the receiver-free
outer callee pin and unconditional arguments or original template/substitutions.
The same actual source planner owns both guarded chain and outer operand states.
Terminal-Property chain References retain their separate boundary. See the
[grouped Call yield owner](grouped-optional-call-value-yield-ownership.md).
See the [optional chain contract](generator-optional-chain-yield-ownership.md).
The new three Engine fixtures and actual IR controls remain unexecuted.

The generator focused targets are `lila-engine --test aot_generators -- aot_generator_invocation_references::`
and `lila-ir --test generator_invocation_boundaries`, with existing await and
generator call controls affected. Fourteen generator fixture sources and twelve
boundary refusal inputs are authored. Independent review corrected the
prototype-setter admission mismatch and found no remaining source blocker;
Rust type checking, Wasm validation and execution remain pending.

Primary specification: [Call evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-evaluatecall), [ArgumentListEvaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-argument-lists-runtime-semantics-argumentlistevaluation), [super property evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-super-keyword), and [tagged-template evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-tagged-templates).
