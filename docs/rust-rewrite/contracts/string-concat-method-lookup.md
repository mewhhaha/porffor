# Concat method lookup

`receiver.concat(arguments)` acquires the actual property before evaluating
arguments. The property name does not identify String or Array concat: boxed
Strings, Arrays and arbitrary objects can shadow it, and either prototype can
be replaced with a method or accessor.

The concat emitter evaluates the receiver once, obtains its lookup object with
`ToObject`, and performs `Get` with the original receiver. Primitive getters
and strict callees therefore observe the primitive `this`. Abrupt receiver,
boxing or getter completion stops argument evaluation. After a successful Get,
arguments run once in order before the shared call emitter checks callability
and propagates the callee completion. A non-callable property raises TypeError
after argument evaluation.

Planning retains all three possible concat owners: Array prototype, String
prototype and static Iterator concat. The String builtin continues to reject
nullish borrowed receivers and convert its receiver followed by arguments with
`ToString`; the dispatch repair adds no special conversion for `undefined`.

`wasm_concat_method_lookup.js` covers boxed undefined, dynamic String receivers,
Array behavior, own and prototype overrides, accessor `this` and method
acquisition, getter/argument throws, non-callable and nullish order, and ordered
builtin conversions. The engine target also embeds the unchanged pinned
`S15.5.4.6_A1_T9.js` and prepares both sloppy and strict script modes.

This source and its alias followup are integrated in the 2026-10-03
implementation-first batch. Compilation, focused execution and current pinned
conformance verification remain pending; integration supplies no runtime pass.

An inferred builtin target may select the body that planning must retain, but it
must preserve the property's original reference. For example, after
`receiver.alias = Array.prototype.concat`, `receiver.alias(value)` acquires
`alias`; an unrelated `receiver.concat` override or throwing getter cannot
replace that callee. Inherited aliases keep the original receiver, and the
acquired method survives argument evaluation that replaces the alias or the
receiver's prototype. String receiver coercion still follows argument
evaluation.

The original fixture's 37 check call sites remain unchanged. Its earlier
39-entry prepared-label inventory included two descriptor property-key strings
named `concat`, which were not checks. The followup adds 15 check call sites
for transferred Array/String aliases, unrelated canonical properties, inherited
receiver identity, alias/prototype mutation and alias-getter order, for 52
prepared checks. These are source controls awaiting actual Wasm-AOT execution;
the retained independent alias-review failure does not become a passing result.
