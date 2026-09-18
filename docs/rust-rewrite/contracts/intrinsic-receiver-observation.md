# Intrinsic receiver observations

Intrinsic signatures describe catalog functions and host functions, which have no
source activation that can own a lexical `this` capture. Before a call observes a
receiver, their `this_info` is an empty seed (`undefined` with
`this_observed == false`). The seed is metadata; it does not supply the runtime
receiver or declare that an intrinsic has been called with `undefined`.

The first receiver observation replaces that seed. Later observations merge with
it through `merge_function_this_info`. A standard or host intrinsic receives the
explicit receiver directly, without the sloppy-function fallback. Return value
and constructed-instance metadata remain separate from receiver metadata.

Source function signatures retain their existing initial receiver information:
source arrows can consult their enclosing activation before a receiver has been
observed. Function bodies and lexical captures are owned by source `FunctionPlan`
entries, not by catalog signatures. Dynamic-source intrinsic signatures likewise
have no source body; a compiled function created by such an intrinsic has its own
source plan and receiver observations.

Do not seed every intrinsic signature with a copy of the script global object.
Call-context propagation clones signature maps, so those copies multiply the
complete nested global shape at every propagation step. Catalog signature
factories are static functions to keep their initial metadata independent of a
particular lowering activation.

`aot_intrinsic_receiver_observation` exercises repeated intrinsic calls, builtin
getters, source receiver binding and lexical capture, construction, and the empty
dynamic function protocols. The canonical native-function harness remains the
integration regression; its source and assertions must not be substituted to
reduce lowering work.
