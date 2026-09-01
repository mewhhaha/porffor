# Promise combinator algorithmic error Realm

Status: implemented on 2026-08-26; focused verification is recorded below.

## Ownership

`Promise.all`, `Promise.allSettled`, `Promise.allKeyed`,
`Promise.allSettledKeyed`, `Promise.any` and `Promise.race` create their own
algorithmic TypeErrors and RangeErrors in the executing static method's Realm.
Constructor `C` independently owns the returned Promise and its resolving
functions. It is not error-Realm authority.

This distinction is observable when a created-Realm method is borrowed with
the entry `%Promise%` as `C`: the returned Promise has the entry
`%Promise.prototype%`, while an invalid iterable or keyed input rejects it with
a TypeError from the created method's Realm.

## Typed boundary

`PromiseCombinatorAlgorithmErrorRealmContext` is private, non-`Copy` and
must-use. It pairs the TypeError and RangeError prototype locals derived from
one executing method Realm.

The private
`builtins/promise/promise_combinator_algorithm_error_realm.rs` child owns the
paired context, its sole factory, both typed error emitters and the consuming
release. The Promise parent neither imports nor re-exports the context. Its
three retained combinator bodies can only borrow the inferred value through
child-owned methods, so adjacent Promise code cannot construct a mixed Realm
pair or project either raw prototype local.

A zero standard-builtin environment explicitly selects the entry prototype
globals. A nonzero environment must be the self-backed method function. The
factory loads its defining Realm, that Realm's intrinsic record and both error
prototype slots; missing state traps without a dynamic-current-Realm or
constructor fallback.

The factory reserves both durable prototype locals before its transient Realm
and intrinsic-record locals. The transient locals are released inside the
factory. One consuming release returns RangeError then TypeError in reverse
reservation order.

## Closed census and order

The three shared lowering functions each acquire one context only after the
fallible `Get(C, "resolve")` has completed:

1. `emit_promise_race` borrows it for six TypeErrors;
2. `emit_promise_keyed` borrows it for two TypeErrors; and
3. `emit_promise_combinator` borrows it for six TypeErrors and the structural
   maximum-length RangeError.

The complete source census is therefore three acquisitions, fifteen typed
borrows and three consuming releases. The live combinator bodies contain no
raw `emit_throw_runtime_error` call. The separate duplicate callable-resolver
check in `emit_promise_static_settle` and errors created by callbacks or other
Promise algorithms are outside this boundary.

## Focused evidence

The bounded structure target pins context privacy and pairing, the explicit
entry route, strict nonentry Realm/intrinsic traversal, reverse local lifecycle,
the exact `3/15/3` census, acquisition after `C.resolve`, and the absence of raw
error construction in all three combinator bodies.

The source-equivalent extraction selected the exact six-line context at
SHA-256
`b66cf09315fe22f69c6dd74d3ed3deb752f9ced7c0d08450ad94e948f55e1fbc`
and 110-line four-method lifecycle at SHA-256
`799c7f889d06dd6a69371508557e259725881d0ee1127f3cb5f7377d20d31aa3`.
Their combined 116 selected lines retain SHA-256
`49037556ac09bceed8e1f7138b409b66f2503d9bc55d8101fe1014f81731df8e`.
The 119-line child has SHA-256
`c7cf098541a7a0ef5c15ad3e105fcfe8b8ebb5c4a3c96fedfb3a5fdbdca24223`
and reduces `promise.rs` from 9,038 to 8,923 lines. The recursive ownership
target passes `5/5` as a standalone include-only Rust structure target.

The finite CLI fixture borrows all six created-Realm methods with the entry
Promise constructor and invalid iterable or keyed inputs. It observes entry
returned Promises and created-Realm TypeErrors. The `2^53 - 1` iteration
RangeError remains structural because reaching it is not a bounded runtime
witness. The structure target passes `5/5` and the exact CLI target passes
`1/1` on 2026-08-26.

```sh
cargo test -p lila-aot-wasm --test promise_combinator_algorithm_error_realm_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_created_realm_promise_combinator_algorithm_errors --quiet
```

The coordinated semantic golden passes `2/2` in 717.58 seconds with 674 dumps.
It adds this witness plus the independent Temporal overflow-options and GroupBy
result-kind witnesses, removes none and leaves all 671 retained dumps equal
after accounting normalization. Broad Test262 verification remains deferred.

For this extraction checkpoint, the focused Cargo structure target again
passes `5/5`, the neighboring combinator-mode target passes `3/3`, and the
existing created-Realm CLI witness again passes `1/1`. The shared `cargo xc`,
workspace formatting, diff, module-boundary and task-plan checks are green. No
new semantic golden was captured; the coordinated golden recorded above
remains the behavior-equivalence evidence. The four moved method bodies and
three combinator callers are otherwise unchanged, so no new behavior or
conformance claim is made.

This contract does not complete the Promise combinator algorithms, callback
materialization, `Array.fromAsync`, T06, T14 or the full Test262 acceptance
matrices.
