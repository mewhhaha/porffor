# Owned Wasm local declarations

`code_sink::LocalDeclarations` is the exact typed declaration plan consumed by
`Function::new`. Its fields are private and it implements neither `Clone` nor
`Copy`. Every product body constructor now supplies that plan, including the
three FunctionBuilder body producers and the six allocation/property helpers.

The function owns its plan alongside its encoded body and control frames.
`retain_local_prefix` consumes the function, derives the retained declaration
from its own original typed runs, and keeps all instruction bytes and label
identities. Callers cannot supply a separate planned count, substitute an all
`i64` declaration, or reuse a borrowed body during trimming. Cloning a complete
function internally forks its body, declaration and frame state together; it
does not expose a cloneable standalone declaration plan.

The existing local rewrite regression now checks exact encoder bytes for a
retained prefix containing `i32`, `anyref` and `eqref`, as well as live frame
identity. Existing finished-body and exception-handler regressions follow the
new consuming operation. The declaration-prefix bound remains a checked runtime
boundary; this change does not prove that every emitted local instruction uses
the right type or that trimming removes only unused slots.

This is a dry source foundation. Current product declarations and semantic
value ABIs remain unchanged; JavaScript identities still use the uncollected
linear heap. No semantic GC migration, weak reachability or URI allocation fix
is established. Compilation, focused regressions, emitted-byte golden comparison
and broad verification are pending the complete invariant batch.
