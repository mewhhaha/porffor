# RegExp execution result mode

Status: normative for Lila's internal RegExp execution emitters.

## Boundary

`FunctionBuilder::emit_regexp_prototype_exec_from_locals` checks the RegExp
brand, converts the input to String, reads `lastIndex` and completes its
observable ToLength conversion before the sole compiled-program matcher reads
current flags or the program handle. A missing program uses the nonreturning
T19 semantic rejection; an invalid nonzero descriptor remains a genuine
corrupt-program Error. The old zero-program simple matcher and pattern fallback
are deleted. The matcher produces exactly two JavaScript result shapes:

| Mode | Producers | Observable result |
| --- | --- | --- |
| `MatchArrayOrNull` | `RegExp.prototype.exec` and the noncallable-`exec` fallback of ordinary `@@match` | a match Array on success and `null` on failure |
| `Boolean` | the intrinsic fallback in `RegExp.prototype.test` | `true` on success and `false` on failure |

The private `RegExpExecResultMode` derives no cloning, copying, debugging,
equality or default-construction capability. The wrapper owns the authority
and lends it once to the compiled-program matcher. Five direct exhaustive
matches select result materialization and the matcher's capture-carrier
lifetime. There is no Boolean projection, default or wildcard arm. A newly
added result mode cannot silently inherit an allocation, cleanup or
result-shape policy.

This is Rust-time emitter state only and adds no emitted ABI word. Retirement
removes the obsolete handled-local handshake and dead fallback locals. The
compiled matcher's five result projections, real error paths and `lastIndex`
coercion and update order are preserved.

The private cached-exec emitter implements the callable/fallback branches of
[RegExpExec](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpexec)
for the two `@@match` branches. The caller performs Get(exec) once; IsCallable
includes proxies. A callable result must be Object or null. A noncallable value
checks the receiver's real RegExp internal slots before using the intrinsic
Array/null producer, including a program replaced by that getter. Ordinary
custom-exec objects remain valid and RegExp proxies do not inherit their target's
internal slots. Input ToString, flags, global reset and per-iteration exec Gets
retain their order. Bad call results use the executing builtin's Realm TypeError.

String.match's RegExpCreate path always invokes the created receiver's observed
`@@match`, including the intrinsic or a callable proxy, after input and pattern
coercion. It has no intrinsic-identity exception or alternate pattern matcher.
The method's arbitrary result and abrupt completion flow directly to the caller,
as required by [String.prototype.match](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.match).

The callable custom-`exec` branch of `RegExp.prototype.test` remains outside
this boundary. That branch observes the user-supplied call result and converts
object/null to Boolean as required by the public RegExp protocol.

## Durable witnesses

`regexp_exec_result_mode_structure.rs` pins the exact private, attribute-free
two-variant domain, its absent capabilities, one owning and one borrowed typed
parameter, the single forwarding call, five unchanged lexical body fingerprints,
the recursive 16-mention ownership census and exact three-intrinsic-producer mapping.
It rejects reintroduction of the simple matcher and handled locals, and verifies
that the generic rejection import call is immediately followed by Unreachable.

The existing CLI ToLength structure guard keeps one ActiveHandler conversion
before compiled matching and forbids a second conversion in the matcher.
The runtime fixture checks throws from both static and computed patterns reach
their enclosing catches.

`wasm_regexp_exec_result_modes.js` distinguishes Array/null from Boolean results
and preserves captures, indices, inputs and global `lastIndex` updates/resets.
Its static capture pattern, computed single-character pattern and computed
empty pattern all use real compiled programs. Computed `a+` and `(a)+` String
patterns invoke the synthetic RegExp's intrinsic `@@match` and preserve match
metadata and captures. The added `\D{2}` controls exercise
UTF-16 and Unicode grouping, sticky failure, flags getters, custom exec,
lastIndex and input-coercion recompilation. Added controls cover noncallable
exec fallback, ordinary custom-exec receivers, Object/null validation, callable
proxies, class constructors, revoked callable proxies, getter throws and getter
recompilation. Synthetic RegExp invocation
also observes an overridden prototype method and accepts its arbitrary result.
Historical simple/fallback case
labels remain fixture labels; they no longer describe a product dispatch path.

## Focused verification

```sh
cargo test -p lila-aot-wasm --test structure_builtins -- regexp_exec_result_mode_structure::
cargo test -p lila-aot-wasm --test runtime_link -- runtime_regexp_entry_kind_structure::
cargo test -p lila-cli --test cli regexp::run_wasm_backend_preserves_regexp_exec_result_modes -- --exact --test-threads=1
cargo test -p lila-cli --test cli throw_propagation::regexp_exec_exceptional_to_length_routes_cover_the_compiled_matcher -- --exact
cargo test -p lila-cli --test cli throw_propagation::run_wasm_backend_routes_exceptional_to_length_throws_to_their_owners -- --exact --test-threads=1
./scripts/check-module-boundaries.sh
cargo fmt --all -- --check
git diff --check
```

The earlier source-equivalent capability closure passed its shared checks and
workspace semantic golden (`2/2`, 665 dumps). That historical receipt preceded
runtime capability rejection and fallback retirement. Retirement verification
is pending the combined corrective batch; no new broad pass is claimed here.

## Deferrals

This corrective batch does not close arbitrary runtime pattern compilation, RegExp
grammar or matcher opcodes, Realm allocation, match Array descriptors, broad
RegExp/Test262 execution or status publication. The global `@@match` source
catalogue is retired: its ordinary protocol follows object validation
unconditionally. The three orphaned child owners and their nine layout-only
tests are deleted. The former String.match raw-pattern fallback and its two
sole helper callees are also deleted. Separate String.matchAll, `@@search` and other String source
routes still require their own audit.
