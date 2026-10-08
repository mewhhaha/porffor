# Grouped optional Reference await ownership

Status: 2026-10-04 dry source implementation. Compilation, Wasm validation and
execution remain pending. T14 and published conformance results remain open.

Plain async functions outside all loops admit an outer ordinary Call or tagged
template whose grouped callee/tag is an optional Property/Call chain ending
in a property, with an awaited target, key or Call argument, such as
`((await target)?.method)(argument)`, `(target?.[await key])(argument)` or
``(target?.[await key])`head${substitution}tail` ``. Repeated parentheses preserve
this route. The consumed checked source factory requires an actual grouped
optional source, checked Property/Call links, a genuine terminal property,
a plain async activation and loop depth zero. Grouped admission requires Await
in the actual source and consumes the same complete `from_links` owner for
either terminal kind. Ordinary value admission retains its link-await gate.
The existing mandatory prefix walk
checks these Reference operands before
allocating states, alongside its existing logical-assignment Reference check.
Nested activation bodies retain their separate admission scope.

The same optional value owner retains the original base GetValue once and
materializes each intermediate Get before a later key suspension. Its private
terminal mode consumes the existing `CapturedCallReceiverIr` and emits the
existing `CaptureOptionalCallReference` around the final actual property chain.
The terminal Get supplies both the callee value and its original raw receiver;
there is no separate terminal Get followed by replay for receiver capture. An
eager suffix keeps the existing optional-chain emitter's per-link shorting and
primitive/Proxy/key-hook behavior. Raw computed keys keep their original facts;
the emitter still owns ToPropertyKey.

A target-only Await completes before the synchronous Property tail starts. The
same retained base and full-tail Reference consumer handle that route without
new conditional resume states. Nullish targets skip all synchronous key/Get
work while grouped ordinary outer operands remain unconditional. Earlier
synchronous Calls before the terminal Property use their existing Reference
and Value owners. A nested optional first Call over the grouped Property passes
the same complete source proof, preserving its receiver before selected awaited
arguments.

Each shorted link guards its entire remaining suffix. Its skipped arm publishes
undefined and leaves the captured receiver's initialized undefined value. The
selected arm uses the existing private scope, conditional result cell and
checked state/fact join. Only Normal completion publishes the callee. Erased
syntactic awaits use the existing ordinary If fallback without claiming
resumption states. Ordinary optional value mode retains its final Get activation
binding and established result ownership unchanged.

The shared invocation owner pins the joined callee before evaluating the outer
arguments. Grouping ends optional shorting: nullish callees still evaluate all
outer arguments or substitutions before the existing noncallable error. A tag
uses the existing retained GetTemplateObject before substitutions, preserving
the frozen, cached template object. Argument/substitution mutations cannot
replace the saved function or receiver. Arbitrary key/Get/call throws,
rejections, awaited finally and execution-Realm restoration remain with their
existing consumed owners.

The closed Call/Construct purpose domain admits this terminal Reference mode
only for awaited Calls and tags. Constructors keep the existing value route.
Earlier same-chain Calls consume the [optional Call owner](optional-call-await-ownership.md)
and retain their resulting Values before the terminal property. A first Call on
a grouped awaited target passes the corresponding closed terminal-kind proof
before states are allocated. Terminal Call results now consume the separate
[grouped Call Value mode](grouped-optional-call-value-await-ownership.md), including
target-only suspension and a preserved inner first-Call receiver. This contract's
terminal Property Reference path retains its actual Get/receiver ownership.
Direct delete, direct private/super targets and private links, all loops,
generators and mixed suspension protocols remain explicit refusals
before state consumption. No continuation IR, dispatcher, ABI or object model
was added.

Meaningful IR controls inspect terminal capture/result/receiver association,
state ranges, eager suffix retention, template ordering, erased-await ownership,
target-only base retention and nested first-Call source ownership. Earlier
grouped-Reference refusal maintenance is retained; the two now-owned target-only
Property rows move into the existing positive grouped cohort. Three paired
strict/sloppy Wasm-AOT sources cover actual key/Proxy/getter hooks, raw primitive
receivers, intermediate-child retention, suffix skipping, outer spread/jobs,
nullish argument/substitution order, template cache/descriptors and foreign
abrupt identity. New target-only controls remain uncompiled and unexecuted.
Earlier verification checkpoints do not establish executable acceptance of
this batch.

The ordering follows [optional chain evaluation](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-optional-chains),
[EvaluateCall](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-evaluatecall)
and [template argument evaluation](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-argument-lists-runtime-semantics-argumentlistevaluation).
This bounded route supersedes the optional value contract's blanket grouped
callee/tag refusal; its other Reference and protocol boundaries remain open.
