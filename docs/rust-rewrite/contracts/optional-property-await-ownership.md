# Optional property await ownership

Status: 2026-10-03 source implementation. Compilation, Wasm validation and
execution remain pending. T14 and published conformance results remain open.

Plain async value expressions outside loop bodies and inside checked awaited
while conditions with eager bodies admit property-only optional chains
with awaited computed keys, including nested shorted links and later ordinary
links. The checked source constructor consumes actual AST fields and shorted
flags, rejecting private links and direct private/super targets before the
property continuation owner is entered. Call-bearing tails additionally require
the [outside-loop Call source owner](optional-call-await-ownership.md).
Each shorted link guards its entire
remaining suffix; returning undefined from one access and continuing the suffix
outside that guard would be incorrect.

The actual target's GetValue is retained once in an activation binding. A
selected property read consumes this retained base and its checked source link;
its completed GetValue is materialized before a later key suspension. A shorted
link uses the same strict saved-value nullish predicate as `??`, the existing
conditional result cell and `AsyncFunctionIfPlanIr`. Only its selected arm lowers
the complete suffix. Later shorted links reuse that owner recursively. A suffix
with no suspension and no Calls uses the existing `OptionalPropertyChain`
emitter, preserving
its per-operation shorting, ordinary nullish checks and primitive/Proxy receiver
semantics. No continuation IR, dispatcher, object model or host helper was added.

Raw awaited key values retain their actual kind information. The existing
property emitter still owns ToPropertyKey and arbitrary hook failures. For an
ordinary computed suffix on a nullish receiver, it evaluates the raw key value,
then rejects RequireObjectCoercible before key coercion. The removed all-tags
identifier-to-String narrowing was unreachable behind earlier String/Symbol/
coercible returns; its deletion is cleanup, not a demonstrated semantic repair.

Every branch arm uses the existing private scope and prefix/state census before
joining the same outer binding-fact domain. Activation slots remain registered.
Nested statically nullish optional expressions can remove all syntactic awaits
from an admitted branch. In that case the existing plan constructor returns
None, proving both arms remain at their computed entry points. The result owner
emits the existing ordinary If and restores the enclosing entry state; it still
owns both result writes and the resumed read. Real suspensions retain the
existing disjoint state plan.

Synchronous optional siblings bypass the Await walk as before. A target-only
Await followed by a synchronous chain uses the existing linear target prefix.
An awaited property value may be a Call argument or constructor value. An outer
ordinary property access also keeps its own Reference owner, including an outer
method call or delete. Grouped callees and tags whose last link is a property
now consume the [terminal Reference owner](grouped-optional-reference-await-ownership.md)
outside all loops, including earlier same-chain Calls. They retain the final Get
and raw receiver together before
outer operands. Direct delete and target-only awaited callee/tag References
remain refused. Terminal Call results from awaited inner chains used as outer callees/tags,
private/super routes, Call-bearing tails in loops, awaited loop bodies,
other loop heads, async generators and mixed suspension protocols remain
explicit boundaries. Bounded ordinary-generator values with one yielded first
shorted key use the separate
[generator branch owner](plain-generator-value-branch-ownership.md). The [logical assignment owner](logical-assignment-await-reference-ownership.md)
composes for its checked declarative/ordinary-property References outside loops.
Its runtime/with/global/unresolved and suspended-LHS boundaries remain. Existing
statically nullish chain exceptions remain unchanged. The [checked awaited while owner](plain-async-awaited-while-condition.md)
keeps retained bases/branch results activation-owned across condition reentry and
checks exact value-prefix state association; erased awaits leave a restartable
eager prefix rather than an effect hoisted outside the loop.

Five IR controls cover retained base/result association, raw key facts, state
ranges, Get-before-next-key order, nested/target-only/synchronous composition,
zero-suspension joins and true source boundaries. Three paired strict/sloppy
Wasm-AOT sources cover nullish/HTMLDDA selection, Symbol/key hooks, Proxy/getter
receivers, thenable mutation, full suffix skipping, ordinary nullish key order,
outer References, argument/constructor values, arbitrary cross-Realm abrupt
identity and awaited finally. No authored source has been executed.

The semantic ordering follows [optional chain evaluation](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-optional-chains)
and [computed property access](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-evaluate-property-access-with-expression-key).
This scope supersedes the earlier contracts' blanket dynamic-optional refusal;
the remaining Reference and protocol boundaries above are still open.
