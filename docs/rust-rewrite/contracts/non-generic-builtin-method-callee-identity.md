# Non-generic builtin methods retain the acquired callee

## Decision

A property call whose resolved target is a non-generic builtin must retain the
function value acquired by `GetValue(ref)` in the executable IR. It may not be
rewritten to a key-only `ExprIr::CallMethod` unless the receiver/key pair is
also proof that the backend's method fast path selects that exact function.

The first closed domain is:

- `%Boolean.prototype.toString%`;
- `%Boolean.prototype.valueOf%`.

For either target, lowering emits `ExprIr::CallIndirect` with the original
property read as `callee` and the property base as `this_arg`. If the property
base expression would otherwise be evaluated twice, the existing consuming
receiver-materialization path stores it once and uses that binding for both
operands.

## Why target knowledge is not receiver proof

Shape analysis can correctly know that `object.toString` currently contains
`%Boolean.prototype.toString%`. That fact determines the call's result type and
the builtin body's input analysis. It does not prove that `object` has Boolean
internal data.

This distinction is observable when a non-generic method is transferred:

```js
const value = new String();
value.toString = Boolean.prototype.toString;
value.toString(); // throws TypeError
```

`ExprIr::CallMethod { receiver, key }` deliberately leaves callee acquisition
to the backend. Its receiver/name fast paths are allowed to select intrinsic
methods. Replacing the already-acquired Boolean function with that node erases
the identity that makes the example throw.

`ExprIr::CallIndirect { callee, this_arg }` states the required invariant
directly: call this function object with this receiver. The Boolean builtin's
closed receiver-brand check remains the sole owner of the resulting TypeError.

## Boundaries

This contract does not classify every builtin as generic or non-generic, and
does not disable `CallMethod` fast paths for calls whose receiver proves the
selected intrinsic. Additional families must enter this boundary only with a
focused semantic witness and an explicit target classification; name equality
alone is not evidence.

The durable IR witness covers both Boolean methods, both their standard names
and unrelated destination names, valid boxed-Boolean calls, and
Object/String/Number/Date wrong-brand receivers. It also pins the one
materialized receiver binding shared by the callee read and `this_arg`. The
corresponding ten wrong-brand Test262 files are the bounded runtime gate.
