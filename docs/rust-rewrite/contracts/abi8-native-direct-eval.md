# Native direct eval ABI8 component

This unapplied source component adds a dedicated native caller-context compiler
boundary, separate from global Script and Function compilation. No Rust build,
Wasm validation, native execution or full-suite acceptance was run for it.

`compile_direct_eval` receives source String, immutable caller descriptor bytes,
Realm, nullable lexical/variable/private environments, this Value3, new.target
Value3, required invocation context and calling ProgramInstance (14 machine
operands). Completion5, status and nullable syntax message are separate results.
The host-only `lila.runtime_direct_eval_factory` prepends actual P2 (15 operands)
and returns Completion5/status. A JS throw has Entered status; actual parse/early
errors belong to the host SyntaxError result. All request roots survive callback
and all returned roots are bound before their request owners are released.

The existing intrinsic identity, non-string behavior and finite direct-eval paths
remain the gate to this native miss. The descriptor preserves actual caller
strictness, invocation grammar, private mapping, derived ownership and foreign
import-origin fact. It contains no foreign FunctionIds. Full caller equality also
includes actual hidden private-scope capacity; descriptor-visible bindings alone
cannot identify a cached artifact. The artifact retains full caller, exact UTF16
source and exact original R Arc after pinned Intl profile verification. The host
must validate these immediately before entry and retain the original execution
control Arc/deadline through parse, lowering, module compilation and same-Store
factory entry. Escaped references must remain OwnedRooted; there is no ambient P.

P2 enters only its witnessed exact source/kind/root with the real caller
lexical/variable/private records, snapshots and invocation context. The factory
cannot publish an ordinary or deferred-syntax row as a direct root. Its P2
operand is distinct from the calling P1 operand. Escaping derived super uses the
constructor's actual defining P and typed initializer hook, including private
brands/methods/accessors; see `abi8-instance-elements.md`.

External-caller `import()` retains a typed foreign origin. Until a real P1 Script
Record bridge exists, only actual execution reports capability code11 after
operand effects and abrupt propagation. An unexecuted branch or nested function
containing import must still compile; foreign origin never becomes global None.

ABI8 changes FunctionContext and retains separate authenticated ABI4, ABI6 and
ABI7 envelope rejection controls. Those controls are not evidence of old-schema
GC link rejection; actual typed-Wasm closure and native same-Store link tests are
still required. Six new Rust controls are authored and unrun (four direct-eval
AOT controls and two ABI7-envelope controls); the existing IR wire-domain control
also covers code11. Publication README/status updates are a separate root-owned
join; this source packet claims no installed capability or passing native ID.
