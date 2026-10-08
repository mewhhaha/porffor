# Native iterator helper operation dispatch

The atomic GC draft uses the private IteratorHelperOperation domain with
Next and Return variants. The actual shared dispatcher and each concrete
family resume entry consume it through exhaustive matches. HelperKind owns
seven concrete brands and the closed mapping to their real internal entries;
zip and zipKeyed share the Zip brand. All thirty actual builtin arms consume
the authored native helper owners.

Shared next/return validates the concrete brand and running state before
changing lifecycle fields. Already-completed calls create a fresh done result
in the called Realm. Active resumes materialize the real internal callable
with the captured creation Realm, then use the sole whole-value invocation
owner, including Realm restoration on normal and abrupt completion.
Suspended-start return uses the called Realm and completes before Close;
resumed return retains the running state during Close.

The closed types make an omitted operation or unhandled new variant a type
error; they do not prove the semantic association of a named variant with its
builtin. Finite paired controls cover reentrancy, closing, whole values,
borrowed Realms and the concrete family semantics. The previous raw dispatcher,
its obsolete operation enum and its spelling guard retire together.

See the [native helper contract](gc-iterator-helper-execution.md). The earlier
manual-layout verification remains historical evidence, not acceptance of this
source successor. Compilation, fixture parsing, Wasm validation and execution
remain pending until the complete source pass; later checks require a confirmed
aggregate 4096 MiB cgroup budget and one worker.
