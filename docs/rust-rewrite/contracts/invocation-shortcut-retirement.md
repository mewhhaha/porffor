# Retire duplicate Array and String invocation shortcuts

Status: the complete bounded successor source passed the ref105 grouped
all-target Rust type check with generator, indexed-collection and portable
performance batches. Emitted-Wasm/runtime acceptance remains pending.
Historical backend proofs remain scoped to their recorded source. Full T16,
T18 and T26 acceptance and fresh pinned publication remain open.

## One acquired-callee route

The retirement removes all five direct Array join/toString/reverse returns
across two duplicate lowerer blocks, and the inferred String substring/slice
branch that reconstructed a canonical method key. For these source calls, the
existing acquired-callee path now owns receiver capture, the complete argument
list, result facts and invocation effects. No replacement classifier, opcode,
backend body or lifecycle wrapper is added.

The early Array-shaped block ignored own method properties. It could claim a
String result for an own join returning a Number, route an own toString through
the canonical builtin, or retain reverse's pre-call element facts. The second
Array-kind block independently bypassed the shared owner for fresh/default-
prototype join and reverse. Both are removed; a transferred String alias likewise
keeps its original property rather than looking up substring or slice again.

The actual PropertyRead/GetV and raw receiver pass to the existing indirect
method publication. Its materialized receiver is evaluated once and feeds both
the callee Reference and this argument. Function acquisition precedes all
argument and real SpreadArgument evaluation. Rebinding the source receiver,
replacing its method during arguments, ignored trailing operands and abrupt
spread completion preserve ordinary evaluation order and the captured callee.
Callable Proxies use the existing general Call owner.

[EvaluateCall](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-evaluatecall)
provides the Reference/GetValue/argument/Call ordering. The installed Array
[shared toString](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.tostring)
algorithm may return an arbitrary callable join result. Source syntax does not
license a canonical-name substitution or a narrower result fact.

## Final Wasm invocation consumer

The final indirect-call emitter also retires its exact-target interception for
String Match/Split/Slice. That block called direct method adapters before
compiling the acquired callee. The ordinary general emitter now evaluates the
actual callee, raw this value and complete argument vector before the existing
Function-or-Proxy call owner. It retains completion propagation and cleanup.
This closes Slice's final consumer as part of this batch; the coupled Match and
Split interception shares the same retired block.

Direct eval, class field initialization and the separate Reflect.defineProperty
consumer retain their existing branches. Native method bodies and independently
live direct adapters remain. The complete [String invocation-family successor](string-invocation-family.md)
retires primitive CharCodeAt/Split and thirty-three inferred canonical-name
branches, closes generic String effect flags and corrects MatchAll/spread result
facts. That successor needs its own grouped type/runtime checkpoint; no full
String protocol closure follows from deleting this emitted CallIndirect block.

## Existing facts and coercion owners

Actual Array targets consume the completed
[indexed-collection invocation owner](indexed-collection-invocation-reference.md).
Its result-bearing admission and one indirect publication retain truthful
arbitrary join values, ToObject results, no stale receiver shape, and conservative
caller invalidation. An overridden or unknown method instead uses existing
candidate analysis and its own return/effect facts. Native toString, join and
reverse algorithms retain their prior backend implementations.

String substring/slice fall through ordinary lower_call_args with an explicit
method receiver and then the final indirect publication. Their builtin signatures
have no formal inference parameters that could eagerly coerce source arguments.
All arguments, extra ignored operands and ARGUMENT_LIST spread remain actual IR
operands. Their return remains String with no speculative shape. The existing
literal-fold owner does not admit either target, so it cannot discard the raw
receiver or argument effects.

The two actual String catalog rows carry SYNCHRONOUS_USER_CODE: receiver ToString
and numeric index coercion can invoke user code. Both ordinary call-info and
spread-aware effect paths consume these flags, and the analyzed effect owner
attaches to the same emitted invocation. The canonical range bodies still own
receiver coercion, start/end conversion, clamping/relative indexes and UTF-16
materialization, including lone surrogates and called-function Realm errors.
No separate range algorithm or general catalog campaign is introduced.

## Controls and verification

Actual lower_script controls retain the preceding collection cases and inspect
original source keys, actual target identity, materialized receiver, full
spread/extra operands, truthful own/shared result facts, reverse flow invalidation
and String coercion effects. Two finite Engine cohorts use fresh Array source
gates and separate literal String aliases, covering overrides and callable Proxies,
acquisition before argument replacement, full spread order, arbitrary values,
getter/coercion effects, original throws, Realm errors and UTF-16 extraction.
They run in paired strict/sloppy WasmAot with Test262 host, one compilation
worker, 30,000 ms timeout, exact Normal Number262 and one exact cohort print.
Authored expectations require execution and provide no conformance result.

Complete code, types, controls and documentation preceded the passing ref105
grouped all-target Rust check. Root owns focused actual IR/Engine and unchanged
backend-owner guards,
then the required broader/pinned checks. No task closure or published status
change follows from dry source.

## Remaining boundaries

The String invocation-family successor removes the specified substr/trim/other
method branches and closes generic String effect metadata. The complete
[Number hook and dispatch successor](number-string-hook-and-dispatch-retirement.md)
also retires primitive Number borrowed-hook CallMethod producers and their copied
tracking, the unproduced StringCharCodeAt form and HTML-name-only machinery.
Both successors need their own type/runtime acceptance after the full-task dry
source pass. The complete [remaining invocation successor](remaining-invocation-reference-ownership.md)
retires both literal folds, canonical factory/iterator calls and the species
forwarding bypass. Optional/suspended source owners and the wider catalog remain
separate.
Backend direct adapters remain live for their independent consumers. This
retirement closes these specified lowerer shortcuts; it does not certify all
method dispatch, the full Array/String API or the whole builtin flag catalog.
