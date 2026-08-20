# Non-generic builtin methods retain the acquired callee

## Decision

A property call whose resolved target is a non-generic builtin must retain the
function value acquired by `GetValue(ref)` in the executable IR. It may not be
rewritten to a key-only `ExprIr::CallMethod` unless the receiver/key pair is
also proof that the backend's method fast path selects that exact function.

The closed domain is the Boolean, Number, BigInt, and String prototype methods
whose specification begins by extracting a branded primitive from `this`:

- `%Boolean.prototype.toString%`;
- `%Boolean.prototype.valueOf%`;
- `%Number.prototype.toExponential%`;
- `%Number.prototype.toFixed%`;
- `%Number.prototype.toLocaleString%`;
- `%Number.prototype.toPrecision%`;
- `%Number.prototype.toString%`;
- `%Number.prototype.valueOf%`;
- `%BigInt.prototype.toString%`;
- `%BigInt.prototype.toLocaleString%`;
- `%BigInt.prototype.valueOf%`;
- `%String.prototype.toString%`;
- `%String.prototype.valueOf%`.

For every target, lowering emits `ExprIr::CallIndirect` with the original
property read as `callee` and the property base as `this_arg`. If the property
base expression would otherwise be evaluated twice, the existing consuming
receiver-materialization path stores it once and uses that binding for both
operands. A private `NonGenericBuiltinMethod` enum owns this exact
thirteen-member domain; family/name strings cannot independently decide
whether to discard an acquired callee.

## Why target knowledge is not receiver proof

Shape analysis can correctly know that `object.toString` currently contains a
particular classified prototype method. That fact determines the call's result
type and the builtin body's input analysis. It does not prove that `object` has
the internal data required by that method.

This distinction is observable when a non-generic method is transferred:

```js
const value = new String();
value.toString = Number.prototype.toString;
value.toString(); // throws TypeError
```

`ExprIr::CallMethod { receiver, key }` deliberately leaves callee acquisition
to the backend. Its receiver/name fast paths are allowed to select intrinsic
methods. Replacing the already-acquired Number function with that node erases
the identity that makes the example throw.

`ExprIr::CallIndirect { callee, this_arg }` states the required invariant
directly: call this function object with this receiver. The selected builtin's
closed receiver-brand check remains the sole owner of the resulting TypeError.

## Boundaries

This contract does not classify every builtin as generic or non-generic, and
does not disable `CallMethod` fast paths for calls whose receiver proves the
selected intrinsic. In particular, generic String methods retain their
existing key-only fast paths. Additional families must enter this boundary
only with a focused semantic witness and an explicit target classification;
name equality alone is not evidence.

The durable IR witness covers three calls for each of the thirteen classified
methods: a valid same-brand boxed receiver under an unrelated destination name,
and an Object wrong-brand receiver under both the method's standard name and an
unrelated name. The six Number methods add a boxed-Boolean receiver under the
standard name because Boolean value folding is an independent pre-call path.
The lowerer has no object-identity token joining copied binding shapes, so a
property write first consumes the complete pre-write set of remembered Boolean
binding names: it clears every fold fact and erases the copied heap shape of
every candidate other than the precisely resolved write target before applying
that target's shape update. An alias therefore cannot leave either a Boolean
literal fold or an old builtin target on the original binding. BigInt's valid
boxed receiver is expressed as `Object(1n)` because `BigInt` is not
constructable. All 45 family cases pin the expected result kind on both the
outer materialization and inner indirect call, plus the one receiver binding
shared by the callee read and `this_arg`. A separate alias witness writes
`Number.prototype.toString` through a copied boxed-Boolean binding and proves
the original binding lowers to a dynamic acquired-callee `CallIndirect`, with
no stale function target.

The current runtime evidence is narrower than that spec-closed family. Ten
Boolean `toString`/`valueOf` files and ten Number `toString`/`valueOf` files are
the bounded wrong-brand Test262 gate. Both complete five-file Number prefixes
now pass 10/10 on the current checkout after the final alias-safe Boolean-fold
repair. The four Number formatting methods enter the same IR invariant because
their specifications perform the same branded `thisNumberValue` extraction,
but this contract does not claim a corresponding focused Test262 transfer
cohort for them.

Pinned Test262 sources separately witness BigInt's `thisBigIntValue` contract
in `built-ins/BigInt/prototype/toString/thisbigintvalue-not-valid-throws.js`,
the two `built-ins/BigInt/prototype/valueOf/this-value-invalid-*-throws.js`
files, and
`intl402/BigInt/prototype/toLocaleString/this-value-invalid.js`. The current-pin
BigInt leaf is 77/77, but those tests acquire the methods and invoke them with
`call`; they do not cover the key-only property-transfer lowering shape.

Pinned Test262 sources witness String's `thisStringValue` and current-function
realm contracts in the four `non-generic.js` and `non-generic-realm.js` files
under `built-ins/String/prototype/toString` and
`built-ins/String/prototype/valueOf`. They likewise do not provide a direct
property-transfer call for every destination-name/receiver combination. The IR
witness owns that structural coverage without overstating runtime evidence.

Symbol's branded `toString`, `valueOf`, `@@toPrimitive`, and `description`
getter, and Date's branded prototype methods, are outside this domain. Current
lowering has no key-only call rewrite for either family: their calls already
retain the acquired callee through the general indirect-call path. Adding them
would not make an otherwise plausible omission fail compilation.
