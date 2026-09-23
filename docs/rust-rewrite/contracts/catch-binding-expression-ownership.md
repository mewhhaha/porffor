# Catch binding expression ownership

Catch parameter BindingInitialization runs after the catch parameter environment
is created and before the catch body block creates its environment. Expressions
inside array/object binding defaults and computed keys belong to the catch
parameter environment. Their classes and callables use the ordinary analysis
collectors and retain that environment after the catch body exits.

The catch scan traverses the complete binding pattern with the existing pattern
expression visitor. Parameter aliases extend the enclosing aliases; body lexical
aliases are added only after the parameter scan. This keeps a default closure's
outer lexical read distinct from an identically named declaration in the body.

When binding initialization emits statements, lowering places them before an
ordinary nested Block IR for the catch body. The wrapper has no lexical
environment. The original body retains its own lexical environment, completion,
and resumable statements, so the runtime creates parameter closures before
entering the body scope. Ordinary catch parameters without initialization retain
the existing direct body representation.

Current-main evidence includes four compiler panics in named class defaults
(object and array patterns, both modes) and three explicit unsupported callable
cases. These results come from the frozen 2abe45211 compiler, not from the
historical full-suite totals. The root-owned candidate run records outcomes for
ordinary reproductions before integration; no pending execution is called fixed.

Focused IR coverage checks callable/class owner collection, computed keys,
nested patterns, and distinct parameter/body capture owners. Native coverage
checks class identity, captured parameter lifetime, TDZ and abrupt defaults,
body lexical shadowing, nested generator defaults, and synchronous/asynchronous
catch bodies that suspend. The full catch cohort retains all pinned execution
identities, including prior passing cases, for regression verification.
