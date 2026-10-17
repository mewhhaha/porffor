# Async-generator return Await ownership

The pinned ECMAScript ReturnStatement algorithm distinguishes syntax with no
expression from an expression that evaluates to undefined. `return;` directly
produces ReturnCompletion(undefined). `return expression;` in an async generator
evaluates the expression and performs Await on its value, including undefined,
before producing the return completion. Falling off the body produces a normal
undefined completion. Neither a global binding name nor a value-kind estimate
can replace these rules.

IR retains the distinction: the expression form owns an AsyncAwait continuation
with Return resume mode; the bare form directly produces its undefined return
completion. The final runtime body-completion classifier completes an actual
undefined value directly, or completes an already-awaited return value directly.
It does not inject an additional Await for the bare form. A nonundefined,
unawaited completion retains the existing return-Await fallback.

This classification occurs after source evaluation. It cannot remove the IR
Await for an explicit undefined expression, `void`, or a shadowed `undefined`
binding. It does not change the separate AsyncGeneratorAwaitReturn or
AsyncGeneratorYieldReturn protocol used by external `.return(value)` requests.
Those requests await undefined and nonundefined values as required. Promise
resolution still performs observable proxy `Get(then)` and handles thrown
getters before the appropriate continuation and finalizer.

Pending completions retain their actual Await stage while finalizers suspend.
A finalizer's bare return may override a pending completion with undefined;
a finalizer that yields preserves an incomplete external return until resumed.
The native regression target preserves the exact pinned test body and harness
bytes in both language modes, and checks these protocol, runtime-value,
thenable, abrupt-completion, and finalizer distinctions. The new checks are
authored but unrun until the final composed batch is verified.
