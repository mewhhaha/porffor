# Inferred indexed-collection invocation References

Status: the complete dry source is authored for the shared 55-target
Array/TypedArray inferred invocation owner. It passed the ref105 combined
all-target Rust type check. Emitted-Wasm/runtime controls and broad/pinned
acceptance remain pending. It does
not close T16, T17 or T26, change a published count, or establish full conformance.

## Original Reference and one invocation

A method transferred to `savedJoin` must call the acquired function with the
original receiver, even if that receiver has a different `join` property:

```js
const target = {
  length: 2, 0: 'a', 1: 'b',
  savedJoin: Array.prototype.join,
  join() { return 'wrong'; }
};
target.savedJoin(':'); // required 'a:b'
```

The earlier selected-owner match published a canonical method name instead of
the original callee. Its common CallMethod consequently substituted another
property Reference. The new private
`inferred_indexed_collection_result_info(StandardBuiltinId) -> Option<ValueInfo>`
combines admission and the actual result fact. Each selected Some is complete;
there is no second target list, canonical-name output or admitted-target result
hole. Other builtins retain the ordinary invocation route.

The source owner lowers the full argument list once and uses the existing
`lower_indirect_method_call` publication. Its materialized receiver feeds both
the original PropertyRead and CallIndirect.this_arg. The emitted call acquires
the original function before left-to-right argument evaluation and real spread
iteration, then calls it with the captured receiver and all arguments. Callee
replacement, receiver rebinding, a poisoned canonical name and trailing ignored
operands cannot change these observations. General callable and Proxy dispatch
remain their existing owners.

[EvaluateCall](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-evaluatecall)
requires this Reference, GetValue, receiver, argument-list and Call ordering.
[Array join](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.prototype.join)
also requires ordinary generic receiver semantics.

## Result and effect facts

The consumed result match publishes only algorithm-guaranteed facts:

| Result family | Permitted fact |
| --- | --- |
| Number, Boolean, Undefined, join/locale String | The algorithm's primitive result kind. |
| Element lookup and reduction | Dynamic, all runtime tags, no speculative shape/targets. |
| Seven Array species methods | Existing unshaped Object-like domain, including arbitrary Object, Function and Arguments species results. |
| Fill, Sort, Reverse and CopyWithin | ToObject result: Object-like tags and no retained pre-mutation receiver shape or unboxed primitive tag. |
| Shared Array/TypedArray toString | Arbitrary callable join result, all runtime tags and unknown callable targets; no String coercion assumption. |
| Six Array/TypedArray iterator factories | Object-only result, no speculative method shape and no callable targets. |
| Strict TypedArray Map/Filter and guaranteed Array copies | Their actual constrained result domain, without invented element facts. |

The same coupled result facts are corrected in live builtin call-info and
signature producers. The existing String iterator result also has no fabricated
Array iterator shape: real iterator instances inherit mutable methods from
their own prototypes and publish internal slots, not fixed own next/@@iterator
properties. The unused fabricated iterator shape owner and sole false
Concat/Map/layout result helpers are retired. Existing iterator prototype and
species constructor-analysis owners stay live.

Caller facts are conservatively invalidated for unknown synchronous user code
before the one invocation publication. Generic indexed getters, coercions,
callbacks, species and arbitrary join cannot borrow sparse catalog flags as
proof that hooks are absent. The shared toString catalog row gains its actual
synchronous-user-code flag for other invocation routes too. Existing receiver
mutation, Push/Shift/Unshift updates, species constructor this, callback
parameter/this/exact-context and comparator analyses remain consumed. ToSorted
joins the existing Sort comparator observation; the six TypedArray predicate
and find methods join the existing callback observation. A previous omitted-
argument callback call cannot keep those later parameters narrowed to Undefined.

## Meaningful controls and acceptance

The production lane contains separately lowered literal intrinsic-alias cases
for all 55 selected targets. They inspect original source property keys,
callee identity, once-only materialized receiver, CallIndirect receiver and
full real arguments. Separate controls cover result domains, mutable iterator
prototypes, arbitrary species, effect invalidation and mixed-use callback facts.
The finite Engine target is `aot_indexed_collection_invocation`, with three
cohorts for References/arguments, results/effects and Realms/abrupt completion.
Each uses paired strict/sloppy WasmAot, the Test262 host surface, one compilation
worker, a 30,000 ms run bound, exact Normal Number 262 and one exact cohort print.
These authored expectations require execution; they are not passing evidence.

After all production, controls, types and documentation are complete, Root owns
one grouped all-target type checkpoint, the focused actual IR and Engine
regressions, unchanged canonical algorithm guards and required broader checks.
Fresh pinned full-tree acceptance remains necessary for T16/T17 and publication.

## Explicit neighboring work

The related [shortcut retirement](invocation-shortcut-retirement.md) removes
all five early direct Array join/toString/reverse returns and the inferred String
slice/substring canonical-name branch. Their successor source retains the
acquired callee through the completed shared or ordinary indirect owner, including
the retired exact Match/Split/Slice interception at the final Wasm consumer.
The ref105 grouped all-target Rust type check passed; runtime acceptance remains
pending. This batch does not certify other
direct invocation routes, the whole
builtin flag catalog, arbitrary callback inference or all Array/TypedArray
algorithms. No interpreter, runtime representation, backend opcode, skip or
source rewrite is introduced.
