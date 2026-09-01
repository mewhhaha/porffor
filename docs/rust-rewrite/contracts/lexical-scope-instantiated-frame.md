# Lexical-scope instantiated frame

`LexicalScopeInstantiation` owns whether its declaration-instantiation sweep
created the Environment Record that must be removed when the statement list
ends. That lifecycle policy is a private two-row `InstantiatedFrame` domain:

- `Pushed` means the constructor pushed the frame and `finish` must pop it;
- `Current` means the caller already owns the frame and `finish` must leave it.

The domain has no derived or manual capabilities. The token is consumed by
`finish`, whose exhaustive match is the only semantic observer. Adding a row
therefore requires a deliberate lifecycle decision instead of inheriting a
Boolean or default policy.

`instantiate` and `instantiate_switch` push before creating any binding and
produce `Pushed`. `instantiate_in_current_scope` performs no push and produces
`Current`. All three constructors complete their declaration sweep before
returning the token.

The bounded structure guard recursively fixes the nine source mentions, exact
three producer mappings, push-before-sweep and complete-switch-sweep ordering,
and the `Pushed` pop / `Current` no-op consumer arms. This closure changes no
lowered IR or JavaScript behavior; it removes capabilities that were unused by
the ownership protocol.

Focused verification passes the structure target `3/3`, the exact block-TDZ
witness `1/1`, and the exact switch shared-environment TDZ witness `1/1`.
Independent review confirmed the capability and nine-mention closure, all three
producer bodies, push/sweep/pop ordering, exhaustive lifecycle arms and
source-equivalent behavior. Coordinated `cargo xc`, full formatter, diff,
module-boundary and task-plan checks are green. Broad conformance verification
remains deferred.
