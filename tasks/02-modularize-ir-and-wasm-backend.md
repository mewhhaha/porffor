# T02 — Modularize the IR and Wasm backend

**Status:** In progress — major builtin ownership bottlenecks plus the for-of, for-in, throw-value inference and static-JSON parse owner splits; broader lowering/emitter seams remain

**Parallel group:** Bootstrap/foundation  
**Depends on:** None  
**Blocks:** Safe parallel work in T04-T24

## Current repository state

Both crates now expose dedicated IR, lowering, analysis, diagnostics,
operations, ABI, heap, object, function, environment, control-flow and builtin
modules, and `./scripts/check-module-boundaries.sh` enforces the highest-value
seams and line budgets. The split remains partial: `lowering.rs`,
`builtins/standard.rs`, several family files and object/operation emitters are
still large implementation stores. Treat the landed boundaries as independent
ownership surfaces, but continue coordinating broad edits to those remaining
hotspots.

Two `lila-ir` parents now remain below their existing raw-line caps through
domain ownership rather than test relocation. The closed callable source-text
representation, exhaustive materializer and focused test have one 38-line
`builtins/callable_to_string.rs` owner with the canonical public re-export;
`builtins.rs` is 1,748 lines against its 1,760-line cap. The nonduplicable
invocation-effect proof, builtin result carrier, closed analyzed-effects state
and opaque source/host caller-flow aggregate have one 192-line private
`lowering/invocation_effects.rs` lifecycle owner with direct sibling imports
and no compatibility re-export; `builtin_call_info.rs` is 2,243 lines against
its 2,250-line cap. The analyzed state makes already-applied effects and
effects that must accompany emitted call IR distinct variants instead of an
ambiguous optional proof. The host catalog exhaustively admits only
`CreateRealm` as caller-flow preserving; non-callback mutations remain
invalidating. The standard-builtin catalog marks every modeled Object/Reflect
operation that can dispatch a proxy trap as synchronous user code, so exact,
spread and mixed-candidate calls share one conservative effect authority while
existing proven-safe exact returns bypass that fallback. The
[`lila-ir` module-budget owner-split contract](../docs/rust-rewrite/contracts/lila-ir-module-budget-owner-splits.md)
records the type and source-policy invariants.

Source-call caller-flow preservation now has one 769-line private
`source_call_flow_proof.rs` owner. An opaque effect carrier can reach its
proven-safe state only through a nonduplicable proof token derived by an
exhaustive finalized-invocation walk over parameter defaults plus all 34
statement, 83 expression and 29 spec-operation variants. Open or
mixed-unproved targets remain invalidating; the builtin catalog owns the exact
13 indexed-receiver mutators. Class signatures are reset before current-pass
element lowering, then monotonically merged; base constructors fold instance
initializer effects and synthetic derived constructors include their implicit
dynamic `super` call. Optional
property chains preserve facts only for proven data reads and keep accessors,
dynamic keys and unknown shapes effectful. Module policy pins the variant
census, token privacy, constructor inventory, catalog contract and line cap.

Static String flow facts now have one 33-line private
`lowering/static_string_binding_facts.rs` owner. Its child-private map is keyed
by `BindingInfo.storage_name`, and every point operation requires the binding
record rather than a source spelling. Shadowed bindings can coexist, scope exit
removes only the departing identity, and the outer fact becomes visible again.

The complete Date local-string format policy now has one private
`builtins/date/local_string.rs` owner. Its capability-free
`DateLocalStringFormat` domain, sole exhaustive consumer and all four raw
producers moved together while the standard-builtin dispatcher retained its
exact semantic call surface. The parent can no longer name, construct or
project the raw format policy. The exact five-line domain, ten-line `Date()`
producer and 270-line formatter selection retain SHA-256
`8189b9bba6c4e3c5dbb6f771fdbf23aa7a4f4e96d3898bec520380ac9e7d7916`,
`7736637485e2f8118b32ca75f7eb5e7ca5fc8cf12c826d729dfa6966d14d7cff`
and
`59c1fe7398cca3b5066118019ee541285520fe4a4305ddab4a5a932420ad64b2`;
their combined 285-line selection retains SHA-256
`a155dd53ada727aadd829f02894112843b4187f47c9316710ef932234992493a`.
The formatter's final method delimiter moves with the owner outside that
semantic selection. Including it, the 271-line physical formatter and complete
286-line physical move have SHA-256
`455adc77784d562e552aadb0ae299a73b532d77e5b9bfab4af5d97d359cd49ec`
and
`45898a09089b25568c753b7990b7fba581fa19222ab8d41810fab80466f8d069`.
The 292-line child has SHA-256
`a75df03ae28a2524c322fa66b028ca50d28d2dcda642334fd0ce0ad73dfce143`
and reduces the concurrent parent from 2,010 to 1,722 lines. The recursive
guard and module policy pin zero parent policy names, all nine format mentions
and five raw consumer/producer sites in the child, and the unchanged
one/two/two/two semantic dispatcher calls. The unchanged Date-function call
line and 25-line prototype dispatch slice retain SHA-256
`45e55826e4ee110a73b8067f64c1555e01b2d418a74fbae785a7d2c73cd597d4`
and
`d3ea28e34b1c66dec63dbdb38d299a5e1d93f2477457bbfa2d0fe6e76be567ee`.
At the 2026-08-28 Batch Y checkpoint, the exact structure target passes `7/7`,
the injected-clock engine and CLI witnesses each pass `1/1`, and `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green. Emitted-Wasm
goldens and broad semantic witnesses remain deferred, and no behavior change is
claimed.

The complete Date time-value source lifecycle now shares that same private
`builtins/date/local_string.rs` owner. `DateTimeValueSource`, its exhaustive
consumer, the sole clock-import access and all five raw producers moved
together; the parent and standard dispatcher retain only semantic emitter
calls and cannot name or reconstruct the source policy. The exact four-line
domain and 41-line consumer/current-time-wrapper selections retain SHA-256
`71ba6e635d162f63abcd7a35eb6cf7e66ae2e53b02eb43d038ec79febc0d3492`
and
`4ff02aca4eca4f2bc447c380079b9c9a6e01182b072de4366364dd9947dfa6dc`;
their combined 45-line selection retains SHA-256
`4c570b074e898f3a4b9930d42b12cb0694ce3c64e3469266c78d4acf7c6afe61`.
The resulting 1,675-line parent and 339-line child have SHA-256
`afe18d7006f8d8ffde380e8d667c56837cdef6d19ebe0e845c9434b41e0609c0`
and
`ae6d62f5ac5586704695a77839582e3a2fe8dc3fdb0b95e50b3be5157f4ec435`.
The recursive guard and module policy pin zero parent source names, raw
consumer calls and clock accesses, and exact child censuses of ten source
names, seven qualified variants, three consumer/caller sites and one clock
access. The retained six-line `Date.now()` wrapper and standard constructor
call retain SHA-256
`026e7208baa5402ac0bc15e098caf04c72b69f8cffb480d058c6759287666e63`
and
`c15bb6f7dbebf9fd006c56abfd49faf9b78e6bdc660dcc1efa81283d2bd03afa`.
This is a source-equivalent owner move. At the 2026-08-28 Batch Z checkpoint,
`cargo xc` is green, the recursive structure target passes `7/7`, both exact
backend source-oracle tests pass `2/2`, and the exact injected-clock engine and
locale-string CLI witnesses each pass `1/1`. Emitted-Wasm golden verification
remains deferred.

The complete `Intl.Locale` reserved-to-initialized-to-published result
lifecycle now has one private `builtins/intl/construction_lifecycle.rs` owner.
Its two non-`Copy`, `#[must_use]` carriers and sole reserve, initialize and
publish transitions moved together while the parent constructor retained its
exact inferred call sequence. Rust requires the carrier and transition names
to be `pub(super)`, but their tuple fields remain child-private; the parent
cannot construct or project either state, and the recursive source policy
forbids explicit naming, import or re-export. The exact 15-line carrier and
97-line transition selections retain visibility-normalized SHA-256
`f7515bf0b336e4307fac6cdefb699e32b4b3794bd0a6eff9e4f3d58113473725`
and
`7aea2daa1ccd0b8d9bdd8f5ac35eb2287eef2c686037a8e41f226c7ce659d0fa`;
their combined 112 selected lines retain SHA-256
`ca374b7f75159c8b7c978d46ee0be44be1faafb9ac34d6b8e686a200ba5d4ac4`.
The 117-line child has SHA-256
`3ebcef67424bcff990b0e6f6ed519e5c40185288d3b3ab9c21311c9110dc1bd0`.
The source move alone reduces the concurrent 2,368-line parent snapshot to
2,256 lines; the expanded colocated recursive guard leaves the current file at
2,364 lines, with 2,171 pre-test lines. The unchanged reserve call and ten-line
initialize/publish block retain SHA-256
`3fd56c270a997572d0c093e16d933bc366f4ec3f5371fb20d36835affe92c3d9`
and
`44a6f0a8622f2073daad21bf287597fee7048bd60325c79d7f1606125b4b5b4e`.
The recursive structure and module-boundary policies pin zero parent
production carrier names, the exact four/four child carrier, one/two raw
projection and two/two/two recursive transition censuses, plus the unchanged
observable ordering. `IntlLocaleConstructor` is now in the closed
direct-returning constructor domain, so generic receiver allocation cannot
perform a duplicate prototype `Get` before the lifecycle reserve transition.
At the 2026-08-28 Batch X checkpoint, `cargo xc` is green, the exact structure
unit and registered CLI fixture each pass `1/1`, and the pinned tag-coercion and
subclassing leaves pass `4/4`. The getter-order leaf remains `0/2` because
Locale options are still ignored. Broad semantic verification remains
deferred, and no published conformance count is changed.

The complete `Intl.DateTimeFormat` reserved-to-initialized-to-published result
lifecycle now has one private
`builtins/intl_datetimeformat/construction_lifecycle.rs` owner. Its two
non-`Copy`, `#[must_use]` carriers and sole reserve, initialize and publish
transitions moved together while the parent constructor retained its exact
inferred call sequence. Rust requires the carrier and transition names to be
`pub(super)`, but their tuple fields remain child-private; the parent cannot
construct or project either state, and the recursive source policy forbids
explicit naming, import or re-export. The exact 15-line carrier and 74-line
transition selections retain visibility-normalized SHA-256
`da0ef9cbc1c427a53a63bb0234432393a614beef69c9c458943f39d856d1ede9`
and
`0a20691937370549c0cf98a0216a232ba8340a320aeeadaae2358dc6b273a391`;
their combined 89 selected lines retain SHA-256
`8938d3dd2422b71750393deedcf5e00f5f0a82dbd0cdc446340d4588f44b8214`.
The 94-line child has SHA-256
`8a99ba9d18ad16e4d2321fd74e7c54b25d9646c46bd7a654139f9762224c650d`
and reduces the concurrent parent from 7,182 to 7,093 lines. The unchanged
reserve call and six-line initialize/publish block retain SHA-256
`3c4063c6330e34afc29af4509b19410c24c4acaad337eb41084a0961b1256220`
and
`c11b51e3dec4d4c0b9018cc64a875d508cc465e48d4edc1d4487695b93d0bb27`.
The recursive structure and module-boundary policies pin sole carrier,
construction, projection and transition ownership, the exact four/four and
two/two/two censuses, and the unchanged observable ordering. At the 2026-08-28
Batch W checkpoint, `cargo xc` is green, the recursive structure target passes
`1/1`, and the exact construction-order CLI fixture passes `1/1`. Pinned Intl
Test262, semantic snapshot and broad verification remain deferred, and no
behavior change is claimed.

The complete prepared BigInt radix formatting lifecycle now has one private
`builtins/bigint/radix_formatting.rs` owner. Its raw non-`Copy`, `#[must_use]`
carrier, private constructor and projections, sole producer, two formatter
reads and consuming release moved together. Only the semantic
`emit_bigint_radix_string_result` wrapper is sibling-visible; the parent keeps
its exact exhaustive policy arm but cannot name, import, construct or project
the prepared local. The exact 13-line carrier, 28-line consumer/releaser and
47-line producer retain visibility-normalized SHA-256
`6a204fe4279fe2887901a4eeac6179c52a13d3fe91e269c66c158c4e33cb1855`,
`0b8c98f44841961d75dc81143ca96ebd46c285b635a2fad8cc3f59d6f211b330`
and
`b8c715a6186a586a9ff0ecf0e967aeb9bfd5a96e626d9f566cd6c8763eb8f4f1`;
their combined 88 selected lines retain SHA-256
`aff03c509eac8acc9b67893ceb3b9992ad4a27d31798fd378d8b16114f11f5fb`.
The 94-line child has SHA-256
`3bf6fb2fa973a4c21c00c3023dff857b2e29e8e425ffcbb7910d87145ee8abe9`
and reduces the concurrent `bigint.rs` snapshot from 896 to 807 lines. The
unchanged eight-line policy arm and full 26-line result match retain SHA-256
`e42b93bc786cda7b3733d24f6d03b24500fc8d189e327ca17bd04a100e55ff7f`
and
`63fd71cea79c5f5281f142d9125ec624bdea8227c2ecb063e118ffc90fb3296f`.
The recursive structure and module-boundary policies pin sole raw carrier,
construction, projection, producer and release ownership plus one parent
semantic call. At the 2026-08-28 Batch V checkpoint, the structure target
passed `4/4`, the shared `cargo xc` gate was green, and the unchanged pinned
`radix-2-to-36.js` leaf passed `2/2`. The broader combined product fixture still
fails before the radix assertions on its unrelated main-lexical Symbol/Realm
control, so it is not reported as green. Broad semantic verification remains
deferred, and no behavior change is claimed.

The complete synchronous DisposableStack move-capability transfer now has one
private `builtins/disposable_stack/capability_transfer.rs` owner. Its
private-field, non-`Copy`, `#[must_use]`
`TransferredDisposableStackCapabilityLocals`, sole producer and sole consumer
moved together while the parent `move` choreography remains byte-identical.
Rust requires the carrier and methods to be `pub(super)` for that inferred
call chain, but the parent cannot construct or project the three raw locals;
the recursive source policy also forbids imports, re-exports and explicit
carrier naming. The exact six-line carrier, 47-line producer and 28-line
consumer retain pre-extraction SHA-256
`17f24324b20ac7a70bff152e7072c7854bd049fcf2426c4ec4b2a9e98d2b2c8c`,
`07789407e0841ab201ed86a639b902adf111185d489205f1ed193f9dc7023887`
and
`517410ab691afc729e7d5b669a46bda582dfe480f7ee45eb870a4c30c4079097`;
their combined 81 selected lines retain SHA-256
`f6892805fd8a1e7d190453354513aa4187207b92d97f54176091e3bbdcf967be`
before visibility and module indentation. The 87-line child has SHA-256
`73879c9356fabe3115c427ee769ab0942a965015944a256af8a7b3210d71a971`
and reduces the concurrent `disposable_stack.rs` snapshot from 1,139 to 1,057
lines. The unchanged ten-line `take -> install -> finalize` choreography
retains SHA-256
`9a108a075d795f380d169afe6f5a0fba169cfcc5333501fc0d50739279d4ff51`.
The recursive guard pins sole carrier, field, construction and method ownership,
the exact four/two/two identifier census, snapshot-before-clear ordering and
the unchanged parent call sequence. At the 2026-08-28 Batch U checkpoint, the
complete lifecycle structure target passed `8/8`, the exact intrinsic-unit and
CLI lifecycle witnesses passed `1/1` each, and the shared `cargo xc` gate was
green. Semantic goldens and the broad DisposableStack Test262 directories were
not rerun, and no behavior change is claimed.

The complete Reflect descriptor-object prototype-proof lifecycle now has one
private `builtins/reflect/descriptor_object_prototype.rs` owner. Its non-`Copy`
carrier, sole entry/created-Realm producer and consuming allocator/release moved
together, while `compile_reflect_define_property_builtin` retains its exact
inferred call pair. The carrier and sibling-visible methods require
`pub(super)`, but the tuple field stays child-private, so parent-side raw proof
construction or destruction does not compile; the recursive source policy
forbids explicit naming, import or re-export. The exact two-line carrier,
53-line producer and 12-line consumer retain visibility-normalized SHA-256
`b1c715d874f23c0d210ee092b547457eead1cb42557eaff40124f4fe59ba68a0`,
`0ed08206648a1d4f58e9aa3683448dd738d85ea9000d48402e66eed4b34d74f9`
and
`ed599d485865a47e4b425a5ed23630ca3b4c1e3c5c01e18a14869dc39fac1bf2`.
The unchanged six-line caller retains SHA-256
`770f6319489eb9aa746a3e1147f7484a550413db9c6c4bb4e8bc2da018fd40e5`.
The 73-line child has SHA-256
`30522774764563257635779bbb9c4f59639af31b98136d119cc42ca9dd38688f`
and reduces the concurrent `reflect.rs` snapshot from 2,415 to 2,347 lines.
The retargeted recursive guard pins five child-only carrier identifiers, sole
construction/destruction, zero imports/re-exports, both Realm routes, three
required-state traps and the unchanged parent call order. The retargeted
structure target passes `4/4`, the engine Realm witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. No behavior or conformance change is
claimed for this source-equivalent move.

The complete PromiseResolve Realm-context lifecycle now has one private
`builtins/promise/promise_resolve_realm_context.rs` owner. Its operation and
intrinsic paired contexts, three factories, call, two releases and intrinsic
resolve consumer moved together. A narrow abrupt-normalization capability
operation also owns the last constructor-payload projection, so the Promise
parent and finally-completion sibling can pass only inferred contexts between
child-owned operations. Rust requires both carrier names to be `pub(super)` in
those sibling-visible signatures, but their fields remain private; parent-side
construction and projection do not compile, and the recursive source policy
forbids explicit naming, import or re-export. The exact ten-line carrier,
103-line factory, 41-line call/release and 26-line resolve selections retain
pre-extraction SHA-256
`d15798a3e31f7b38ad6f9779797a480304d12321e438aef6d74de711a6c801f9`,
`77f4c4bf5164acc705865aef1e567e930f326df895c94e062dd6c5bd85a3113c`,
`2a895e67fa55537efdc0d230f253ccef95172cacce32d0f6f12b6250c9a44110`
and
`a76484e712c9c620512d11804f541fbc0e39fd128520728a7887747e62c1c3`
after normalizing only required `pub(super)` visibility. The abrupt wrapper's
exact eight-line capability block retains SHA-256
`557738476f2f2f01137b243dda634ec95cd0e119f2b23059a246c78a5b7a627f`.
The 206-line child has SHA-256
`9aefd81a7d9fee98addd74c002a897fd6c9815306556e6e22d99320970814842`
and reduces the concurrent `promise.rs` snapshot from 7,488 to 7,304 lines.
The retargeted recursive Realm-context and authority guards pin sole carrier,
construction, projection and lifecycle ownership, the exact `4/5/1`
parent/Realm-child/finally-child authority census, unchanged inferred callers
and zero import/re-export paths. The internal-function materializer census is
now split `5/2/1/1/2` across parent, Realm child, combinator-materialization
child, finally child and keyed-combinator child, preserving eleven total.
Batch S used only non-compiling source, formatting, hash and diff checks during
implementation. At the coordinated checkpoint,
`promise_resolve_realm_context_structure` and
`promise_resolve_realm_authority_ownership_structure` each pass `4/4`, while
`promise_internal_function_realm_context_structure` passes `6/6`, for `14/14`
focused structure checks. The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green. Semantic goldens were not
rerun because this is a source-equivalent owner move.

The complete Promise internal-function materialization lifecycle now has one
private
`builtins/promise/promise_internal_function_materialization.rs` owner. Its
non-`Copy`, must-use four-local carrier, three factories, borrowing
materializer, closure-context loader and consuming release moved together.
Rust requires the carrier and methods to be `pub(super)` in retained parent and
sibling-visible signatures, but all fields remain child-private. The parent has
one private import for its unchanged inferred caller bodies; neither it nor a
sibling can construct or project the raw context. A narrow child-owned
Realm-intrinsics capability also replaces PromiseResolve's last direct
`realm_local` projection while emitting the same load. The exact 14-line
carrier and 179-line lifecycle selections retain pre-extraction SHA-256
`f4e235ec31396e2a6d937505c6ccaa59fe7450498b262f72060889a017ca057d`
and
`39e984084505b37e9c3c95b73a0a4f05bce82ed8e020cee738b42138b5cbe2ce`;
the combined 193-line selection retains visibility-normalized SHA-256
`45b615e0aad9e0deb0c63a620408304a0c2729a8c5471985dc39471798453ef3`.
The former six-line raw sibling projection has SHA-256
`77ad4ff7cf2a1b826b9a8093df2deb6aab2ff8919860651d2dab6d944e59320d`,
and its five-line capability call has SHA-256
`a9b67364fab3ee72ee79d4f3421c4975e96a7135b6a0a88e98ad6f67996e3812`.
The 212-line child has SHA-256
`18506619c5365cbb4354ead3759f98cc235a6e8581432f8f4c7235fc45039556`;
the reduced 7,111-line parent has SHA-256
`eea99601f506ffa59e1870607a04e24481b5ea83349aedb0ee439ff273d617f6`.
The recursive guards pin eleven carrier identifiers, the exact
`4/7/2/11/9/9/2` lifecycle/capability census and zero PromiseResolve raw Realm
projections. At the coordinated Batch AG checkpoint, the internal-function,
PromiseResolve Realm-context and callback-created-allocation structure targets
pass `6/6`, `4/4` and `7/7`, for `17/17` focused structure checks. The exact
`functions::run_wasm_backend_preserves_created_realm_promise_internal_callbacks`
and
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witnesses each pass `1/1`, and shared `cargo xc` is green. No Test262
cohort or semantic golden was run because this is a source-equivalent owner
move.

The complete prepared-AggregateError carrier lifecycle now has one private
`builtins/errors/aggregate_error_preparation.rs` owner. Its non-`Copy`,
private-field `PreparedAggregateErrorLocal`, the two origin-specific producers
and the consuming finalizer moved together without changing the constructor or
Promise.any caller bodies. The parent and sibling can pass the inferred
`pub(super)` carrier between child-owned methods, but cannot name, import,
re-export, construct or project it. The exact four-line carrier and 109-line
method blocks retain pre-extraction SHA-256
`59ab0448985ad9c7999915fdec43a0d61fdb5515c4eca589644fc8e53b15d5fd`
and
`8e329e9650c12ffc058d579336106ab0697bb75936e8ef63ee76d1924291c337`;
their combined 113 selected lines retain SHA-256
`c088d2a5a86fb16727fb262bae571a421a54f0f0247d2df25f03e55064af5e63`.
The 118-line child has SHA-256
`575af1e9b93d451beea17409ada96654d169668747f91432a8161a834d96490e`
and reduces the concurrent `errors.rs` snapshot from 1,557 to 1,443 lines. The
strengthened recursive construction and module-boundary witnesses enforce sole
carrier, field, construction, projection and method ownership plus the exact
one constructor preparation, one Promise.any preparation and two finalization
calls. This source-equivalent checkpoint uses only scoped structure, boundary,
task-plan, formatting and diff verification. The recursive structure target
passes `3/3`, and each dry audit is green. The shared `cargo xc` checkpoint and
the exact AggregateError constructor-properties CLI control pass. Semantic
goldens were not rerun for this source-equivalent extraction.

The complete `Promise.try` callback-TypeError prototype proof lifecycle now has
one private `builtins/promise/promise_try_callback_type_error.rs` owner. Its
non-`Copy` proof, sole factory and consuming throw emitter moved together
without changing the retained `Promise.try` caller. The parent may pass the
inferred `pub(super)` proof between child methods, but cannot name, import,
re-export, construct or project its raw prototype. The exact two-line proof and
46-line method blocks retain pre-extraction SHA-256
`ab90ccc6decb25132becf4d66e08b2ee989ea3795974428d29cc59ccd7b60737`
and
`b236edae20d47dc546d9c641ddb6841086ee117205610fb9de4fc19c8c3163f1`;
their combined 48 selected lines retain SHA-256
`8f17cb7d9f467e43b229d96e6354fbce421746b6d01518b7491b5bab513b6eb2`.
The 53-line child has SHA-256
`64b2ce31da7fe3ac71a60e93302388ed0fd2ef81c390d0ce019e395ed5c6aff3`
and reduces the concurrent `promise.rs` snapshot from 9,606 to 9,558 lines.
The strengthened recursive structure and module-boundary witnesses enforce
sole proof, tuple construction, projection and method ownership plus the exact
one factory/consumer call pair. This source-equivalent checkpoint uses only
scoped structure, boundary, task-plan, formatting and diff verification. The
recursive structure target passes `5/5`, and each dry audit is green. The exact
created-Realm Promise internal-callback CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. Semantic goldens were not rerun for
this source-equivalent extraction.

The complete Promise prototype receiver-TypeError proof lifecycle now has one
private `builtins/promise/promise_prototype_receiver_type_error.rs` owner. Its
non-`Copy` proof, sole factory and consuming throw emitter moved together. The
closed diagnostic selector now also belongs to that private child; its raw
consumer is child-private, and the parent can call only the named
`then`-incompatible and `finally`-non-object wrappers. The parent may pass the
inferred `pub(super)` proof between child methods, but cannot name, import,
re-export, construct or project its raw prototype or raw error policy. The
exact two-line proof and 47-line method blocks retain
pre-extraction SHA-256
`f674791ba2a55602068a89bda2c001417a2bf38c369f69bafb3ba5391d4a8ee9`
and
`da7c455d62b10e71264034444dd59ff767c7b079384fffa0454e8f39212de998`;
their combined 49 selected lines retain SHA-256
`9156f4cd0689dd3c78bdedfbf1fb356c32e0b2f179f5103a4ace7e3d2fa2f457`.
The 54-line child has SHA-256
`8e6935089851bc5087be8dbfc019b88b0c644d03a6c9183462aec2b0dbc4ad80`
and reduces the concurrent `promise.rs` snapshot from 9,558 to 9,508 lines.
The strengthened recursive structure and module-boundary witnesses enforce
sole proof, tuple construction, projection and method ownership plus the exact
two factory/consumer caller pairs. This source-equivalent checkpoint uses only
scoped structure, boundary, task-plan, formatting and diff verification. The
recursive structure target passes `8/8`, and each dry audit is green. The exact
created-Realm Promise internal-callback CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. Semantic goldens were not rerun for
this source-equivalent extraction.

The follow-up diagnostic-policy closure moves the exact 14-line
`PromisePrototypeReceiverError` domain and exhaustive message projection at
SHA-256
`475de0cb7a31556b182f6e705f8c5b64cbf33f1bd4d267bc883c3b028b7ca1f8`
into the existing private owner without changing the block. Its two named
semantic wrappers add 31 lines at SHA-256
`0a470e691356d67011c025864045b7f6d5767ed814dfc7e39742ba85ef4c1a5a`.
The resulting 101-line child has SHA-256
`f8d25e1f5a4950fb1e01abde37a0bce0048af500fc1c65af3bd4054110605719`
and reduces the concurrent `promise.rs` snapshot from 7,608 to 7,591 lines.
The recursive structure and module-boundary witnesses pin the sole private
domain, exhaustive projection, private raw consumer, exact wrapper-to-variant
mappings and one parent caller per wrapper. Batch P uses only non-compiling
source checks during implementation. At the coordinated checkpoint, the
recursive structure target passes `8/8`, the exact created-Realm runtime
witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green. Semantic goldens were not
rerun for this source-equivalent move.

The complete standard Promise combinator reaction-pair lifecycle now has one
private `builtins/promise/promise_combinator_reaction_pair.rs` owner. Its
non-`Copy` `PromiseCombinatorReactionPairLocals` fulfilled/rejected carrier,
exhaustive three-mode construction,
consuming projection and sole `then` invocation moved together. The parent can
call only the named `pub(super)` semantic operation and cannot name, import,
re-export, construct or project the raw pair. The exact five-line carrier and
43-line selection/projection/invocation block retain SHA-256
`ebdea093363ba0f80b64d7787525bf050b47b85c2f2146b1b6e4306da98b8585`
and
`d6f594bb8c009eafbfe9414be7b3342967920529e8a71c1bd500c246168440e3`
after relocation. The resulting 74-line child has SHA-256
`0566a6451be281d6da341e602695a683986e2a45d5e31493813bbe8ae795ae0e`
and reduces the concurrent `promise.rs` snapshot from 7,591 to 7,561 lines.
The recursive `promise_combinator_reaction_pair_ownership_structure` guard pins
zero import/re-export paths, five child-only carrier mentions, sole
construction and projection, exact callback order and one semantic parent
call. The neighboring mode guard pins the unchanged six
standard policy projections as diagnostic plus four parent-body and one
child-owned reaction decisions. The two focused structure targets pass `6/6`,
the exact all-mode runtime witness passes `1/1`, and the shared `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green. Semantic
goldens were not rerun for this source-equivalent owner move.

The complete Promise combinator element-function materialization lifecycle now
has one private
`builtins/promise/promise_combinator_element_materialization.rs` owner. Its
non-`Copy`, must-use internal-function/AggregateError-prototype carrier, sole
active-function factory, sole borrowing materializer and consuming release
moved together without changing the standard-combinator caller body. The
parent passes the inferred carrier between child-owned operations. Rust
requires its name to be `pub(super)` in those sibling-visible signatures, but
both fields stay child-private, so parent-side construction and projection do
not compile; the recursive source policy forbids explicit parent naming,
import or re-export. The original exact five-line private carrier has SHA-256
`c7430a277ef2c67049ff8e71f75889af4a80580649a3b9d0e57c63e3197f2e3c`.
Its relocated raw hash is
`c0270ce24ae08522ec895d3781adc132e4b0ef2b1e109122c067d30c99ab47f6`,
while visibility normalization retains the original hash;
the exact 35-line factory, 23-line materializer and seven-line release retain
SHA-256
`cb48555544e8ef28bfb0d7663e43f8fdb81303c13fa5c1cd11812ac03e660d07`,
`b51507411423930036d4aa3a74f88e670414c47dbf9f198d8100e4a09b340f2c`
and
`63d45e822fadada44a6b05de7791dcf0be7937d87194872628715637f10bd35f`
after normalizing only the required sibling visibility. The resulting 77-line
child has SHA-256
`ca669fc19647144028e80761f7d62015f1ec3f1c71d6a2d960178e3a7aa91cf1`
and reduces the concurrent `promise.rs` snapshot from 7,561 to 7,488 lines.
The recursive `promise_callback_created_allocation_realm_structure` guard pins
five child-only type mentions, sibling-only type visibility, private fields,
sole construction, exact field projections, the `1/2/1` parent call census and
zero parent name/import/re-export paths; the internal-
function census remains split across ten parent and one child materializer
mentions. Batch R used only non-compiling source checks during implementation.
At the coordinated checkpoint, the callback-created-allocation, internal-
function and PromiseResolve structure targets pass `7/7`, `6/6` and `4/4`
respectively (`17/17` total). The exact
`functions::run_wasm_backend_uses_callback_realms_for_promise_created_allocations`
CLI witness passes `1/1`, and `cargo xc` is green after the required
`pub(super)` carrier-privacy correction. Semantic goldens were not rerun for
this source-equivalent owner move.

The complete validated Promise prototype delegated-`then` lifecycle now has
one private `builtins/promise/promise_prototype_then_invocation.rs` owner. Its
non-`Copy` method/receiver carrier, sole validator and consuming Call emitter
moved together without changing the retained `catch` and `finally` callers.
The parent may pass the inferred `pub(super)` carrier between child methods,
but cannot name, import, re-export, construct or project its paired fields. The
exact five-line carrier and 45-line method blocks retain pre-extraction SHA-256
`5d76c430bb3d979c257021485616e144a97ed64a56543d25fb2e305b92ae3e0e`
and
`42107b7cb4e722fb213841ee3c72cf350a839562e979da591f95affee35eca02`;
their combined 50 selected lines retain SHA-256
`294e1d92a3579b3cab9abd0625d501044d1c03fb13978a5193b1d0719a4fe89d`.
The 55-line child has SHA-256
`a630f2c5aeb0af045a090078e891a552226701ea7dbbf88c1e074bdb842d192a`
and reduces the concurrent `promise.rs` snapshot from 9,508 to 9,457 lines. The
strengthened recursive structure and module-boundary witnesses enforce sole
carrier, construction, projection and method ownership plus the exact two
validator/consumer caller pairs. This source-equivalent checkpoint uses only
scoped structure, boundary, task-plan, formatting and diff verification. The
recursive structure target passes `8/8`, and each dry audit is green. The exact
created-Realm Promise internal-callback CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. Semantic goldens were not rerun for
this source-equivalent extraction.

The complete Promise all-settled record allocation proof lifecycle now has one
private `builtins/promise/promise_settlement_record_allocation.rs` owner. Its
non-`Copy` Object-prototype context, sole self-backed factory and consuming
allocator moved together without changing the standard or keyed all-settled
callers. The parent may pass the inferred `pub(super)` context between child
methods, but cannot name, import, re-export, construct or project its raw
prototype. The exact four-line carrier and 61-line method blocks retain
pre-extraction SHA-256
`fe4dae7d7c230d964adbff8da382a9268049d19ee3fb8262544788d09802bbc2`
and
`9d5475d82fb38f03f9967e900e13ea31f79cc1fc96ec7a3c8089f84067e16fc1`;
their combined 65 selected lines retain SHA-256
`a3a37f83754be67d87f914f86ed5d823cb9a0d0147d4cbf762e2ef619f1c66c9`.
The 70-line child has SHA-256
`ca39265d56101bd92f0dd324aad35e115ef4dfb51e47822a0850841c81327305`
and reduces the concurrent `promise.rs` snapshot from 9,457 to 9,391 lines. The
strengthened recursive structure and module-boundary witnesses enforce sole
carrier, construction, projection and method ownership plus the exact two
factory/allocator caller pairs. The recursive structure target passes `7/7`,
the exact created-Realm Promise allocation CLI witness passes `1/1`, and the
shared `cargo xc` checkpoint is green. Workspace formatting, diff hygiene,
module boundaries and task-plan policy are green. Semantic-golden execution
does not apply to this source-equivalent extraction.

The complete `Promise.withResolvers` result-prototype allocation lifecycle now
has one private
`builtins/promise/promise_with_resolvers_result_allocation.rs` owner. Its
non-`Copy` Object-prototype context, sole current-function-Realm factory and
consuming installer moved together without changing the retained builtin
caller. The parent may pass the inferred `pub(super)` context between child
methods, but cannot name, import, re-export, construct or project its raw
prototype. The exact four-line carrier and 74-line method blocks retain
pre-extraction SHA-256
`a65d951c68f000af43899262f7e8fb82dbc543c1849024841fb5aa9af98a20e2`
and
`7711cc79c15e769c56e3f642e9e0cb538ecddbcfeb1db1c07945f7d6ff08306b`;
their combined 78 selected lines retain SHA-256
`58a16feba356e97d5fad4bac3311c632335a03d7105df42c9492347c3fb448bf`.
The 83-line child has SHA-256
`3c3ee3ee1187f412e1b0a8232ca6a77e03fc94d7e3d0f50d4dd3e0d68163ba2a`
and reduces the concurrent `promise.rs` snapshot from 9,391 to 9,312 lines. The
strengthened recursive structure and module-boundary witnesses enforce sole
carrier, construction, projection and method ownership plus the exact one
factory/installer caller pair. The focused recursive owner target passes `5/5`
and the neighboring created-allocation-Realm target passes `7/7`; module-boundary,
task-plan, formatting and diff-hygiene checks pass. The shared `cargo xc`
checkpoint is green, and the exact created-Realm Promise publication CLI
witness passes `1/1`. A semantic golden does not apply to this source-equivalent
owner move.

The current-function-Realm intrinsic Promise capability proof now has one
private
`builtins/promise/current_function_realm_intrinsic_promise_capability.rs`
owner. Its non-`Copy`, must-use constructor witness, sole defining-Realm
factory and consuming capability adapter moved together without changing the
grouped async-generator request dispatcher. The Promise parent cannot name,
import, re-export, construct or project the proof; the dispatcher can only pass
its inferred value between child-owned methods. The exact 11-line proof,
54-line factory and 24-line consumer retain pre-extraction SHA-256
`ee77d3da82ce3ff12687a42d0ba048e6106f4b0274b275fee96600fd61284cda`,
`6d788c661b1e39cc835862440c3e8963fef107b1b416dae1bc87a3fa9e57ef23`
and
`5d7d6497aee9052a765c8d9b558e86741ec9cbc114835efff31800332dc036f4`;
their combined 89 selected lines retain SHA-256
`d8f7520291ba35c725b72ad5a36ebdd58565c1cf5e78b9c4eadfb6c94ef717dd`.
The 92-line child has SHA-256
`c83e3819f36ef479aefa13d18b040fe73349cab0af739337b3471450c8e75bd3`,
and the concurrent Promise parent is 9,224 lines at this checkpoint. The
recursive owner structure and both retargeted neighboring targets pass `4/4`,
pinning the private module, zero imports/re-exports, sole construction, two raw
projections and exact factory/consumer ownership. The existing CLI behavior
witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checkpoints are green. The semantic golden
remains deferred; no behavior or conformance change is claimed.

The paired Promise SpeciesConstructor Realm lifecycle now has one private
`builtins/promise/promise_species_realm_context.rs` owner. Its non-`Copy`,
must-use default-constructor/TypeError context, sole defining-Realm factory and
consuming SpeciesConstructor helper moved together without changing the
retained `then` and `finally` callers. The Promise parent cannot name, import,
re-export, construct or project the paired proof; callers can only pass its
inferred `pub(super)` value between child-owned methods. The exact six-line
context, 71-line factory and 110-line consumer retain pre-extraction SHA-256
`4c668cadeb82c06ec0e1d66e0c8baf3c1417d407e077e8400b7671af0158dfd2`,
`676d21f916e740dd4e90a92c87c27970a0371e6d23ed8ef20b8c3d37445f3aa5`
and
`e9faaabf5ea9fac01908d9c653caba9f64d7850fded52961ac4c1ddc5c2abe46`;
their combined 187 selected lines retain SHA-256
`3ce5bdf128aa01e4f1586af03ed56f48a98d4e64f4d28430b59d5826471c4188`.
The 190-line child has SHA-256
`91c2fe569f01c52e088cd49c1299a51afc7a3e3caf16b486d2419fd84e077d3e`
and reduces the concurrent `promise.rs` snapshot from 9,224 to 9,038 lines.
The recursive owner target passes `6/6` and the adjacent receiver-order target
passes `8/8`, pinning the private module, zero imports/re-exports, sole paired
construction, exact raw-field projections and two factory/consumer caller
pairs. The existing created-Realm Promise callback CLI witness passes `1/1`,
and the shared `cargo xc`, formatting, diff, module-boundary and task-plan
checks are green. The semantic golden remains deferred; no behavior or
conformance change is claimed.

The paired Promise combinator algorithm-error Realm lifecycle now has one
private
`builtins/promise/promise_combinator_algorithm_error_realm.rs` owner. Its
non-`Copy`, must-use TypeError/RangeError context, sole defining-Realm factory,
two typed error borrowers and consuming release moved together without
changing the retained race, keyed or standard combinator bodies. The Promise
parent cannot name, import, re-export, construct or project the paired proof;
callers can only borrow its inferred `pub(super)` value through child-owned
methods. The exact six-line context and 110-line method block retain
pre-extraction SHA-256
`b66cf09315fe22f69c6dd74d3ed3deb752f9ced7c0d08450ad94e948f55e1fbc`
and
`799c7f889d06dd6a69371508557e259725881d0ee1127f3cb5f7377d20d31aa3`;
their combined 116 selected lines retain SHA-256
`49037556ac09bceed8e1f7138b409b66f2503d9bc55d8101fe1014f81731df8e`.
The 119-line child has SHA-256
`c7cf098541a7a0ef5c15ad3e105fcfe8b8ebb5c4a3c96fedfb3a5fdbdca24223`
and reduces the concurrent `promise.rs` snapshot from 9,038 to 8,923 lines.
The recursive include-only owner target passes `5/5`, pinning the private
module, zero imports/re-exports, six child-only type uses, sole construction,
exact raw-field projections and exact `3/14/1/3` parent call census. Cargo
execution of the same focused target also passes `5/5`, the neighboring
combinator-mode target passes `3/3`, and the existing created-Realm CLI witness
passes `1/1`. The shared `cargo xc`, formatting, diff, module-boundary and
task-plan checks are green. No new semantic golden was captured for the
source-equivalent move; no behavior or conformance change is claimed.

The fully selected Promise-job enqueue authority now has one private
`builtins/promise/promise_job_to_enqueue.rs` owner. Its non-`Copy` two-variant
payload domain, complete reaction and thenable producers, exhaustive consumer
and sole FIFO append moved together without changing the retained reaction-list,
Promise-resolution or settlement callers. The Promise parent cannot name,
import, re-export, construct or project a selected job; callers can only invoke
the two child-owned `pub(super)` producers, and the raw consumer remains private
to the child. The exact 15-line domain, 17-line reaction producer, 52-line
thenable producer and 122-line consumer retain pre-extraction SHA-256
`372060fd20211269848fa8ff18be925ee15bcc771ec2d2be4f2460b1d138b58d`,
`4086b791c4da713c743617447fd8e75448fe02eb76091e0bc61b3a8caf3081e7`,
`6d814a02b3ac71f0dcee2a8ca94574d2e91246d2dec826478b76c6775e973c6d`
and
`351e4c06208555795e68495874a346fb4aca978690c237a06e0e89b6757e6f4b`;
their combined 206 selected lines retain SHA-256
`1d8d89b234378b015344445320aabbd5787ff0494aa7da4194457f46fd5850e9`.
The 210-line child has SHA-256
`c4587437a74e5788055bb21104c1f5096518a7933feba487be7619287c3416c6`
and reduces the concurrent `promise.rs` snapshot from 8,923 to 8,717 lines.
The retargeted recursive owner target passes `3/3`, pinning the private module,
zero imports/re-exports, six child-only authority mentions, sole constructions,
one owned parameter, two exhaustive projections, two producer-to-consumer
routes and the exact FIFO append. The reaction-FIFO and thenable-settle-once
engine witnesses pass `1/1` each, the created-Realm internal-callback CLI
witness passes `1/1`, and two pinned Promise leaves pass all `4/4` Wasm-AOT
executions with every failure bucket at zero. The shared `cargo xc`, formatting,
diff, module-boundary and task-plan checks are green. No semantic golden or broad
suite was run for this source-equivalent move; no behavior or conformance change
is claimed.

The closed Promise `finally` completion authority now has one private
`builtins/promise/promise_finally_completion.rs` owner. Its non-`Clone`,
non-`Copy` two-variant domain, two consuming projections, four named semantic
producers and two raw consumers moved together without changing the four
standard-builtin dispatcher calls or any method body. The Promise parent and
dispatcher cannot name, import, re-export or construct the raw completion;
only the unchanged `pub(crate)` semantic wrappers cross the child boundary.
The exact 25-line domain/policy and 219-line method lifecycle retain
pre-extraction SHA-256
`c9f079447b9c82792ae69b5438e13811ef9ac5a99c62080052837eb9a6b0edf3`
and
`e42e360bf0d1739557b05ffd2c5429b3cde863d6f4bb9c75e8737f858aa4dcb6`;
their combined 244 selected lines retain SHA-256
`464867bbceb6ffda71e52cef1b733fe254abd36d76a3eef9fb16c95d4c1501d6`.
The 248-line child has SHA-256
`8aff4f6c500f2eaaf48f61b68ddfe5c5fac0f534cd95b1d2fabd5302bb1ceef3`
and reduces the concurrent `promise.rs` snapshot from 8,717 to 8,473 lines.
The retargeted include-only recursive owner target passes `4/4`, pinning the
private module, zero imports/re-exports, eight child-only domain mentions, the
four exact producers, two exact consuming projections and unchanged complete
consumer fingerprints. Targeted formatting, diff hygiene, module-boundary and
task-plan checks are green. The Promise-finally settlement engine witness and
created-Realm internal-callback CLI witness each pass `1/1`. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
No semantic golden or broad suite was run for this source-equivalent move; no
behavior or conformance change is claimed.

The closed keyed Promise element projection authority now has one private
`builtins/promise/promise_keyed_element_projection.rs` owner. Its two-variant
fulfilled-value/settlement-record domain, two named semantic producers and sole
raw consuming emitter moved together without changing either `pub(crate)`
wrapper or the standard-builtin dispatcher. The Promise parent and dispatcher
cannot name, import, re-export, construct or project the raw choice; a
settlement direction can enter only through the settled-keyed wrapper. The
exact four-line domain and 224-line producer/consumer lifecycle retain
pre-extraction SHA-256
`800bd5beb3809f1076d8ba44ad9ff1e1b4fbbe84a94a03cc48dc5ad40e013db2`
and
`a603dc1fc5222699b626849223d31816a5215428692e0a0034d756f1a09f812d`;
their combined 228 selected lines retain SHA-256
`8933e0453be7da7baab292babfd052c663ffa6923cbdcb53715cd4d7bc9300df`.
The 232-line child has SHA-256
`ddc8233e8b079fede9ea977e5cdda3cca449e79dae8a0943a42c910888651d24`
and reduces the concurrent `promise.rs` snapshot from 8,473 to 8,245 lines.
The retargeted include-only recursive owner target passes `3/3`, pinning the
private module, zero imports/re-exports, six production authority mentions,
the exact two producer choices, one owned consumer and both exhaustive
projection arms. It also pins the child-owned typed settlement-record
allocation after the earlier allocation-owner extraction. Targeted formatting,
diff hygiene, module-boundary and task-plan checks are green. The keyed
all-settled engine witness and all-five-mode CLI witness each pass `1/1`. The
shared `cargo xc`, formatting, diff, module-boundary and task-plan checks are
green. No semantic golden or broad suite was run for this source-equivalent
move; no behavior or conformance change is claimed.

The restricted keyed Promise combinator mode now has one private
`builtins/promise/promise_keyed_combinator_mode.rs` owner. Its two-case
`Values`/`SettledRecords` domain, both named semantic producers and sole raw
keyed lowerer moved together without changing either `pub(crate)` wrapper or
the standard-builtin dispatcher. The Promise parent and dispatcher cannot
name, import, re-export or construct the raw keyed mode, so first-fulfillment
policy cannot be supplied to keyed lowering through a parent-owned call. The
exact five-line domain and 632-line producer/consumer lifecycle retain
pre-extraction SHA-256
`489be0d316aa862e31ef48f6b526e40233f171f2066dd47abb6c8b382d6459ba`
and
`f38a4742ce2a901162f25f68bb1cb6cf3e353862c4212805d4b5b92d569cf363`;
their combined 637 selected lines retain SHA-256
`aaf6deb2d7557380c453c06da4a2e5b2a22f42aa00f9aed307e9ed75fed37f5f`.
The 641-line child has SHA-256
`3b36cbf4fb06350f3dfd551c31a180b6ac523101f30aceac4d614082097deda7`
and reduces the concurrent `promise.rs` snapshot from 8,245 to 7,608 lines.
The retargeted include-only recursive mode target passes `3/3`, pinning the
private module, zero imports/re-exports, ten child-only keyed-mode mentions,
the exact two wrapper choices and all three exhaustive raw policy projections,
while preserving the parent-owned standard mode checks. The keyed all-settled
engine witness and all-five-mode CLI witness each pass `1/1`, and the shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
No semantic golden or broad conformance suite was run for this
source-equivalent extraction; no behavior or conformance change is claimed.

The required resolved-Realm ordinary-prototype lifecycle now has one private
`functions/required_resolved_realm_ordinary_prototype.rs` owner. The closed
nine-variant `OrdinaryDefaultPrototype`, its exhaustive realm-slot map, the
non-`Copy` resolved prototype witness and the load, full new-target orchestration
and consuming-install methods moved together without changing the four retained
generic construct pairs or two Error-family callers. The parent re-exports only
`OrdinaryDefaultPrototype`; it can pass the `pub(super)` witness as an inferred
value, but cannot import, construct or project its private raw local. The exact
43-line domain/witness and 99-line method blocks retain pre-extraction SHA-256
`b37af658ad2dae3817a94c070da0305488686510057e9b60d586e2d726cbf9a4`
and
`c1b1786018f52bee3a8d37f140d0c95319cc88ba783fc8387f7d4ebb44b0e401`;
their combined 142 selected lines retain SHA-256
`6889cb20756041dce60e685cd3f61b9c0ee8af20f6fdc0ec17159c2c5384a8f9`.
The 147-line child has SHA-256
`39692aaa0f33487df324707b4e7bddfa82d03af873f1dda87ee303844d9c7907`
and reduces the concurrent `functions.rs` snapshot from 12,269 to 12,127 lines.
The recursive structure and module-boundary witnesses enforce sole type,
offset, tuple, projection and method ownership, the narrow re-export and exact
five-load/five-install/three-orchestration census. Existing constructor and
FunctionRealm owner slices now read the child while their caller assertions
remain on the original files. This source-equivalent checkpoint uses only
scoped structure, exact-unit, boundary, task-plan, formatting and diff
verification. The new structure target passes `3/3`, the retargeted
FunctionRealm structure target passes `4/4`, and all six exact constructor
units pass. The Error-family unit's stale diagnostic-name expectation now
matches the existing `name: "TypeError.prototype"` spelling shared by the heap
and module registries; no layout marker or runtime behavior changed. The shared
`cargo xc` checkpoint passes. CLI and semantic-golden verification remain
unrun for this source-equivalent extraction.

The complete created-Realm `%Array.prototype%` lifecycle now has one private
`functions/created_realm_array_prototype.rs` owner. Its non-`Copy` reserved and
initialized local states and six reserve, initialize, publish, define, bind
and consuming-release methods moved together without changing the sole host
bootstrap caller. The parent neither imports nor re-exports either inferred
state, so no sibling or retained parent method can construct a lifecycle proof
from a raw local. The exact 16-line state and 168-line method blocks retain
SHA-256
`d557dc697bfaf3c5b9ac81521126963a18f1c5fbb7cd11ab7afbad94d76d0b0a`
and
`1784e1c9ebf445d237c1da5ad952e250a3a5d1cf8dd15c773bae6bf2a19aea17`;
their combined 184 selected lines retain SHA-256
`10e8fae5d82a0ae8440df773b12241da77102853725bbb35bbd7e99d8e279fa1`.
The 189-line child has SHA-256
`22b55e6995b096b43ee910a8fd158b73d6d0f455e37dde1ffb78277a65e988f2`
and reduces `functions.rs` from 12,452 to 12,267 lines. The new recursive
structure and module-boundary witnesses enforce sole type, tuple, projection
and method ownership, the exact `1/1/1/3/1/1` call census, initialization and
property attributes, and the unchanged host lifecycle order. The neighboring
callable-prototype guard's end marker was retargeted to the immediately
following retained intrinsic-slot documentation after its former marker moved
with this owner. This source-equivalent checkpoint uses only scoped structure,
boundary, task-plan, formatting and diff verification. The new structure target
passes `3/3`; the neighboring callable-prototype target remains at its known
`7/8` because its planning-root slice observes zero Function-prototype roots
where it expects one, outside this extraction and marker maintenance. CLI,
semantic-golden and broad compilation remain deferred.

The active-function-Realm Array-prototype proof now has one private
`functions/current_function_realm_array_prototype.rs` owner. The opaque,
non-`Copy` `CurrentFunctionRealmArrayPrototypeLocal` and its load/consuming-
install methods moved together without changing their sole Array allocator or
the Promise and Iterator consumers. The parent neither imports nor re-exports
the witness, so no sibling or retained parent method can manufacture the proof
from a raw local. The exact 83 moved source lines retain SHA-256
`5e46c40f4844d54c08c0d74f25cc6046fef7063ad255be7e73add4dd5e87b490`.
The 88-line child has SHA-256
`58863a33bd4367870947a6b562b619e5fd1559e4464347fa96212ba472c09739`
and reduces `functions.rs` from 12,533 to 12,451 lines. The retargeted unit,
integration and module-boundary witnesses enforce sole type, tuple, projection
and method ownership, the one load/install pair after successful raw
allocation, and the two higher-level allocator consumers. This
source-equivalent checkpoint uses only scoped structure, boundary, task-plan,
formatting and diff verification; CLI, semantic-golden and broad compilation
remain deferred.

The complete Arguments ParameterMap carrier lifecycle now has one private
`functions/arguments_index_mapping.rs` owner. The non-`Copy`, private-field
`ArgumentsIndexMappingLocals` and its capture, read, write, restore and
consuming-release methods moved together without changing the five caller
bodies. The parent neither imports nor re-exports the carrier, so sibling and
parent emitters cannot assemble mapped-state and slot locals independently.
The exact 153 moved source lines retain SHA-256
`1866bb0a7938406f35929397e94c201335092e48a0cd9f631e36803b30511195`.
The 158-line child has SHA-256
`7d68d462b7a5419a1306d21cb0ddcef8ebcfc886a396199db2a1fb7a9a25fa43`
and reduces `functions.rs` from 12,685 to 12,533 lines. The strengthened
recursive structure and module-boundary guards enforce sole type, literal,
field and method ownership plus the exact five captures, three reads, four
writes, one restore and five releases. This source-equivalent checkpoint uses
only scoped structure, boundary, task-plan, formatting and diff verification;
the structure target passes `4/4` and each dry audit is green. CLI,
semantic-golden and broad compilation remain deferred.

The complete `GetFunctionRealm` result lifecycle now has one private
`functions/function_realm.rs` owner. Its closed outcome and revoked-route
domains, raw and resolved local states, and Get, route and release methods moved
as one source-equivalent family. The parent re-exports only
`FunctionRealmRevokedRoute` for the three existing sibling consumers and
privately imports `ResolvedFunctionRealmLocal`; the raw result and outcome stay
behind the child boundary. The exact 272 selected source lines retain SHA-256
`4305eed14fcf73c1330411004824c66f73bacbf3421cf1bfcc901c25bc2ae548`.
The formatted 278-line child has SHA-256
`a62b41ffde6966725c2d18e594fd1232ceed9cd632362389de64dc5ae3107415`
and reduces `functions.rs` from 12,956 to 12,684 lines. The retargeted four-test
recursive guard enforces sole ownership, the narrow import/re-export surface,
the raw-result lifecycle and the existing five Get/route pairs. This checkpoint
uses only scoped structure, boundary, format and diff verification; CLI,
semantic-golden and broad compilation remain deferred to the coordinated shared
verification checkpoint. No Realm, constructor, Proxy or Promise behavior is
claimed.

The lowering-private two-row `LogicalAssignmentReachability` authority now
derives no capabilities. Its selected-`with` and definite-Reference producers
remain exact, while the conditional-global route and three metadata
projections borrow the domain through exhaustive matches instead of relying on
Copy and equality. A recursive structure guard pins all thirteen source
mentions, both producer contexts and the four semantic decisions. This is a
source-equivalent invariant closure; Reference selection, evaluation order, IR
and inferred metadata are unchanged. The new structure target passes `3/3`,
the focused lowering units pass `2/2`, and the neighboring backend structure
target passes `4/4`. Independent review confirmed the exact branch bodies,
capability closure and narrowly necessary neighboring guard update. The
coordinated `cargo xc`, formatter, diff and repository policy checks are green.

The private-element carrier and its complete eighteen-method backend family now
have one private `objects/private_elements.rs` owner. Thirteen methods retain
their existing `pub(crate)` visibility and five remain owner-private; the
closed five-variant carrier and every projection are exhaustive. The module
boundary audit enforces that ownership and visibility split. A pre/post Wasm
golden capture contains the same 633 fixtures and has an empty recursive diff,
and the focused carrier projection test passes `1/1`. This is an ownership-only
extraction; the broader object emitter and lowering seams remain open.

The complete `InstanceofOperator` / `OrdinaryHasInstance` request family now
has one private `operations/has_instance.rs` owner: three closed private types,
three existing `pub(crate)` entry methods and one private dispatcher moved
without widening visibility or changing their `repr(u64)` order. The focused
ownership assertion, dynamic-RHS guard and CLI fixture each pass `1/1`, and the
same 633-fixture golden baseline remains byte-identical. The complete structure
target remains `4/5` both before and after this move because an older assertion
still expects contract prose that the checked-in contract no longer contains;
that unrelated baseline defect is not repaired in this ownership batch.

Bound-function allocation now has one private
`functions/bound_function_allocation.rs` owner. The closed
`ExactBoundThisSource` domain, the two `pub(crate)` semantic entry points and
the two owner-private allocation methods moved as one 364-line family; call and
construct consumption stays in `functions.rs`. The strengthened inline source
contract enforces the private module, exact two-public/two-private inventory,
sole type/method ownership, exhaustive dispatch and the two reviewed sibling
callers. The moved type, method family and combined source retain SHA-256
`93294a07eececde4c2a0df8c5dc7d447c64eda856551afc4bc72342e44d2de5f`,
`984a49a2c6beb71165f24a9c7713ffacc192b77f07002c62cc5e81a53836db1f`
and `ab51221e93735f18674fa4cf070dc0e044cee53ff599587402af52316081fc0c`
respectively. The focused owner test passes `1/1`, `cargo xc` is green and the
645-artifact pre/post Wasm golden has an empty recursive diff.

The complete Array and TypedArray `FindViaPredicate` compiler family now has
one private `builtins/array/find_via_predicate.rs` owner. The four-way
`FindViaPredicateKind`, its two private direction/projection domains, seven
exhaustive projections, the private non-`Copy` validated-predicate witness, six
owner-private emitters, two existing `pub(crate)` compiler entries and the
focused mapping unit test moved together. The parent re-exports only
`FindViaPredicateKind` for the unchanged standard dispatcher; the shared array
iteration helper and direct-call wrappers remain in `array.rs`. The exact
164-line type/test block, 653-line emitter block and combined source retain
SHA-256
`f731b1fbbf599e2fab5f0077384e73058cab4df6837d0687e066d59bbc8b2d17`,
`e9634ea44a60ea9fc531c28a4fbf11087d4a2b2a226ab6b291b5018ba720ba81`
and `890bde772529510e38867930c2f1c5e9e202740d45bf959afba12f9a80f79a03`
respectively. The 822-line child reduces `builtins/array.rs` from 26,831 to
26,015 lines. The focused mapping test passes `1/1`, and the strengthened
private-module, sole-owner, exact-inventory, retained-parent and eight-dispatch
structure target passes `5/5`. `cargo xc` is green, and exact Array `find`,
Array `findLast` and TypedArray `find` CLI owners each pass `1/1`. The shared
golden checkpoint retains all 646 existing artifacts byte-for-byte; artifact
647 is solely the concurrently added Map/WeakMap fixture. No FindViaPredicate
behavior or conformance improvement is claimed for this ownership-only move.

The private String `slice`/`substring` range coordinator now has one file-backed
`builtins/string/string_code_unit_range.rs` owner. Its four non-`Copy`
`#[must_use]` local carriers, closed two-variant method domain, four private
operations and two existing `pub(super)` entry points moved as the complete
inline-module body; the two `pub(crate)` compiler wrappers, direct-call routing
and Annex B `substr` algorithm remain with their existing owners. The move
changes no instruction or ownership order; rustfmt only compacts one argument
list after removing the old module indentation.
The extraction reduces `builtins/string.rs` from 21,531 to 21,296 lines. The
strengthened private-module, exact-inventory, typed-materializer, dispatcher,
created-Realm, Annex B and saturating-normalizer structure target passes `6/6`.
The file-backed child is 230 formatted lines. `cargo xc` is green and the exact
range, `slice` and Annex B `substr` CLI owners each pass `1/1`. In the shared
647-artifact golden checkpoint, the recursive pre/post diff is empty. This
isolates the extraction as behavior-neutral without claiming String-tree
conformance progress.

The complete Wasm-AOT static-JSON reviver specialization now has one private
`builtins/json/static_reviver.rs` owner. Its private property-key domain and
source projection, eleven private methods and existing `pub(crate)` compiler
entry moved as one 1,005-line family; the two expression callers keep the same
inherent-method boundary. The dynamic frame domains, shared post-call result
owner and create-property helper remain in `builtins/json.rs`, where the
dynamic reviver also consumes them. The combined moved source retains SHA-256
`9a00ab1ffd79e0327fd442a370dfc46767a32744d93030c789db80825aa240d1`;
the formatted 1,011-line child has SHA-256
`8f23ccce84985c2b618daa6db0799161c774f19d79f4103973875f84a4753dab`
and reduces `builtins/json.rs` from 9,325 to 8,319 formatted lines. The
strengthened private-module, exact-inventory, caller and retained-shared-owner
structure target passes `5/5`, and the named static-reviver lowering unit
passes `1/1`. `cargo xc` is green, and the array-getter throw, forward
modification and nonconfigurable-property CLI fixtures pass `3/3`. The shared
647-artifact Wasm golden has an empty recursive pre/post diff. No behavior or
conformance change is claimed for this ownership-only extraction.

The complete validated JSON parse-frame-state local lifecycle now has one
private `builtins/json/parse_frame_state.rs` owner. Its non-`Copy`, must-use
carrier, private local projections, sole validator, borrowed comparator,
consuming frame push and consuming release moved together. Rust requires the
carrier and four methods to be `pub(super)` in signatures used by the retained
parser, but its tuple field remains private. The parent has zero carrier names,
imports or re-exports and keeps inferred validator/comparator/push values. The
exact 12-line carrier, 29-line validator/comparator and 130-line push selections
retain visibility-normalized SHA-256
`afe685b22fd49eb1eecb3d80d08ee8b38eb964abd658ccbeceb32d305eea91ce`,
`77f6bc539c0485f5c8102df5c657f92d251d5b1ea99e3b3f46c9d101945e2015`
and
`d8678affdd90d1504e973bb9ed39ca2881341d3dfd4432e0237c58019d3c88e7`;
their combined 171-line selection retains SHA-256
`c986b658834ada28c28de190fd9be4901d24ffe0e44cd0ffcac006cbd481c62e`.
The former parent release projection and its child-capability replacement have
SHA-256
`3f0cc5616949f020f094a8685ef29d1557506e1934f4f10523c55189f6cbf085`
and
`1ce3bbf805cd7b8634c30787d2ef4a36ea77311238628a03a46cd4961bb6fa29`.
The 184-line child has SHA-256
`c4e485dab8c2adc0b079d97ac9dce090a17b19922af008e2b72f9c690e6a56ab`
and reduces the concurrent parent to 8,307 lines at SHA-256
`67e9262b7f78fdd952f65b6fbd2b2dcbf5e874c1fc3fb7caba91d33a98d22de1`.
The recursive guard pins seven child-only carrier identifiers, sole
construction, two raw projections and the exact `5/9/4/2`
validator/comparator/push/release census. Batch AH uses only scoped source,
formatting, hash, boundary, task and diff checks during implementation. The
recursive frame-state target passes `4/4`, and the unchanged neighboring
reviver target passes `5/5`. At the coordinated checkpoint, shared `cargo xc`
exits `0`, and the exact
`language_numerics::run_wasm_backend_succeeds_for_json_parse_dynamic_reviver_frame_fixture`
CLI witness passes `1/1`. No Test262 cohort or semantic golden was run because
this is a source-equivalent owner move; no behavior or conformance change is
claimed. Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The `Intl` namespace plan and rooted-member witness now have one private
`planning/intl_namespace.rs` owner. The three existing `pub(crate)` types and
three existing `pub(crate)` methods moved as the complete 93-line inline-module
body without widening the private witness fields or re-exporting
`IntlRootsSeeded`. The root list, compile-time constructor-containment proof,
runtime bootstrap plan and its two policy calls remain in `planning.rs`; the
bootstrap emitter still receives only `IntlNamespaceMembers`. After removing
only the old module indentation, the body retains SHA-256
`1e9a3617bded9c0d27a264639519d0e7bf2b20e515789bcd167283e98030837c`
and reduces `planning.rs` from 9,592 to 9,498 lines. A dedicated three-test
source target enforces the private file module, exact inventory and re-export,
retained roots/proof and closed parent/bootstrap call map. That target passes
`3/3`; all six matching Intl unit tests and the exact construction-order CLI
fixture pass, and the pinned DateTimeFormat descriptor leaf passes both
sloppy/strict variants. `cargo xc` is green, and the 647-artifact Wasm golden
has an empty recursive pre/post diff. No behavior or conformance change is
claimed.

The validated legacy-RegExp surrogate-pair carrier now has one private
`lila-ir/src/regexp/legacy_utf16_pair.rs` owner. Its private lead/trail fields
and three existing `pub(super)` operations moved as the complete 25-line
inline-module body; the parsed-term domains, parser admission, nullability and
forward/reverse lowering remain in `regexp.rs`. The parent keeps one constructor
call and two calls to each instruction projection, with forward lowering
emitting the mandatory lead before the quantified trail and reverse lowering
doing the opposite. After removing only the old module indentation, the body
retains SHA-256
`c5780826ab2dc8962c151f2c30b8c21b1b66b85d88120211fb4c917124e72f0b`
and reduces `regexp.rs` from 7,132 to 7,106 lines. A dedicated three-test source
target enforces private ownership, exact visibility/inventory, the closed
caller census and both lowering orders. The structure target passes `3/3`, the
focused direct-non-Unicode IR regression passes `1/1`, and `cargo xc` is green.
The 647-artifact Wasm golden has an empty recursive pre/post diff, including the
legacy-astral fixture. Its existing CLI assertion remains red with
`string(unicode optional full scalar result)`; byte-identical emitted artifacts
show that failure predates and is unaffected by this ownership move. No behavior
or conformance change is claimed.

The complete `Object.prototype.toLocaleString` Invoke family now has one
private `builtins/object/object_to_locale_string_invoke.rs` owner. Its two
private non-`Copy` state carriers, three private transition methods and one
compiler entry moved together; `pub(in crate::builtins)` preserves the entry's
former effective `pub(super)` scope while the unchanged standard dispatcher
remains its sole external caller. After normalizing only the inherent-impl
wrapper, indentation and equivalent visibility spelling, the 155-line family
retains SHA-256
`f7549c1da14cb45693d1b2dc62bd3e5e13b107118040b892f6efa1ab4ff2ed54`.
The formatted 159-line child has SHA-256
`7fb873370ffd54ef6a772776f5e58784afbd3299195529e024d9100ba0ba7c7b`
and reduces `builtins/object.rs` from 9,075 to 8,921 lines. The strengthened
four-test source target enforces private file ownership, exact type/method and
visibility inventory, the closed helper/caller census and the existing GetV,
validation, Proxy-aware call and abrupt-propagation order. Rustfmt and scoped
equivalence/census/diff checks are complete. The structure target passes `4/4`,
the direct Proxy-aware CLI fixture passes `1/1`, and `cargo xc` is green. The
647-artifact Wasm golden has an empty recursive pre/post diff. The focused
Test262 leaves were not rerun in this ownership checkpoint. No behavior or
conformance change is claimed.

The complete own-descriptor predicate policy now has one private
`builtins/object/own_descriptor_predicate.rs` owner. Its capability-free three-
variant domain, sole raw compiler and all three semantic wrappers moved
together; `pub(in crate::builtins)` preserves the wrappers' former effective
`pub(super)` scope for the unchanged standard dispatcher, while the 8,902-line
Object parent cannot name, construct, import or project the raw policy. The
moved five-line domain and 191-line raw compiler retain SHA-256
`36ed9747dec1c589dd32f763a7bc907fc84d3070988bd0fef7641b08e6138098`
and
`05f279033ab151a2c156cdb76ca0da20ad330d953c9ccae8ea055b9d9fbce4a1`.
The resulting 230-line child has SHA-256
`f4db50dd3eb3ba382999dec0dfd9fc578253de1328bbaf41c2a48a0b73b827ba`,
and the reduced parent has SHA-256
`b81dc402514e20731f7b808fe652e8b24a4c0f429e439adba96213491d968754`.
The recursive guard and module policy pin 14 child policy mentions, five uses
of each variant, the raw definition plus three private calls, zero parent raw
names and the three builtins-visible semantic wrappers and unchanged standard
calls. The raw owner move is byte-equivalent; only equivalent wrapper visibility
spelling changes. Batch AK shared `cargo xc` is green, the focused structure
target passes `4/4`, and the exact CLI fixture passes `1/1`; no Test262 cohort
or semantic golden is required for this owner move. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.
The bounded contract is
[`object-own-descriptor-predicate-kind.md`](../docs/rust-rewrite/contracts/object-own-descriptor-predicate-kind.md).

The complete prototype-accessor lookup policy now has one private
`builtins/object/prototype_lookup.rs` owner. Its capability-free
`PrototypeLookup::{Getter, Setter}` domain and sole raw compiler moved together;
the standard dispatcher sees only two fixed `pub(in crate::builtins)` semantic
wrappers, while the Object parent retains only the private module declaration.
After normalizing only the former `pub(super)` visibility, the four-line domain
and 132-line emitter retain SHA-256
`7ca467738a2dfd39524325c1fac34084715cbb79a46baf9a862dd54737778a57`
and `f6ba5e5158701597301fc843f203a2e3997665afc54bffefd908fbe9d866876f`.
The 155-line child has SHA-256
`bf4ec5630203d7a10b6982ac101dcef9192f693f1e819e4c0e2bb79f0f06c2ec`,
and the reduced 8,765-line parent has SHA-256
`82437af076110c4151c1a82943c93054d89c24ef0caf19b217ad946d77c0fa`.
The exact parent/child/dispatcher guard and module policy pin six child policy
mentions, two uses of each qualified variant, the raw definition plus two
private calls, two narrow semantic wrappers and zero parent/dispatcher raw type
constructions, imports or emitter calls. The raw move is source-equivalent; the
frozen dispatcher selection changed only from raw variants to their
corresponding fixed wrappers. At the Batch AL checkpoint, `cargo xc` is green,
the structure target passes `4/4`, and the two exact CLI witnesses pass `2/2`.
No Test262 leaf or semantic golden was required or run. The bounded contract is
[`object-builtin-policy-domains.md`](../docs/rust-rewrite/contracts/object-builtin-policy-domains.md).
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The complete Object integrity-test policy now has one private
`builtins/object/integrity_test.rs` owner. Its capability-free
`IntegrityTest::{Sealed, Frozen}` domain and sole raw compiler moved together;
the standard dispatcher sees only two fixed `pub(in crate::builtins)` semantic
wrappers, while the Object parent retains only the private module declaration.
After normalizing only the former `pub(super)` visibility, the four-line domain
and 198-line emitter retain SHA-256
`0f81ace14c7caea6494f3c6ac21f2b0bba61ba10bb10637eb79e10a22b0f2d64`
and `7263b51a1dcfdcd4eb0bc1a1bcb6569516652347ec3b8bfe773c187bddb7bf79`.
The 221-line child has SHA-256
`ad029d42fc1fdeb65ae03ac765c7186a7bd7efa8cbe7da51e932f9733ad53d93`,
and the reduced 8,562-line parent has SHA-256
`67232c7c756062fa9eb24d83506a750e147744b087464ce77deccd4243b27cee`.
The exact parent/two-child/dispatcher guard and module policy pin six child
policy mentions, two uses of each qualified variant, the raw definition plus
two private calls, two narrow semantic wrappers and zero parent/dispatcher raw
type constructions, imports or emitter calls. The raw move is source-equivalent;
the frozen dispatcher selection changed only from raw variants to their
corresponding fixed wrappers. At the Batch AM checkpoint, `cargo xc` is green,
the structure target passes `4/4`, and the exact policy-domain CLI witness
passes `1/1`. No Test262 leaf or semantic golden was required or run. The
bounded contract remains
[`object-builtin-policy-domains.md`](../docs/rust-rewrite/contracts/object-builtin-policy-domains.md).
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The complete enumerable-own-properties policy now has one private
`builtins/object/enumerable_own_properties.rs` owner. Its capability-free
`EnumerableOwnProperties::{Entries, Values}` domain and sole raw compiler moved
together; the standard dispatcher sees only two fixed
`pub(in crate::builtins)` semantic wrappers, while the Object parent retains
only the private module declaration. After normalizing only the former
`pub(super)` visibility, the four-line domain and 309-line emitter retain
SHA-256 `791b3ae06c58f2ed8ca870d44a823882e4a7a3262c0eb528d323169116c54dc4`
and `2feb0f6ab4e8fa5c68e75311a45f637fcebc811ad26aad38bd2abb7b5db7ce06`.
The 338-line child has SHA-256
`8d47fee7765fbcb0691be6b4f1df1de876e662db8552b29f8599a9a2a37d7777`,
and the reduced 8,248-line parent has SHA-256
`3229401d4da5d26395572f184167c246442d67b2c7121d79adce385b33c7b3b1`.
The exact parent/three-child/dispatcher guard and module policy pin eight child
policy mentions, three uses of each qualified variant, the raw definition plus
two private calls, two narrow semantic wrappers and zero parent/dispatcher raw
type constructions, imports or emitter calls. The raw move is source-equivalent;
the frozen ten-line dispatcher selection changed only from raw variants to
their corresponding fixed wrappers, whose formatted two-line selection has
SHA-256 `531e8ccf0c457ec36d0ed5273dd9eb0832b81a4033c966871092987337da39dc`.
At the Batch AN checkpoint, `cargo xc` is green, the policy and Realm structure
targets pass `4/4` and `1/1`, and the exact policy-domain CLI witness passes
`1/1`. No Test262 leaf or semantic golden was required or run. The bounded
contract remains
[`object-builtin-policy-domains.md`](../docs/rust-rewrite/contracts/object-builtin-policy-domains.md).
Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green.

The complete `Object.defineProperty` descriptor family now has one private
`builtins/object/define_property.rs` owner. Its two descriptor carriers,
Arguments-index and `callee` consumers, and sole builtin compiler moved
together; the Object parent retains only the private module declaration and the
standard dispatcher retains one fixed builtin call. The resulting 2,500-line
child has SHA-256
`01ea9de92ace5f710bc6fcea6b7b4d64326e8d726e165920a06fdb1d8368b4c6`,
and the reduced 5,751-line parent has SHA-256
`d8e910a3b8e2edcd7ab4e9fd6ee19507d86de6fbd6721d1da29655ffef817a53`.
Batch AO dry review pins private sole ownership and exact entry visibility.
`cargo xc` is green; the strengthened owner target and five descriptor/proxy
neighbors pass `25/25`, and the exact object-descriptor CLI passes `1/1`.
This source-equivalent move makes no behavior or conformance claim.

The complete `Object.getOwnPropertyDescriptor` compiler now has one private
`builtins/object/get_own_property_descriptor.rs` owner. Its 1,431-line
ordinary, Array, Arguments, TypedArray, Function and Proxy implementation moved
together; the Object parent retains only the private module declaration and the
standard dispatcher retains one fixed builtin call. The exact ownership
contract is
[`object-get-own-property-descriptor-owner.md`](../docs/rust-rewrite/contracts/object-get-own-property-descriptor-owner.md).
Batch AP's owner, Arguments neighbor, Array neighbor and Proxy ownership
structure targets pass `4/4`, `4/4`, `3/3` and `4/4`; the exact
object-descriptor CLI passes `1/1`; and `cargo xc` is green. This is a
source-equivalent move with no new descriptor behavior or conformance claim.

The complete `Object.getOwnPropertyDescriptors` compiler now has one private
`builtins/object/get_own_property_descriptors.rs` owner. Its 182-line coercion,
own-key enumeration, per-key descriptor lookup, Realm allocation, key
conversion and result-definition family moved together; the Object parent
retains only the private module declaration and the standard dispatcher keeps
one fixed call. Normalizing the child entry visibility reproduces the frozen
source SHA-256 exactly. The focused contract is
[`object-get-own-property-descriptors-owner.md`](../docs/rust-rewrite/contracts/object-get-own-property-descriptors-owner.md).
At the 2026-08-28 Batch AQ checkpoint, `cargo xc` is green, the private-owner
structure target passes `4/4`, and the exact
`built-ins/Object/getOwnPropertyDescriptors/normal-object.js` leaf passes both
sloppy and strict Wasm-AOT executions (`2/2`) with every failure bucket at
zero. This source-equivalent move claims no new Object behavior, broader
Test262 result or published conformance-count change.

The complete `Object.assign` compiler now has one private
`builtins/object/assign.rs` owner. Its 262-line target/source coercion, own-key
enumeration, descriptor observation, enumerable filtering, source `Get`, target
`Set`, abrupt-completion and local-release family moved together. Normalizing
only the child entry visibility reproduces the frozen source SHA-256
`65680b329345d9833065718b97a828484f464208d19bc5ff09d7f6ad3a46f6cd`.
The focused contract is
[`object-assign-owner.md`](../docs/rust-rewrite/contracts/object-assign-owner.md).
At the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the owner structure
target passes `4/4`, and the exact `Target-Object.js` Wasm-AOT leaf passes `2/2`
with every failure bucket at zero. This source-equivalent move claims no new
Object behavior, broader Object conformance or published conformance-count
change.

The complete Super Property Reference mutation lifecycle now has one private
`expressions/super_property_mutation.rs` owner. Its two private non-`Copy`
carriers and four-method evaluate/GetValue/PutValue/operation family moved
together; the entry's new `pub(super)` spelling preserves its former effective
scope for the two unchanged exhaustive dispatch paths in `expressions.rs`.
After normalizing only the inherent-impl wrapper, indentation and equivalent
entry visibility, the 278-line family retains SHA-256
`2e5d7bb5520246a3884a331c52810a5020d9eb3a5d3d7f694e17557b3f698a82`.
The formatted 282-line child has SHA-256
`502e52afa38ef71455dbdd3626c2a2b523f984a36bab06e1e5000948d1d02109`
and reduces `expressions.rs` from 4,619 to 4,342 lines. The strengthened
six-test lifecycle target enforces private ownership, exact visibility and
type/method inventory, the closed helper/caller census, one key coercion,
retained base/receiver/key state, exhaustive mutation arms and post-PutValue
result publication. Two sibling structure bounds now end at the next retained
parent method; verification also corrected their stale references to the
previously extracted assignment-lowering owner. The lifecycle, numeric-update
and eager-compound structure targets pass `6/6`, `7/7` and `7/7`; the focused
IR invariant and direct CLI fixture each pass `1/1`. `cargo xc` is green, and
the 647-artifact Wasm golden has an empty recursive pre/post diff. The exact
Test262 cohort was not rerun in this ownership checkpoint. No behavior or
conformance change is claimed.

The closed main-body completion-exit state machine now has one private
`emit/completion_exit.rs` owner. Its crate-visible wrapper, private three-state
representation and five projections/transitions moved together; the parent
re-exports only `CompletionExit`. Three methods use `pub(super)` to preserve
their former `emit`-private scope, while the ABI and active-checkpoint
projections retain `pub(crate)` for their unchanged parent and control-flow
callers. After normalizing only those equivalent visibility spellings, the
54-line family retains SHA-256
`88c1174b802878600758c639c2a63645d2868f1f5a13c5e5e04c4a24dfd7221b`.
The formatted 56-line child has SHA-256
`c2532e5123ae85745216d6586eb0efc485e7c3093b97685e76e5cf68b045b887`
and reduces `emit.rs` from 5,333 to 5,281 lines. A dedicated three-test source
target enforces the private owner and narrow re-export, exact type/method and
visibility inventory, closed caller census, exhaustive state projections,
checked enter/leave transitions, main-body block order and abrupt-completion
routing. The structure target passes `3/3`; the focused engine checkpoint test
and both normal/primary-throw unhandled-rejection CLI fixtures each pass `1/1`.
`cargo xc` is green, and the 647-artifact Wasm golden has an empty recursive
pre/post diff. No behavior or conformance change is claimed.

That stored completion-exit authority now derives no capabilities. Its ABI,
active-target, checkpoint-entry and checkpoint-exit decisions all borrow the
one state through exhaustive matches, while the active `ControlTarget` alone
is copied into the unchanged branch API. The recursive five-wrapper/eighteen-
state mention census, exact four producer transitions and checkpoint branch
order are pinned by the strengthened existing structure target. This is a
source-equivalent compiler invariant: no Wasm instruction, temporary local,
branch target or JavaScript behavior changes. The structure target passes
`3/3`, the exact engine checkpoint witness and both exact rejection-order CLI
witnesses each pass `1/1`, and the scoped formatter is clean. No broad suite or
Wasm golden was run for this closure. Independent dry re-review is clean, and
the following shared workspace compile, formatter, module-boundary, task-plan
and diff gates all pass.

The complete Date string-parser compiler family now has one private
`builtins/date/date_string_parse.rs` owner. Its two private ISO byte/decimal
operations and two existing `pub(crate)` parse methods moved as one contiguous
612-line family; the unchanged standard dispatcher remains the only external
owner, with one `Date.parse` call and one one-argument Date-constructor call.
The exact moved block retains SHA-256
`8e8b962beb2070165e2377bfd0fdf057ac9b02b29945a2192c5555aaa422ea73`;
the formatted 616-line child has SHA-256
`a48eb3f67b2ea9459654b2522bba162ccbadff4ea97e05fdd8f739b3d71d1c6c`
and reduces `builtins/date.rs` from 2,623 to 2,012 lines. A dedicated
three-test source target pins private ownership, exact visibility and method
inventory, the closed internal/external call maps, both standard routes, the
Date parse fixture domain and its golden-corpus inclusion. Rustfmt, exact
source-equivalence, caller-census and scoped-diff checks are complete. The
structure target passes `3/3`; the focused `Date.parse` and Date-copy CLI
fixtures each pass `1/1`, and `cargo xc` is green. The 647-artifact Wasm golden
has an empty recursive pre/post diff; no behavior or conformance change is
claimed.

The complete HTMLDDA host compiler pair now has one private
`builtins/host/html_dda.rs` owner. The `$262.IsHTMLDDA` creation body and its
internal callable body retain their two existing `pub(crate)` entries; the
already-dirty `emit.rs` remains their sole external caller through the adjacent
`CreateHTMLDDA` and `HTMLDDA` exhaustive arms and was not edited by this move.
The exact contiguous 51-line family retains SHA-256
`3b1a845e900473644279eced312a455c50e06b5247436d01b2ca9aae8fa4efd0`;
the formatted 55-line child has SHA-256
`0c48e40060e48f9033444c994b707ca21a113a28e0270198235e234d950af19b`
and reduces `builtins/host.rs` from 8,194 to 8,144 lines. A dedicated
three-test source target pins private ownership, exact method visibility, both
dispatch callers, the extensible-function allocation invariant, the null
callable result, the direct CLI fixture domain and golden-corpus inclusion.
Rustfmt, source-equivalence, caller-census and scoped-diff checks are complete.
The structure target passes `3/3`, the focused HTMLDDA CLI fixture passes `1/1`,
and `cargo xc` is green. The 647-artifact Wasm golden has an empty recursive
pre/post diff. The direct Test262 leaves remain deferred; no behavior or
conformance change is claimed.

The complete created-Realm `WeakRef` materialize-to-publish lifecycle now has
one private `builtins/host/created_realm_weak_ref_intrinsics.rs` owner. Its
non-`Copy`, must-use carrier, sole materializer and consuming publisher moved
together; Rust requires the carrier and methods to be `pub(super)`, but both
fields remain child-private and the host parent cannot construct, project,
import, re-export or explicitly name the carrier. The inferred host producer
and consumer calls remain byte-identical. The exact pre-move five-line carrier
and 153-line method selection retain SHA-256
`50b98c378f3a260b73ab69e22538e856b957c5203c470489baab4e0677568244`
and
`10a82a763c5b87ef5e28dbd72e26d62d1873675864462e7622b3dd06cbff7a68`;
their combined 158-line hash is
`24d820fe3c2b14085b1f3aa3373537fb1def4ef211ef410025aea1f68300f119`,
and the visibility-normalized selection has SHA-256
`9d17048c6ff4a8f4cacfd97b8c6ac0edc40c5039fe18f563843f931fe403e479`.
The resulting 8,941-line parent and 163-line child have SHA-256
`6ffaf8361a886420f7ee766a66154f6fc42bf9c5704cac6a2fc7e9e64e218b3a`
and
`5d460ca0be7e9eef7f81cee28ca258ac1fc9b4b6b655523651f4b64d4caea049`.
The unchanged 12-line caller pair retains SHA-256
`41419c46072b0e4ae037b6217d5312768a77c9ff4f6c780b55381be0253a96eb`.
The recursive guard and module policy pin five child carrier mentions, sole
construction and destructure, one producer/consumer definition and parent call
each, private fields, and consuming release of both retained locals. The
retargeted source target passes `4/4`. At the Batch AF shared checkpoint,
`cargo xc` is green, the exact created-Realm WeakRef CLI fixture passes `1/1`,
and the six exact pinned `proto-from-ctor-realm`,
`newtarget-prototype-is-not-object`, prototype `constructor`, prototype
`prop-desc`, deref `this-not-object-throws` and deref `custom-this` leaves pass
all `12/12` sloppy/strict Wasm-AOT executions with every failure bucket at
zero. Batch AF did not rerun the semantic golden. The bounded contract remains
[`weak-ref-created-realm-publication.md`](../docs/rust-rewrite/contracts/weak-ref-created-realm-publication.md).

### Landed 2026-08-23: static-JSON parse lowering ownership

This ownership seam is deliberately narrower than the complete `JSON.parse`
lowering family. `lila-ir/src/lowering/static_json_parse.rs` owns the ordered
static-reviver protocol, its prepared parsed-value proof and the private
`JsonStaticParser`. `prepare_static_json_parse_reviver` snapshots and parses a
proven static input before argument lowering; `finish_static_json_parse_reviver`
consumes that proof only after the lowered reviver has callable-kind and known
target evidence. The specialized IR explicitly carries the callee, input and
reviver runtime operands, and the emitter evaluates and propagates throws from
them in that order before materializing the prepared parse value. This keeps
callee acquisition, argument side effects and `JSON.parse` input conversion in
ECMAScript order.

The two protocol methods and `PreparedStaticJsonParseReviver` cross the private
child boundary as `pub(super)` for one property-call site in
`lowering/call_expression.rs` and one value-call site in
`lowering/call_expression/non_property_call.rs`; the proof's parsed value and
every parser operation remain private. The parent retains
`known_json_parse_reviver_targets` and `observe_json_parse_reviver_targets`:
the ordinary dynamic `JSON.parse` path also consumes that target/signature
analysis, so moving or copying it would make the static filename a false
owner. `JsonStaticValueIr`, `TypedExpr`, `ExprIr`, `ValueKind`, `KindSet`,
builtin identity and every flow-fact field remain with their existing owners.
No crate-public Rust API is widened; the child surface is limited to the two
protocol methods and their unforgeable prepared value.

The protocol admits only the exact `JSON.parse` identity, exactly two
non-spread arguments, a literal or initialized ordinary-binding static String
outside any `with` environment, a complete static parse, a callable reviver
kind and a known nonempty reviver target set. Mutable captured or repeatedly
evaluated loop bindings remain on the dynamic path, and leaving a lexical
scope removes only the static fact keyed by that binding's storage identity.
The parser must retain full input consumption; source lexemes and parsed `f64` bits;
rejection of leading zeroes and incomplete
fraction/exponent forms; `serde_json` string decoding and raw control-byte
rejection; recursive array/object behavior and trailing-comma rejection; and
duplicate-object-key replacement without changing the first insertion
position.

The boundary audit requires the exact private module declaration, the exact
thirteen-function inventory, the two protocol methods and prepared proof as the
only three Rust-visible child items, both ordered sibling call sites, sole
ownership of the protocol/proof/parser family, both shared target helpers in
the parent, zero parent copies, no copied shared type, no local macro/generated
helper or legacy `include!`/`#[path]` assembly, executable
input-to-parser-to-target wiring, explicit callee and input operands, and
measured parent/child line budgets. The focused
`json_parse_snapshots_its_input_before_reviver_effects` regression pins the
ordering requirement by mutating the source binding from the invoked reviver
body and requiring the prepared parse to keep the earlier String.

At clean parent `9a3ac9ad5`, the capped pre-move Wasm golden passes `2/2` in
309.29 seconds; the post-move capture passes `2/2` in 314.61 seconds. Both
record 633 fixtures in 635 artifacts and their recursive diff is empty. The
236 moved source lines have the same normalized SHA-256 before and after the
move, `edb11f2be0d0376cd042d5541572e08a236ac569a7a81575b322630ae027cd06`.
Formatting, the all-target `lila-ir` check, `cargo xc`, the strengthened module
boundary audit and both independent reviews pass. The moved static IR witness
passes `1/1`, both engine witnesses pass `2/2`, and the three static plus one
dynamic-control CLI witnesses pass `4/4`. The retained
`dynamic_json_parse_observes_reviver_holder_kinds` IR control fails identically
at the clean parent and moved tree, so the combined `json_parse` IR filter is
unchanged at `1/2`. This is architecture and byte-identical no-regression
evidence, not a JSON behavior change, T20 work, broad Test262 coverage or JSON
conformance progress.

### Landed 2026-08-23: throw-value inference ownership

`lila-ir/src/lowering/throw_inference.rs` now owns the complete recursive
throw-value inference closure: `merge_optional_value_info`, block and statement
walkers, the expression wrapper and operand walker, and property-key inference.
The family is one contiguous 891-line parent slice containing exactly six
methods. Its exact move reduces `lowering.rs` from 21,877 to 20,986 raw lines
and produces an 895-line child. Only `infer_block_throw_info` crosses the
private child boundary as `pub(super)` for the sole external caller in
`lowering/try_statement.rs`; the other five methods remain private. No IR type,
public Rust API or compiler behavior moves with it.

The parent retains `merge_value_infos`, `resolve_single_function_target`,
`object_like_kind_set` and the free `unknown_runtime_value_info` helper;
`builtin_shapes.rs` retains `standard_error_instance_info`, while
`reference.rs` retains the exhaustive `carried_put_value_failure` classifier.
Every `StatementIr`, `ExprIr`, `PropertyKeyIr`, `ValueInfo`,
`PutValueFailure`, `Strictness` and builtin identity type remains with its
current owner. The child imports those owners through `super::*` rather than
copying a helper or widening a type.

The move preserves every inference and merge ordering. Block, statement,
resource, argument and case lists stay left-to-right; ordinary `For` remains
initializer, test, update, body; `GeneratorLoop` remains initializer, test,
update, before-suspension, suspension, after-suspension; and try inference
remains try, catch, finally. `StatementIr::Throw` first infers failures in its
operand and only then merges the thrown value. `infer_expr_throw_info` remains
the recursive wrapper: it exhaustively converts the carried strict
`PutValueFailure` and merges it before operand failures, then delegates exactly
once to `infer_expr_operand_throw_info`; recursive operand and property-key
walks route back through that wrapper so nested strict writes contribute. The
exhaustive `StatementIr`, both `ForInitIr`, `ExprIr`, `PropertyKeyIr`,
`PutValueFailure`, super-mutation, object-property, optional-chain,
class-element and suspended-reference matches retain zero wildcard or
bare-binding catch-all arms. Adding a variant to one of those closed domains is
a compile error until its throw semantics are chosen; non-exhaustive `if let`
filters such as the generator resume-mode specialization remain explicit
nonclaims. `RuntimeThrow` continues through the total native error constructor
mapping, while dynamic disposal, spread, property-hook and call contributions
and rejection-only `import()` handling remain exact.

The module audit requires the exact private child declaration, all six sole
method owners, exactly one Rust-visible child item, the sole external caller in
`try_statement.rs`, zero parent copies, zero copied shared helper or type
declarations, zero catch-all arms or `unreachable!`, exactly one carried-failure
read and one wrapper-to-operand delegation, no local macro/generated helper or
legacy `include!`/`#[path]` assembly, and measured parent/child line budgets.
Sixteen negative controls reject a missing child, public module, reintroduced
parent owner, widened helper, modifier-qualified extra method, copied shared
helper or type, alternate external caller, local item macro or trait, forged
string witness, four catch-all spellings and missing recursive delegation.

Two existing source-structure witnesses now follow the owner rather than keep
searching `lowering.rs`: the eager compound-assignment throw arm and the logical
assignment throw arm. Their exact assertions, the IR
`contribute_arbitrary_catch_values` filter, the private-`in` and unbound-global
inference witnesses, the strict/sloppy top-level-try CLI pair and the dynamic
global ReferenceError supplement are the focused behavioral contract. No new
permanent behavior test is justified solely by this exact ownership move, and
the two broader structure targets retain unrelated stale `lower_assign`
source bounds outside this batch.

At clean parent `a77f923b3`, the capped pre-move Wasm golden passes `2/2` in
312.13 seconds; the post-move capture passes `2/2` in 327.32 seconds. Both
record 633 fixtures in 635 artifacts and their recursive diff is empty. Exact
source-equivalence and independent semantic and policy reviews pass. The
all-target `lila-ir` check and `cargo xc` are green. Serial IR witnesses pass
`2/2`, `1/1` and `1/1`; the two exact source-structure witnesses each pass
`1/1`; and the strict, sloppy and dynamic-global CLI witnesses each pass
`1/1`. The module-boundary audit passes after all sixteen negative controls.
No broad Test262, full-workspace behavior or throw-conformance improvement is
claimed by this ownership-only extraction.

### Landed 2026-08-23: for-in lowering ownership

`lila-ir/src/lowering/for_in.rs` now owns the complete for-in lowering family:
the sole statement-facing lowerer and its twelve owner-only helpers for Annex B
initializer prefixes, closed initializer-name recovery, known-empty and
non-enumerable Test262 guards, pattern/property body prefixes, target
classification and final statement construction. The moved family was 564
source lines across four parent blocks; removing their separator lines reduced
`lowering.rs` from 22,444 to 21,877 raw lines and produced a
571-line child. Only `lower_for_in_loop` crosses the private child boundary
as `pub(super)`; all twelve helpers remain private. No IR type, public Rust API
or compiler behavior moved with it.

The statement dispatcher remains the only external caller. The parent retains
the shared for-in/of environment and TDZ lowering, loop-body and declarator
lowering, global-target recognition, single-statement and identifier matching,
static-string analysis, async-entry-state query, expression/pattern/property
lowering, scope/allocation/flow helpers, and the shared `contains`,
`supported_bound_names` and `for_in_loop_binding_storage_name` free functions.
`lowering/for_of.rs` continues to consume the shared environment helper, while
`lowering/call_expression.rs` continues to consume `for_in_global_target`; the
extraction copies neither owner and does not widen their visibility.

The byte-exact move preserves the source order of every observable decision:
refusing an awaiting body before lowering the head; retaining the Annex B
initializer prefix on every supported early exit; evaluating a nullish head
for effects; pattern-head TDZ scope push/pop; inferred-undefined parameter
widening; dynamic/array/string/object/nullish/primitive target classification;
access and pattern assignment prefixes; per-iteration lexical-environment
storage; body scope teardown before var/global flow joins; and the final
dynamic, array, string and object statement choice. The existing known-empty
and non-enumerable shortcuts, diagnostics and specialization rules are copied
exactly, not cleaned up or expanded in this batch.

The module audit requires the exact private child declaration, all thirteen
sole method owners, exactly one Rust-visible child item, zero parent copies,
zero copied shared-helper definitions, no legacy `include!` assembly and
measured parent/child line budgets. Negative controls reject a missing child,
widened module or helper, copied shared or generic helper, copied type and an
extra modifier-qualified function. Existing focused IR and CLI behavior
witnesses remain the semantic contract; no new permanent test is justified
solely by the filename move.

At clean parent `fcb0a924b`, the fresh capped pre-move Wasm golden passes `2/2`,
records 633 fixtures in 635 artifacts and is retained under
`target/golden/for-in-before-fcb0a924b`. The capped post-move golden also passes
`2/2` over the same 633 fixtures and 635 artifacts, and the recursive artifact
diff is empty. `cargo check -p lila-ir --all-targets` and `cargo xc` pass; the
focused IR witnesses pass `8/8`, `2/2`, `1/1` and `1/1`. The CLI `for_in_`
filter is unchanged at `4/7` on both the clean parent and moved tree: array
DefineProperty order, simple-object order and prototype order remain
pre-existing failures. Supplemental exact CLI witnesses are likewise unchanged
at `2/3`, with the object-keys primitive witness the pre-existing failure. This
is architecture and no-regression evidence, not a for-in behavior or
conformance improvement, broad Test262 run or full-workspace claim.

### Landed 2026-08-23: for-of lowering ownership

`lila-ir/src/lowering/for_of.rs` now owns the complete for-of lowering family:
the async array-index resumable specialization, the sole statement-facing
wrapper, the exhaustive head lowering, and the two lowering-only carriers
`AsyncForOfArrayWalkForm` and `ForOfLoweringIr`. The extraction moves one
coherent 1,026-line source family out of `lowering.rs`, `lowering_helpers.rs`
and `ir.rs`. Only `lower_for_of_loop` crosses the child boundary as
`pub(super)`; the other methods, both carriers and their
constructors/conversions are private to the child.

Moving `ForOfLoweringIr` deliberately removes an accidental public Rust API
created by `pub use ir::*`. It has no workspace consumer outside this family,
and its privacy is the invariant: every path out of `lower_for_of_head` must
construct a protocol witness, while no unrelated code may construct, retain or
discard that lowering-only proof. This is an intentional pre-1.0 API narrowing,
not a claim that the patch is only a filename change.

The statement dispatcher remains the sole external caller. Shared loop and
environment helpers stay in the parent, including `plain_async_entry_state`,
`split_resumable_loop_body`, `lower_loop_body`,
`lower_for_in_of_environment` and `lower_for_head_expression_with_tdz`.
`lowering/async_disposable.rs` retains `LoweredForOfHeadKind` plus admission,
pending-head, finalization and statement construction. Public statement/head,
environment, iterator-plan and protocol-witness IR remain in their existing
IR and iterator-obligation owners. The extraction copies none of those helpers
and does not widen their visibility.

The move preserves source order and behavior: scope push/pop on every bailout,
flow-fact snapshots and joins, iterable-before-body evaluation, synchronous-
using TDZ/disposal ordering, async-disposable pending-head sequencing,
suspension-state acquisition, five load-bearing async iterator slot
allocations, the closed array/string/generic/resumable protocol outcomes, and
`AsyncForOfArrayWalkForm`'s non-array / target-shape / captured-iteration /
captured-TDZ classification priority. No specialization cleanup, diagnostic
change, intactness repair or conformance expansion belongs in this batch.

The extraction reduces `lowering.rs` from 23,208 to 22,444 raw lines; the child
is 1,036 lines. The module audit requires the one child owner, all three sole
methods, both private carriers, zero parent/helper/IR copies, no shared-helper
copies, no legacy `include!` assembly and separate parent/child budgets. Its
missing-child, public-module, public-carrier, copied-shared-helper and generic-
helper negative controls all fail correctly. The two source-bounded for-of
backend tests read the new child directly.

The capped pre/post Wasm goldens both pass `2/2`, record 633 fixtures in 635
artifacts and have an empty recursive diff; the post capture finished in
388.51 seconds. The focused IR witnesses pass `3/3`, `3/3` and `1/1`; the three
backend structure targets pass `5/5`, `5/5` and `3/3`; and four exact CLI
witnesses pass `4/4`. `cargo check -p lila-ir --all-targets`, `cargo xc`,
formatting, diff, module-boundary and task-plan checks are green. No broad
Test262 filter or full workspace test was used as evidence. No for-of behavior
or conformance improvement is claimed.

### Landed 2026-08-23: with-statement ownership

`lila-ir/src/lowering/with_statement.rs` now owns the complete Object
Environment lifecycle for `with`: outer-environment object evaluation,
analyzed hidden-binding materialization, resumable/capture refusal, ordered
environment-chain entry and exit, body lowering and nested lexical-block IR
assembly. The statement dispatcher remains its sole caller; allocation,
suspension and reference-lifecycle helpers remain in their shared owners.

This is an exact source move. All 99 method lines compare exactly after
normalizing only `fn lower_with_statement` to private-module visibility. The
extraction reduces `lowering.rs` from 23,307 to 23,208 raw lines, and the child
is 103 lines. The module audit requires the sole owner, rejects copied shared
helpers and lifecycle types, requires exactly one chain entry and exit, forbids
legacy `include!` assembly, budgets parent and child separately, and fails both
missing-child and missing-exit negative controls. No persistent structural test
used the old method as a source-slice sentinel.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Seven targeted IR environment, definition-
cursor, hidden-capture, suspension and fallback witnesses pass `7/7`. The five
targeted CLI fixtures pass `2/5`; one combined run at untouched parent
`870203481` reproduces the other three failures with identical diagnostics:
two existing `instanceof`-callability errors and one Wasmtime function-size
limit. A deliberately broader `with_` IR filter passed `66/67`; its unrelated
Object.seal result-shape assertion remains red and was not used as evidence for
this owner. The all-target `lila-ir` and workspace checks are green. No `with`
behavior or conformance improvement is claimed.

### Landed 2026-08-23: break/continue ownership

`lila-ir/src/lowering/break_continue.rs` now owns labelled and unlabelled
break/continue target validation and final abrupt-control IR assembly. The
statement dispatcher remains its sole caller. The parent-owned active-label
stack and `LabelTargetKind` stay shared with labelled-statement lowering, while
the breakable and loop depths remain lowerer state.

This is an exact source move. All 41 method lines compare exactly after
normalizing only `fn lower_break` and `fn lower_continue` to private-module
visibility. The extraction reduces `lowering.rs` from 23,348 to 23,307 raw
lines, and the child is 45 lines. The module audit requires both sole owners,
rejects copies of the shared active-label types, forbids legacy `include!`
assembly, budgets parent and child separately, and fails its missing-child
negative control. No persistent structural test used either old method as a
source-slice sentinel.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Three focused IR loop, switch-label and
direct-label-target witnesses pass `3/3`; five focused CLI inspect, finally,
iterator-closing and using-loop lifecycle witnesses pass `5/5`. The all-target
`lila-ir` and workspace checks are green. No break/continue behavior or
conformance improvement is claimed.

### Landed 2026-08-23: switch-statement ownership

`lila-ir/src/lowering/switch_statement.rs` now owns discriminant and selector
evaluation, the one shared CaseBlock lexical environment, hoisted and Annex B
function selection, case-body lowering, flow-fact joins, breakable-depth
lifecycle and final `Switch` IR assembly. The statement dispatcher remains its
sole caller; reusable statement-list, function, environment and flow helpers
remain parent-owned.

This is an exact source move. All 86 method lines compare exactly after
normalizing only `fn lower_switch` to private-module visibility. The extraction
reduces `lowering.rs` from 23,434 to 23,348 raw lines, and the child is 90
lines. The module audit requires the sole owner, rejects copied shared
statement-list and environment-materialization helpers, forbids legacy
`include!` assembly, budgets parent and child separately, and fails its
missing-child negative control. Two for-of structural tests now end their
source slice at the adjacent `lower_for_init` owner instead of depending on the
old location of `lower_switch`.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Six focused IR CaseBlock, TDZ, Annex B,
capture and label witnesses pass `6/6`; two focused CLI inspect and throwing
property-read witnesses pass `2/2`; the two affected for-of structural targets
pass `5/5` each. The all-target `lila-ir` and workspace checks are green. No
switch behavior or conformance improvement is claimed.

### Landed 2026-08-23: labelled-statement ownership

`lila-ir/src/lowering/labelled_statement.rs` now owns nested-label collection,
direct labelled-function routing, loop-versus-breakable target classification,
active-label stack installation/removal and final `Labelled` IR assembly. The
statement dispatcher remains its sole caller; the shared `ActiveLabel` and
`LabelTargetKind` types remain parent-owned for break/continue lowering.

This is an exact source move. All 68 method/helper lines compare exactly after
normalizing only `fn lower_labelled` to private-module visibility. The
extraction reduces `lowering.rs` from 23,502 to 23,434 raw lines, and the child
is 72 lines. The module audit requires the sole owner and both private helpers,
rejects copies of the shared label types, forbids legacy `include!` assembly,
budgets parent and child separately, and fails its negative control when the
child is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Five focused IR label/target/lifecycle filters
pass `5/5`; three focused CLI inspect, iterator-closing and await-using filters
pass `3/3`. The all-target `lila-ir` and workspace checks are green. No labelled
statement behavior or conformance improvement is claimed.

### Landed 2026-08-23: while-family ownership

`lila-ir/src/lowering/while_loop.rs` now owns ordinary and resumable `while`
lowering plus the explicit `do while` suspension refusal. Condition/body order,
loop flow-fact joins, async/generator resume-state selection and the final
`While`, `DoWhile` or `GeneratorLoop` choice move together. The statement
dispatcher remains their sole caller; shared loop-resumption helpers remain in
the parent.

This is an exact source move. All 99 source lines compare exactly after
normalizing the two private-module visibility tokens and rustfmt's wrapped
`lower_do_while_loop` signature. The extraction reduces `lowering.rs` from
23,601 to 23,502 raw lines, and the child is 106 lines. The module audit
requires both sole owners, rejects copies of `plain_async_entry_state` and
`split_resumable_loop_body`, forbids legacy `include!` assembly, budgets parent
and child separately, and fails its negative control when the child is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Five focused IR loop/resumption/refusal
filters pass `5/5`; three focused CLI lexical-environment, abrupt-finally and
iterator flat-map filters pass `3/3`. The all-target `lila-ir` and workspace
checks are green. No while/do-while behavior or conformance improvement is
claimed.

### Landed 2026-08-23: if-statement ownership

`lila-ir/src/lowering/if_statement.rs` now owns the complete conditional
lifecycle: condition lowering and static selection, branch-local var/global
facts, post-branch joins, abrupt-completion result typing and generator
yield-state splitting/merging. The statement dispatcher remains its sole
caller; shared static-expression helpers remain in the parent.

This is an exact source move. All 137 method/helper lines compare exactly after
normalizing only `fn` to `pub(super) fn` on the owner method. The extraction
reduces `lowering.rs` from 23,738 to 23,601 raw lines, and the child is 141
lines. The module audit requires the sole owner and both private lifecycle
helpers, rejects a copied `static_bool_expr`, forbids legacy `include!`
assembly, budgets parent and child separately, and fails its negative control
when the child is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Six focused IR branch/flow/resumption filters
pass `6/6`; four focused CLI inspect, Wasm, async-generator and finally filters
pass `4/4`. The all-target `lila-ir` and workspace checks are green. No
if-statement behavior or conformance improvement is claimed.

### Landed 2026-08-23: classic-for ownership

`lila-ir/src/lowering/for_loop.rs` now owns the complete classic `for`
lifecycle: async-disposable head validation, resumable-loop eligibility,
lexical TDZ setup, initializer and per-iteration Environment Record selection,
test/update/body lowering, flow-fact merging, suspension-state construction and
the final `For` or `GeneratorLoop` IR choice. The statement dispatcher remains
its sole caller; reusable loop, scope, flow and resumption helpers remain in the
parent.

This is an exact source move. The only method-body change is private-module
visibility from `fn` to `pub(super) fn`; all 209 source lines compare exactly
after normalizing that token. The extraction reduces `lowering.rs` from 23,947
to 23,738 raw lines, and the child is 213 lines. The module audit requires the
sole owner, forbids a second parent body or legacy `include!` assembly, budgets
the parent and child separately, and fails its negative control when the child
is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Eight focused IR lifecycle/refusal filters
pass `8/8`; three focused CLI lexical-environment, async-disposable and
throw-propagation filters pass `3/3`. No classic-for behavior or conformance
improvement is claimed.

### Landed 2026-08-23: statement-dispatch ownership

`lila-ir/src/lowering/statement.rs` now owns the exhaustive `Statement`
dispatcher and its resumable expression-statement specialization. Direct and
nested async `await`, generator `yield` and assignment resumption, staged
generator templates/expressions, ordinary expression statements and every
control-flow/declaration delegate remain in one closed dispatch. The focused
statement implementations and reusable suspension helpers remain parent-owned.

This is an exact source move. The only method-body change is private-module
visibility from `fn` to `pub(super) fn`; all 255 source lines compare exactly
after normalizing that token. The extraction reduces `lowering.rs` from 24,202
to 23,947 raw lines, and the child is 259 lines. The module audit requires the
sole owner, forbids a second parent body or legacy `include!` assembly, budgets
the parent and child separately, and fails its negative control when the child
is absent.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. Seven focused IR statement/resumption
filters pass `7/7`; four focused CLI inspect and Wasm control-flow filters pass
`4/4`. No statement behavior or conformance improvement is claimed.

### Landed 2026-08-23: new-expression ownership

`lila-ir/src/lowering/new_expression.rs` now owns the complete `lower_new`
lifecycle: constructor target resolution, spread-aware argument evaluation,
builtin and user-constructor result typing, Proxy trap-hint observation,
dynamic-source rejection, instance-prototype inference and static RegExp
compilation. The parent expression dispatcher remains its sole caller.

This is an exact source move. The only source-body change is private-module
visibility from `fn` to `pub(super) fn`; all constructor branches and emitted
IR choices compare exactly. The extraction reduces `lowering.rs` from 24,446
to 24,202 raw lines; the formatted child is 248 lines. The module audit
requires the sole owner, forbids a second parent body or legacy `include!`
assembly, and budgets parent and child separately.

The capped pre/post Wasm goldens both pass `2/2`, capture 635 artifacts each
and have an empty recursive diff. The all-target `lila-ir` and workspace checks
are green. Five focused IR filters pass `2/2`, `1/1`, `1/1`, `1/1` and `1/1`;
the Map/Set iterable-construction filter fails `0/2` both here and at untouched
parent `394e8fda7` with the same shape assertions. Five focused CLI filters
pass `1/1`, `2/2`, `1/1`, `1/1` and `1/1`. No constructor behavior or
conformance improvement is claimed.

### Landed 2026-08-23: property-access ownership

`lila-ir/src/lowering/property_access.rs` now owns the complete ordinary,
private and super property-access dispatcher. Primitive auto-boxing, array and
arguments exotic routing, well-known Symbol recognition, property-hook
observation and unknown-effect invalidation move together; the parent
expression dispatcher remains the sole caller.

The target-kind match now names `ValueKind::Number` explicitly instead of
using a catch-all for its existing unsupported result. That preserves current
behavior while making a future `ValueKind` addition a compile error until this
dispatcher assigns it semantics. The module audit requires that exhaustive
arm, forbids the old catch-all and enforces single ownership.

The source body is otherwise exact after normalizing private-module
visibility. The extraction reduces `lowering.rs` from 24,663 to 24,446 raw
lines; the formatted child is 223 lines. The capped pre/post Wasm goldens both
pass `2/2`, capture 635 artifacts each and have an empty recursive diff. The
all-target `lila-ir` and workspace checks are green. Serial IR filters pass
`2/2` for `property_access`, `6/6` for `property_read`, `1/1` each for
`symbol_description` and `dynamic_string_property`, and `34/34` for `call_`;
the corresponding focused CLI filters pass `3/3`, `1/1` and `6/6`. No
property-access behavior or conformance improvement is claimed.

### Landed 2026-08-23: try-statement ownership

`lila-ir/src/lowering/try_statement.rs` now owns the complete `lower_try`
lifecycle: try/catch/finally block lowering, catch-parameter Environment Record
construction, consumption of inferred thrown values, generator/async resume
planning and final `TryCatch`, `TryFinally` or `TryCatchFinally` assembly. The
parent statement dispatcher remains its sole caller. Reusable throw-value
inference now lives in `lowering/throw_inference.rs`; shared analysis helpers
remain parent-owned.

The former eight-field catch tuple and five-field finally tuple are now private
`LoweredCatchClause` and `LoweredFinallyClause` records. Named generator/async
entry and exit fields make positional state transposition impossible at every
plan and final-assembly use site. The module audit requires both records and
rejects positional tuple-field access in this owner.

The extraction reduces `lowering.rs` from 24,910 to 24,663 raw lines; the
formatted child is 264 lines. The capped pre/post Wasm goldens both pass `2/2`,
capture 635 artifacts each and have an empty recursive diff. The focused
all-target `lila-ir` check and the new module boundary are green. Serial IR
filters pass `12/12` for `try_` and `14/14` for `catch`; the CLI `finally`,
`catchability` and `top_level_try` filters pass `2/2` each. No try-statement
behavior or conformance improvement is claimed.

### Landed 2026-08-23: delete-expression ownership

`lila-ir/src/lowering/delete_expression.rs` now owns the complete
`lower_delete` target dispatcher across ordinary/private/super property
References, identifier deletion and non-Reference values. The parent retains
the sole unary-expression call and the reusable shape, strictness and property
helpers consumed by the implementation.

This is a semantic-free source move. All 213 method lines compare exactly to
the pre-move implementation after normalizing only `fn` to `pub(super) fn`.
The private child boundary reduces `lowering.rs` from 25,123 to 24,910 raw
lines; the formatted child is 217 lines. The module audit requires the sole
owner method, forbids a second parent body or legacy `include!` assembly, and
budgets parent and child separately.

The extraction does not combine the two currently duplicate computed-key
branches. That cleanup can be reviewed separately after this exact move's
behavioral checkpoint; this commit preserves the existing instruction and
invalidation choices byte-for-byte at the Rust source level. No delete behavior
or conformance improvement is claimed.

The capped workspace/all-target check is green. Serial delete-focused coverage
passes `7/7` in `lila-cli`, `2/2` in `lila-aot-wasm` and `4/4` in
`lila-engine`. Formatting, exact source comparison, module-boundary and
task-plan audits are green.

### Landed 2026-08-23: assignment-expression ownership

`lila-ir/src/lowering/assignment.rs` now owns the complete exhaustive
`lower_assign` dispatcher across identifier, property, private, destructuring,
logical and eager compound assignment. The parent expression match remains its
single caller. Specialized ordinary-property and Object Environment Record
Reference lifecycles remain in their existing typed child modules.

This is a semantic-free source move. All 706 body lines compare exactly to the
pre-move implementation; the signature changes only private-module visibility
and rustfmt's multiline layout. The child boundary reduces `lowering.rs` from
25,830 to 25,123 raw lines; the formatted child is 716 lines. The module audit
requires the sole owner method, forbids a second parent body or legacy
`include!` assembly, and budgets parent and child separately.

The capped workspace/all-target check and serial IR `assignment` cohort
(`34/34`) are green. The serial CLI `assignment` cohort reports `6/7` both
before and after extraction; the same with-environment compound-assignment
fixture fails with the same completion and error text. Formatting, exact body
comparison, module-boundary and task-plan audits are green. No assignment
behavior or conformance improvement is claimed.

### Landed 2026-08-23: ordinary function-definition ownership

`lila-ir/src/lowering/function_definition.rs` now owns the complete ordinary
`lower_function` lifecycle: nested lowerer creation, analysis-state transfer,
parameter and body lowering, capture/lexical-environment planning, signature
updates, resumable metadata and final `FunctionIr` construction. The seven
top-level orchestration calls remain in the parent; parameter helpers shared
with generated iterators, class methods and object methods also remain there.

This is a semantic-free source move. All 717 method lines compare exactly to
the pre-move implementation after normalizing only `fn` to `pub(super) fn`.
The private child boundary reduces `lowering.rs` from 26,547 to 25,830 raw
lines; the formatted child is 721 lines. The module audit requires the sole
owner method, forbids a second parent body or legacy `include!` assembly, and
budgets parent and child separately.

The capped workspace/all-target check and serial IR `function_` cohort
(`61/61`) are green. The serial CLI `functions::` cohort reports `45/49`; both
inspect-shape assertions and both mapped-arguments semantics fixtures reproduce
at the exact pre-extraction commit `bda775dfc`. Formatting, exact source
comparison, module-boundary and task-plan audits are green. No function
behavior or conformance improvement is claimed.

### Landed 2026-08-23: builtin call-result analysis ownership

`lila-ir/src/lowering/builtin_call_info.rs` now owns the complete exhaustive
`StandardBuiltinId` result analysis: return kinds and shapes, boxed-builtin
accounting, callback parameter observations and the few result-dependent flow
invalidations. Construct lowering, general resolved calls, RegExp literal
lowering and well-known-symbol method routing remain its four consumers.

This is a semantic-free source move. All 2,146 method lines compare exactly to
the pre-move implementation after normalizing only `fn` to `pub(super) fn`.
The private child boundary reduces `lowering.rs` from 28,693 to 26,547 raw
lines; the formatted child is 2,150 lines. The module audit requires the sole
owner method, forbids a second parent body or legacy `include!` assembly, and
budgets parent and child separately.

The capped workspace/all-target check and the serial CLI `call_` cohort (`6/6`)
are green. The current serial IR `call_` cohort is also green (`34/34`) after a
follow-up contract refresh accepted both typed `PropertyRead` and canonical
`GetV` as the same materialized method Reference read. Formatting, exact source
comparison, module-boundary and task-plan audits are green. No call behavior or
conformance improvement is claimed.

### Landed 2026-08-23: Atomics backend ownership

`lila-aot-wasm/src/builtins/atomics.rs` now owns all fourteen Atomics builtin
bodies, the shared integer-operation machinery, synchronous and asynchronous
wait/notify state transitions, host-agent calls, and atomic memory access
helpers. The flat catalog dispatch retains one typed delegate per builtin
through the closed `AtomicsBuiltin` domain; the family file cannot accept an
unrelated `StandardBuiltinId`.

The extraction also replaces the old RMW helper's broad nine-case operation
parameter and four `unreachable!` catch-alls with a six-case
`AtomicsRmwOperation`. Load, store and compare-exchange can no longer reach an
RMW opcode selector. Only three methods cross the family boundary: the BigInt
element-kind predicate shared with `TypedArray.prototype.with`, the event-loop
waiter-drain checkpoint, and the Promise-job waiter-poll checkpoint.

The moved emitter bodies and their instruction sequences are source-identical
to the pre-move implementation. The non-emitting structural changes are the
typed family dispatch, the narrower RMW carrier and one deliberate visibility
change. The `Atomics.isLockFree` body is unchanged apart from becoming a
private family method. `standard.rs` falls from 33,275 to 30,567 raw lines;
the formatted family file is 2,805 lines. The boundary audit requires the
module, all fourteen typed delegates, the three closed domains, the three
reviewed cross-family hooks, exhaustive matches and separate line budgets for
parent and child.

The capped workspace/all-target check is green. Focused serial Atomics coverage
passes `2/2` in `lila-aot-wasm` and `5/5` in `lila-engine`; the CLI cohort passes
`12/13`. Its remaining `Atomics.isLockFree` core-fixture failure reproduces at
the untouched parent commit, so this structural extraction neither owns nor
claims to fix it. Formatting, module-boundary and task-plan audits are green.

### Landed 2026-08-28: Atomics fixed dispatch boundary

The fourteen Atomics catalog arms now enter `builtins/atomics.rs` through
fixed sibling-visible methods. The raw, non-derived `AtomicsBuiltin` selection
domain and `emit_atomics_builtin` selector are private, so the shared catalog
dispatcher can no longer construct or forward Atomics family policy.

Entry- and created-Realm installation now share the exact ordered
`ATOMICS_PUBLICATION_ORDER: [StandardBuiltinId; 14]` result directly. This
removes the raw Atomics domain and its projection function from both publication
owners without changing their order or catalog metadata lookups. Reconstructing
the former declaration and selector produces the exact original 39-line
selection with SHA-256
`3382f4b6d98ca6acfb04ad9c9f452bd1f93bf65f9d3334e0cef0f17583366231`.

The strengthened
[`Atomics dispatch contract`](../docs/rust-rewrite/contracts/atomics-builtin-dispatch-boundary.md)
target passes `5/5`; seven neighboring Atomics structure targets pass `27/27`.
The capped workspace/all-target check is green, and the exact entry-Realm
surface, created-Realm borrowed-method and created-Realm `waitAsync` controls
each pass `1/1`. Formatting, module-boundary, task-plan and shortcut gates are
green. This is source-equivalent hardening with no new Atomics behavior or
conformance claim.

### Landed 2026-08-23: call-expression lowering ownership

`lila-ir/src/lowering/call_expression.rs` now owns the complete `lower_call`
implementation, including direct-call recognition, builtin and method routing,
argument evaluation, specialization and final indirect-call construction. The
parent keeps expression dispatch and the reusable call, shape and static-value
helpers consumed by that implementation.

This is a semantic-free source move. The only visibility change is the one
`pub(super)` method consumed by parent expression dispatch; the child remains
inside the private `lowering` module. The extraction reduces `lowering.rs` from
31,833 to 28,693 raw lines. The boundary audit requires the child module and
sole owner method, forbids a second parent body or legacy `include!` assembly,
budgets the child at 3,200 raw lines and tightens the parent budget to 29,000.

The extraction is verified by an exact normalized source-body comparison with
the pre-move implementation and a green `cargo check --workspace --all-targets`.
The serial call-focused IR cohort reports 63/67 and the serial CLI call cohort
reports 35/36. All five failures reproduce unchanged at pre-extraction commit
`f2309be48`: four primitive/ordinary method-call inference contracts and the
`arguments.callee` CLI fixture. The module-boundary and task-plan audits are
green. No call behavior or conformance improvement is claimed.

A later contract refresh removed the last current IR `call_` false negative.
Flow widening legitimately selects the canonical `GetV` carrier for an
ordinary method read; the test now verifies that either `GetV` or the typed
`PropertyRead` consumes the one materialized receiver before Call supplies the
same value as `this`. The focused test and current serial IR cohort pass `1/1`
and `34/34`; an independent Wasm-AOT witness completes with `boolean(true)`.
Production lowering and emitted semantics are unchanged.

### Landed 2026-08-23: class-definition lowering ownership

`lila-ir/src/lowering/class_definition.rs` now owns the complete
`lower_class_common_in_name_scope` implementation: heritage validation, public
and private method/field planning, auto-accessor backing and generated-function
scheduling, instance/static initialization plans, shapes and final typed
`ClassDefinitionIr` construction. The parent keeps the declaration/expression
entrypoints, class-name scope orchestration and generated-function helpers.

This is a semantic-free source move. The only visibility change is the one
`pub(super)` method consumed by the parent orchestrator; the child remains
inside the private `lowering` module. The extraction reduces `lowering.rs` from
33,156 to 31,833 raw lines, restoring the enforced 32,000-line boundary. The
boundary audit requires the child module and sole owner method, forbids a
second parent body or legacy `include!` assembly, and budgets the child at
1,400 raw lines.

The extraction is verified by an exact normalized source-body comparison with
the pre-move implementation, `cargo check --workspace --all-targets`, the
focused IR auto-accessor regression (1/1), and the serial CLI class group
(27/27). The module-boundary and task-plan audits are also green. No class
behavior or conformance improvement is claimed.

### Landed 2026-08-17: bound-function invoker ownership

The hidden `BoundFunctionInvoker` body now lives beside
`Function.prototype.bind` in `builtins/function.rs`. `FunctionBuiltin` is the
closed six-case backend domain for the five public Function intrinsics and the
one internal call/construct invoker they create; `builtins/standard.rs` keeps
only one typed delegate per case. This closes the remaining Function-owned body
that unrelated builtin work still had to cross in the shared dispatcher and
reduces that parent from 33,342 to 33,248 raw lines.

The invoker body is a semantic-free source move. Comparing it with the frozen
pre-move `builtins/standard.rs` arm after normalizing only the enum qualifier
shows the same instruction, temporary-local reservation and reverse-release
order. Its existing call, construct, nested `new.target` and heap-rooting CLI
fixtures remain the behavioral characterization; they were not executed while
Cargo and runtime verification remain under the centralized lease.

The module-boundary audit requires all six typed delegates, the unique hidden
variant and its unique child match arm, rejects catch-all/unreachable escape
routes, and budgets both the newly complete family file and the smaller parent.
The static write-phase gates are green: `git diff --check`, the bounded
source-body comparison, `check-module-boundaries.sh`, `check-task-plan.sh` and
focused `rustfmt`. No Function, bind, call/construct, `new.target`, heap-rooting
or conformance behavior improvement is claimed by this extraction.

### Landed 2026-08-13: Number intrinsic ownership

The complete eleven-member Number intrinsic family now lives in
`builtins/number.rs`: `Number`, its four non-coercing predicates and its six
prototype methods. The catalog match keeps one typed delegate per ID. A closed
`NumberBuiltin` owns the parent-facing family choice, and a private closed
`NumberPrototypeOperation` owns the six operations that share receiver
validation. Neither domain admits an unrelated `StandardBuiltinId`; both
behavior matches are exhaustive and the boundary audit rejects catch-all arms.

This is a semantic-free source move. Static source/token comparisons against
the parent commit `4dec427e6` confirm the same emitted instruction and temporary-
local order for all four predicates, the Number and remaining String constructor
paths, the shared prototype receiver path, all six prototype operations and the
final local releases. Numeric conversion and formatting algorithms remain with
`operations.rs`. A focused CLI fixture characterizes the intended unchanged
runtime family surface, but has not executed while the resource-bounded matrix
owns Cargo and Test262.

The static write-phase gates are green: `git diff --check`,
`check-module-boundaries.sh`, `check-task-plan.sh` and
`rustfmt --edition 2024 --check --config skip_children=true` over the four
touched Rust files (`builtins/mod.rs`, `builtins/number.rs`,
`builtins/standard.rs` and `cli/language_numerics.rs`). `skip_children=true` is
required so this focused gate does not recursively format unrelated builtin
modules. Compile, focused fixture execution, current-pin Number and pre/post
golden gates remain deferred. The pre-edit golden must later be built from
parent commit `4dec427e6` in a separate worktree after copying the committed
`wasm_number_builtin_family.js` fixture into that worktree. The before and after
captures therefore use the same 583-fixture corpus and their formal `diff -r`
remains meaningful and reproducible. No Number behavior or conformance
improvement is claimed by this extraction.

### Landed 2026-08-12–13: builtin metadata and family body boundaries

Fourteen previously coupled builtin stores now have separate owners:

- `lila-ir/src/lowering/builtin_shapes.rs` owns 98 pure shape/signature
  constructors. At extraction, `lowering.rs` fell from 39,177 to 31,979 lines;
  subsequent work leaves it at 31,998 lines, below the enforced cap. The moved
  methods have only parent-module visibility except the existing crate test
  hook.
- `lila-ir/src/builtins/catalog.rs` is the single 779-row
  `StandardBuiltinId` registry. One row generates the enum, names, flags,
  function-ID mappings and independent function/global order arrays. Typed
  dense ordinals plus const duplicate/hole/ID checks preserve the deliberately
  different declaration, 779-function and 52-global orders.
- `lila-aot-wasm/src/builtins/object.rs` owns all 34 Object builtin bodies and
  their private helpers. Three grouped choices are closed enums rather than
  generic builtin IDs or booleans.
- `lila-aot-wasm/src/builtins/proxy.rs` owns the three Proxy lifecycle bodies;
  `reflect.rs` remains the separate owner of the 13 Reflect bodies and their
  proxy-trap machinery.
- `lila-aot-wasm/src/builtins/math.rs` owns all 37 Math bodies behind a private
  closed `MathBuiltin` domain. `standard.rs` gives every Math ID its own typed
  one-line delegate, and both Math behavior matches are exhaustive. The
  min/max direction is a two-case private enum rather than a generic builtin
  ID. After the Object, Proxy and Math moves, `standard.rs` has fallen from
  49,179 to 36,807 lines.
- `lila-aot-wasm/src/builtins/symbol.rs` owns all seven Symbol bodies and the
  three shared Symbol receiver/description helpers behind a private closed
  `SymbolBuiltin` domain. `String(symbol)` reaches the one helper it shares
  through a parent-private method; the remaining helpers cannot escape the
  family. The catalog dispatch keeps seven typed delegates, and `standard.rs`
  fell from 36,789 to 36,313 lines without changing an emitted instruction.
- `lila-aot-wasm/src/builtins/bigint.rs` owns all six BigInt intrinsic bodies
  behind a private closed `BigIntBuiltin` domain. The constructor, signed and
  unsigned fixed-width operations, and three prototype methods moved verbatim;
  general BigInt conversion, allocation and stringification helpers remain
  with their existing operation and heap owners. The catalog dispatch keeps
  six typed delegates, and `standard.rs` fell from 36,313 to 35,647 lines.
- `lila-aot-wasm/src/builtins/boolean.rs` owns all three Boolean intrinsic
  bodies behind a private closed `BooleanBuiltin` domain. The constructor keeps
  the same argument/result-local ordering previously shared with Number and
  String, while the two prototype methods keep their boxed-receiver checks and
  realm-local TypeError route together. After the intervening T20 residue
  consolidation, the extraction reduced `standard.rs` from 35,532 to 35,439
  lines.
- `lila-aot-wasm/src/builtins/number.rs` owns all eleven Number intrinsic bodies
  behind a closed `NumberBuiltin` domain. A private closed
  `NumberPrototypeOperation` owns the six methods that share Number receiver
  validation; the constructor and four static predicates complete the family.
  Eleven typed delegates preserve the flat catalog dispatch, and `standard.rs`
  fell from 33,730 to 33,512 lines.
- `lila-aot-wasm/src/builtins/function.rs` owns the complete five-member
  Function intrinsic family and its hidden bound-function call/construct
  invoker behind a private closed `FunctionBuiltin` domain. The catalog
  dispatch keeps six typed delegates, while the moved bodies retain their exact
  instruction and temporary-local order. The original intrinsic extraction
  reduced `standard.rs` from 34,461 to 34,088 lines; moving the remaining
  invoker body reduces it again without changing the public family.
- `lila-aot-wasm/src/builtins/uri.rs` owns all six global URI and Annex-B codec
  wrappers behind a private closed `UriBuiltin` domain. The UTF-8/UTF-16 codec
  primitives remain with their existing `string.rs` owner; only the complete
  global wrapper family moved. Six typed delegates preserve the flat catalog
  dispatch, and `standard.rs` fell from 35,439 to 35,394 lines.
- `lila-aot-wasm/src/builtins/global_numeric.rs` owns both coercing global
  numeric predicate bodies behind a private closed `GlobalNumericBuiltin`
  domain. `Number.isFinite` and `Number.isNaN` remain with the distinct
  non-coercing Number family, while `parseInt` and `parseFloat` remain host
  builtin emitters. The catalog dispatch keeps one typed delegate for each of
  `isFinite` and `isNaN`, and `standard.rs` fell from 35,394 to 35,372 lines.
- `lila-aot-wasm/src/builtins/errors.rs` owns the complete eleven-member Error
  intrinsic family as well as its pre-existing allocation, realm-prototype,
  cause, iterable and throw helpers. A private closed `ErrorBuiltin` domain
  distinguishes the static predicate, the nine constructors carried by the
  existing closed `NativeErrorKind`, and `Error.prototype.toString`; unrelated
  `StandardBuiltinId` values cannot reach this family emitter. Eleven typed
  delegates preserve the catalog dispatch without duplicating the error-kind
  registry, and `standard.rs` fell from 35,372 to 34,948 lines.
- `lila-aot-wasm/src/builtins/json.rs` owns all four JSON namespace bodies
  alongside the parse, reviver, stringify and raw-JSON machinery they already
  consume. A private closed `JsonBuiltin` domain covers `parse`, `stringify`,
  `rawJSON` and `isRawJSON`; hidden static-JSON lowering and runtime helpers
  remain implementation details rather than pretend namespace members. Four
  typed delegates preserve the flat catalog dispatch, and `standard.rs` fell
  from 34,948 to 34,461 lines.
- `lila-aot-wasm/src/builtins/atomics.rs` owns all fourteen Atomics bodies,
  integer/RMW operation domains, wait queues, host-agent calls and atomic
  memory helpers. Fourteen typed delegates preserve the catalog dispatch;
  three explicitly checked helpers remain visible to the TypedArray,
  event-loop and Promise consumers. `standard.rs` fell from 33,275 to 30,567
  lines.

Batch AS closes the remaining dispatcher visibility for the Boolean and Symbol
families. A private `BooleanBuiltin` has three fixed Boolean entries, and a
private `SymbolBuiltin` with no derived capabilities is reachable only through
seven fixed Symbol entries. Their frozen domain/emitter
selections have SHA-256
`48961edd05a7a1789538b92ad90ed76232fad5156cec5144214122dd4c52eaab` and
`3296276e16255ea9aaf39f05b54b77414320a0f71d5c0d4c1a61ed04c1cef9b2`;
restoring only the former visibility and Symbol derive reproduces both sources
exactly. At the 2026-08-28 Batch AS checkpoint, `cargo xc` is green and both
strengthened structure targets pass `4/4`. The exact boxed-builtin CLI owner
passes `1/1`; the selected Boolean leaves pass `8/8` and the selected Symbol
leaves pass `4/4`, with every failure bucket at zero. These source-equivalent
boundaries claim no new Boolean behavior. They claim no new Symbol behavior or
broader or published conformance result.

Batch AT makes the outer Number family a private `NumberBuiltin`; standard
dispatch can reach it only through eleven fixed Number entries. The frozen
160-line domain/emitter selection has SHA-256
`7465f52181186c7cd1dd4bb2be3fa2a124ac6794fe509c4d7a0e003984091e9a`;
restoring the former enum/emitter visibility and constructor worker name
reproduces that source exactly. At the 2026-08-28 Batch AT checkpoint,
`cargo xc` is green, the strengthened structure target passes `4/4`, and the aggregate
Number runtime witness passes `1/1`. No Test262 leaf or semantic golden was
required for this dispatcher-only closure. This source-equivalent boundary
claims no new Number behavior, broader conformance or published
conformance-count change.

Batch AU gives the raw Date setter family a private `DateComponentSetterOperation`
with no derived capabilities. The fourteen
local/UTC catalog IDs can reach it only through seven fixed Date setter entries.
The frozen 306-line domain/emitter selection has SHA-256
`53813c73ebb92bdaa9541b57c83694c11c4f3dcc214c8cc27f056eb980d44240`;
restoring only the former derive and visibility reproduces that source exactly.
At the 2026-08-28 Batch AU checkpoint, `cargo xc` is green, the strengthened
structure target passes `4/4`, the exact setter CLI fixture passes `1/1`, and
the focused `setUTCMinutes` leaf passes both sloppy/strict Wasm-AOT executions
`2/2` with every failure bucket at zero. This source-equivalent boundary claims
no new Date behavior, local-time/default-time-zone support, broader conformance
or published conformance-count change.

Batch AV makes the outer Function family a private `FunctionBuiltin` with no
derived capabilities. It has eight fixed Function entries: seven public
intrinsic operations and the separately named hidden bound-function invoker.
The frozen 409-line domain/emitter selection has
SHA-256
`f922e7edf4c8c1626a9b40920c2a9f418c8b3badcce3c347ffb09b55109d2093`;
restoring only the former derive and visibility reproduces that source exactly.
`cargo xc` passes. The receiver-ownership, callable-prototype and
`Symbol.hasInstance` structure targets pass `4/4`, `8/8` and `5/5`; the exact
Function-builtin Wasm-AOT CLI fixture passes `1/1`. No Test262 or Wasm golden
was required for this source-equivalent boundary, which claims no new Function behavior,
conformance result or published-count change.

Batch AW makes both capability-free Math domains and their raw exhaustive
emitter private to `builtins/math.rs`. Standard dispatch can reach the family
only through 37 fixed Math entries, one per namespace operation. The frozen
825-line domain/emitter selection has SHA-256
`25cedc56bf9f821608dad8f2c4b3d6b079a09279bbc5ca6e0703679d16e98049`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The policy, extremum, `hypot`, `sumPrecise` limb and
`sumPrecise` runtime structure targets pass `4/4`, `3/3`, `3/3`, `3/3` and
`6/6`; the three established Math Wasm-AOT CLI controls pass `3/3`. No Test262
leaf or Wasm golden was required for this source-equivalent dispatcher boundary,
which claims no new Math behavior, conformance result or published-count change.

Batch AX makes the builtin, fixed-width, prototype-result and three
result-authority domains, their associated prototype producers and the raw
exhaustive emitter private to `builtins/bigint.rs`. Standard dispatch reaches
them only through six fixed BigInt entries. The frozen 736-line domain/emitter
selection has SHA-256
`5b61c6cfedaf3b988517eab492bb6c3dedb85a5eb9ac98992120ff39e7f30f18`;
restoring only the former visibilities reproduces that source exactly. Batch AX
`cargo xc` passes. The fixed-width, heap-slot, helper-operation, number-policy
and prototype-result structure targets pass `5/5`, `4/4`, `4/4`, `2/2` and
`4/4`. The constructor, arbitrary-width and wrapper-coercion Wasm-AOT CLI
controls pass `3/3`; a direct stdin control exercises `toString`,
`toLocaleString` and `valueOf` together and returns `number(3)`. The broader
prototype fixture remains red at its unrelated captured-main-lexical Symbol
Realm assertion. No Test262 leaf or Wasm golden was required for this
source-equivalent boundary, which claims no new BigInt behavior, conformance
result or published-count change.

Batch AY makes `StringSymbolHookOperation` and its raw emitter private to
`builtins/string.rs`. Standard dispatch reaches them only through five fixed String symbol-hook entries.
The frozen 306-line domain/emitter selection has SHA-256
`06636af9cd91f1e237e7cb08d47132941a9976c712a818073d1c208ce1271c26`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The symbol-hook, literal-replacement and RegExp
result-mode structure targets pass `5/5`, `3/3` and `3/3`; the complete
symbol-hook Wasm-AOT CLI fixture passes `1/1`. No Test262 leaf or Wasm golden
was required for this source-equivalent boundary, which claims no new String behavior,
conformance result or published-count change.

Batch AZ makes `RegExpFlagGetter` and its raw emitter private to
`builtins/string.rs`. Standard dispatch reaches them only through eight fixed RegExp flag-getter entries.
The frozen 93-line domain/emitter selection has SHA-256
`0bd635a1625364b6db7514af3ce13b96166d14614f9ec5ee5c6f7b25fbd76829`;
restoring only the former enum and emitter visibility reproduces that source
exactly. `cargo xc` passes. The strengthened flag-getter and neighboring
symbol-hook structure targets pass `4/4` and `5/5`; the complete RegExp
prototype-accessor Wasm-AOT CLI fixture passes `1/1`. No Test262 leaf or Wasm
golden was required for this source-equivalent boundary, which claims no new RegExp behavior,
conformance result or published-count change.

The earlier central feature-enabled CLI compile, which covers `lila-aot-wasm`
and `lila-intl`, and the focused builtin catalog tests pass for the moves that
reached that checkpoint. The source moves were also compared against their
pre-extraction bodies, and the boundary audit prevents these stores from being
folded back into their parents. The later
Proxy move is source-equivalent by a static body comparison and is included in
the green compile checkpoint and product-artifact boundary proof. The Math move
is statically source-equivalent, boundary-checked, and covered by that compile
checkpoint. The later Symbol move is statically source-equivalent and
boundary-checked, passes the centralized feature-enabled CLI compile, and is
covered by the exact String/Symbol hook fixture through the product Wasm
backend. The BigInt move is statically source-equivalent and boundary-checked;
its centralized feature-enabled compile and the exact constructor/fixed-width,
wrapper-coercion and cross-realm prototype behavior checkpoints are green.
The Boolean move is statically instruction-sequence equivalent and
boundary-checked; its compile, focused fixture, and real Boolean shard gates
remain queued behind the active resource-bounded matrix run.
The original Function move is an exact 389-line body match after normalizing
only the five public enum arm headers. The later hidden-invoker move preserves
its complete body after normalizing only that sixth arm header. Compile,
focused constructor/call/apply/bind/toString and bound-call/construct fixtures,
and the real `built-ins/Function` shard remain queued behind the same matrix
run.
The URI move is statically source-equivalent after normalizing only the closed
enum path and rustfmt's block-expression layout, and is boundary-checked; its
compile, focused global-codec fixtures and real URI/Annex-B shard gates remain
queued behind the same matrix run.
The global numeric move is statically source-equivalent after normalizing only
the closed enum path, and is boundary-checked; its compile, focused coercion
and cross-realm fixture gates and real `isFinite`/`isNaN` shards remain queued
behind that matrix run.
The Error move preserves the existing emitter and local-allocation sequences;
its only semantic-free rewrites replace raw builtin-ID tests with the closed
`ErrorBuiltin` and `NativeErrorKind` domains. Its compile, focused constructor,
cross-realm, static predicate and prototype-method fixtures, and real Error,
NativeErrors, AggregateError and SuppressedError shards remain queued behind
the same matrix run.
The JSON move is a verbatim body extraction after normalizing only the closed
enum path and rustfmt layout. Its compile, focused parse/reviver, stringify,
raw-JSON and cross-realm gates, and real `built-ins/JSON` shard remain queued
behind the same matrix run.
The Number move is statically source/token-equivalent for the four predicates,
both split constructor paths, shared receiver path, six prototype operations and
temporary-local releases. Its static boundary/task/diff/rustfmt gates are green;
compile, focused fixture, pre/post golden and real `built-ins/Number` gates remain
queued behind the same matrix run.

### Landed 2026-07-31: the `intrinsics/` boundary

`crates/lila-aot-wasm/src/intrinsics/` now holds per-family realm bootstrap
and property-descriptor installation, extracted from
`builtins/bootstrap.rs::init_builtin_constructor_object`. That function was a
single ~4,760-line body and the worst merge point in the backend: two lanes
adding builtins to unrelated families still collided inside it.

`bootstrap.rs` went from 8,080 to 4,117 lines; 23 dispatch arms became one-line
delegations into 15 family modules. The boundary is enforced by
`check-module-boundaries.sh`.

Every arm moved **verbatim** — installers destructure an `IntrinsicInstall`
context back into the original identifier names (including `builtin`, which
multi-variant arms branch on), so no body text was rewritten. The move was
verified byte-identical across all 527 CLI fixtures with
`crates/lila-aot-wasm/tests/emit_golden.rs`, which matters because property
installation order is observable through `Object.keys` and the ordinary suites
assert on program output rather than emitted bytes.

The earlier intrinsic split left three immediate follow-ups. All now have
bounded owners:

- **Resolved 2026-08-12:** the append-only no-op dispatch is gone. The standard
  builtin catalog requires a closed installer class on every row, and
  `bootstrap.rs` consumes it through an exhaustive installer match.
- **Resolved 2026-08-12:** the parallel `StandardBuiltinId` tables are one
  catalog with compile-time ordering and uniqueness invariants.
- **Resolved for Object, Proxy, Math, Symbol, BigInt, Boolean, Number, Function,
  global numeric, URI, Error and JSON 2026-08-13:** their bodies are family
  modules; Reflect already has the same boundary. Other large inline families
  should follow the same exhaustive-delegate shape.

### Landed 2026-08-12: catalog-owned bootstrap routing

`init_builtin_constructor_object` performs common function/prototype setup for
every initialized `StandardBuiltinId`, but only 34 IDs then run one of 33
family intrinsic installers. Before this seam that distinction was encoded
backwards: the 33 productive arms were followed by no-op arms naming every
other ID. Adding a builtin compiled only after someone appended its name to an
unrelated no-op tail, while the catalog that owned the builtin could not say
whether an installer was required.

The landed seam is a mandatory catalog field whose value is a closed
`StandardBuiltinInstaller` domain. `None` must skip family dispatch; every
other case must be consumed by an exhaustive backend match that invokes the
corresponding installer. This is behavioral routing rather than passive
metadata: omitting the field makes a new catalog row fail to parse, and adding
an installer variant makes the backend fail to compile until it handles the
case. The existing catalog/function iteration and the location of dispatch
after common setup remain unchanged, preserving construction and observable
property-installation order.

The catalog now records 35 productive roots across 34 installer classes and
745 explicit `None` choices. `ArrayBuffer` and `SharedArrayBuffer` deliberately
share one class because their installer branches on the carried builtin ID.
The backend match contains only the productive classes; the former raw-ID
no-op groups were deleted, reducing `builtins/bootstrap.rs` from 4,903 to 4,156
lines. A catalog contract pins the productive root sequence, and the module
boundary audit requires both the mandatory field and the typed backend
dispatch. The focused catalog contract and central feature-enabled CLI compile
are green; broader behavioral suites remain part of this task's acceptance
gate.

The complete native-error prototype-location authority now has one private
`module/error_prototype_location.rs` owner. Its private two-address record,
private exhaustive nine-kind mapping and three existing `pub(crate)`
projections moved together; `module.rs` narrowly re-exports only those
projections, so the five reviewed callers retain their existing paths and
visibility. The exact 71-line moved body retains SHA-256
`72b4b095cedd7316a5bd3217ec415a65c2a9dbbc2e15ac85298cb77c8977e250`;
the formatted 73-line child has SHA-256
`4dd1ae683bc4beb529e994175d1fe85891b420e0cdc5650903992ca21f8963d3`
and reduces `module.rs` from 2,517 to 2,450 lines. The dedicated structure
target enforces private file ownership, exact visibility, all nine exhaustive
arms, the single derived entry table and the unchanged caller census, and
passes `3/3`. Broader Cargo, runtime, CLI, Test262 and Wasm-golden gates remain
deferred to the shared verifier; no behavior or conformance change is claimed
for this ownership-only extraction. The shared verifier subsequently passed
`cargo xc` and every repository policy gate. Its 647-artifact pre/post Wasm
golden had an empty recursive diff before the later mixed-BigInt equality
repair.

The complete BigInt helper-operation selector now has one private
`bigint/helper_op.rs` owner. Its fourteen explicitly numbered ABI variants and
two exhaustive arithmetic/bitwise projections moved together; `bigint.rs`
narrowly re-exports only `BigIntHelperOp`, so the unchanged parent, expression
and operation callers retain their existing paths and visibility. The exact
47-line moved body retains SHA-256
`df7653a425f32b07243ab026658a3b616b8f108df892463ba61ad003b27a8690`;
the formatted 49-line child has SHA-256
`67291fc679bb1455e95aea4015a931577f588343247ab39e2e2a59b17ff47c09`
and reduces `bigint.rs` from 2,910 to 2,865 lines. The dedicated structure
target passes `3/3`, and the focused BigInt exponentiation CLI witness passes
`1/1`. The focused bitwise CLI witness remains red: direct execution throws
`TypeError: Cannot convert BigInt to number`; this source-equivalent ownership
move does not claim to repair that runtime behavior. Broader Cargo, shared
module-boundary, Test262 and Wasm-golden gates were then covered by the shared
checkpoint: `cargo xc` and every repository policy gate pass, and its
647-artifact pre/post Wasm golden has an empty recursive diff.

The complete RegExp Unicode-property range-search policy now has one private
`builtins/regexp/range_search.rs` owner. Its closed two-bound domain,
exhaustive offset projection, sole raw reader and complete binary search moved
together; the parent retains exactly its two semantic forward/reverse mismatch
calls and cannot construct a bound, project an offset or call the raw reader.
The exact 14-line domain/projection and 101-line search/reader selections
retain visibility-normalized SHA-256
`7ac765b2195a8ad7e2935bbfb3da1b9e8e641a63906bb007eb67ea49e0da17b6`
and
`eb9ceaad299ab3277aa5bbf1228776d74098876f79f25bed106179e699489098`;
their combined 115-line selection retains SHA-256
`14e35ba4c6a910319e4e301ded5213315ba30fe537a3d92fcd2c3207b29b7801`.
The resulting 3,661-line parent and 120-line child have SHA-256
`60a443e0f39f719c28871815f5be6c7a7fd638e8389e6070167db05abc09b30b`
and
`c36626fb9c53468a49449012538a9ad32e80c37c9387ee7252d807864f9c9e8f`.
Both unchanged 11-line parent calls retain SHA-256
`125fa46e9fab12f49c12f6280b95f618baa2d1cb88fad021fb7fc26029e63ab2`.
The recursive guard and module policy pin zero parent raw-policy mentions,
exact child censuses of five domain mentions, two qualified variants and three
reader sites, and the two parent semantic calls. This is source-equivalent
apart from required child-method visibility. At the Batch AA checkpoint,
`cargo xc` is green, the range target passes `3/3`, the neighboring
matcher-result and Unicode-sets targets pass `4/4` and `7/7`, and the exact
Unicode-property CLI witness passes `1/1`. The emitted-Wasm golden was not
rerun.

The complete RegExp GetSubstitution policy now has one private
`builtins/string/regexp_substitution.rs` owner. Its capability-free six-kind
domain, ordered runtime-code authority, every recognizer and the exhaustive
semantic handler moved with the sole algorithm. The 20,970-line parent retains
only one semantic call and cannot name a kind or encode its raw code. The exact
30-line domain/authority and 448-line algorithm selections retain visibility-
normalized SHA-256
`0f852520992bfe2689f1ba08c1351c8accc5921373cbb32c2ac1f493b56ab453`
and
`d11dd555a3b82a43496296de74b04367c50ffa0fc2b148f8c2b1eb2453ee0d8d`;
their combined 478-line hash is
`c8deaa00580f7d7a74e684273325a4e7b496c3aa39f69d66a7b7da8cfb02f2dd`.
The 483-line child has SHA-256
`5163b1c56b48ee90a6f3ee5ea6f5c19ad013ea1462090c526e3e951bee43a473`,
while the parent has SHA-256
`62caf68bd5a9bc02354c8fdc31b1d73d467a374d68a82717561adcf810a2dd3f`.
The unchanged 14-line call retains SHA-256
`fcecb3ddcc9b61f06b276734b76b5c04211dffb75d962f0f01c0e2f43a862b8a`.
The recursive guard and module policy pin zero parent raw-policy names, all 15
domain mentions and four runtime-code projections in the child, and exactly
one parent semantic call. This is source-equivalent apart from required child-
method visibility. At the Batch AB checkpoint, `cargo xc` is green, the owner
target passes `4/4`, the neighboring flag-getter and literal-replacement
targets pass `3/3` each, and the six substitution leaves pass all `12/12`
Wasm-AOT variants with every failure bucket at zero. No CLI fixture or
emitted-Wasm golden was run for the owner move.

The complete duplicate-named-group pattern policy now has one private
`builtins/string/duplicate_named_group_pattern.rs` owner. Its capability-free
two-variant domain and sole raw pattern-parameterized emitter moved together;
the 20,883-line String parent retains exactly two semantic matcher calls and
cannot name, construct, import or project the raw policy. The exact four-line
domain and 80-line emitter retain SHA-256
`38391f8c3eaadf1cd997b13fffba38dccf8a017955d3bb75b48eb3e587af7280`
and
`bcd1693a0ff5292fa826e8449162eb85e7dedcec857aa0b76ef7b9d5c3bdd387`;
their combined 84-line hash remains
`3a5aa0f6afbd361cf6e88724d0c2e4a4bb1f559b5b0a81a15affd68c455063ee`.
The new 125-line child has SHA-256
`9cb88aa5ee221e66911a1070062e7e15242aaa91585562dbfba51d4c709ee560`
and the resulting parent has SHA-256
`6a8e1b8fb5d7f05b0bfaba1d8196dab577aac30a5a65cbde37c10291321cc984`.
The recursive guard and module policy pin six child policy mentions, the sole
raw definition plus two private calls, zero parent raw names and one call to
each semantic wrapper. The raw owner move is byte-equivalent; the two parent
calls are deliberately narrowed from variant-bearing calls to semantic
wrappers without changing argument order or emitted behavior. At the Batch AC
shared checkpoint, `cargo xc` is green, the structure target passes `3/3`, the
exact CLI fixture passes `1/1`, and the exact String match ordinary-groups and
indices-groups leaves pass all `4/4` variants with every failure bucket at
zero. The semantic golden was not rerun. The bounded contract is
[`duplicate-named-group-pattern.md`](../docs/rust-rewrite/contracts/duplicate-named-group-pattern.md).

The complete global ASCII class quantifier policy now has one private
`builtins/string/global_ascii_class_quantifier.rs` owner. Its capability-free
three-variant domain and sole raw parameterized emitter moved together; the
20,671-line String parent retains exactly three semantic matcher calls and
cannot name, construct, import or project the raw width/polarity policy. The
exact five-line domain and 203-line emitter retain SHA-256
`2c70e7cfdceb62904b990196833997be1cfb643595987e38f8871942bfc49860`
and
`6a7fce3d1705ae08dbd92d96b2046445a07c2740bb314f882d7cf4f6a4320211`;
their combined 208-line hash remains
`9f97f9a45640274960049b633cd448a8090cabc8925ece61692e71c7b5470f69`.
The new 261-line child has SHA-256
`7500204a87f75dccd18aa2dc3cf10ce13642f7eb9e6c7c9cdbe943baad7d8240`
and the resulting parent has SHA-256
`80b9e6796957b0af8b819121339d5690543fdf6cd75d77795f3a796808d0efcb`.
The recursive guard and module policy pin 11 child policy mentions, the sole
raw definition plus three private calls, zero parent raw names and one call to
each semantic wrapper. The raw owner move is byte-equivalent; the three parent
calls are deliberately narrowed from variant-bearing calls to semantic
wrappers without changing argument order or emitted behavior. At the Batch AD
shared checkpoint, `cargo xc` is green, the owner and neighboring postal-code
structure targets pass `3/3` each, and the exact
`string::run_wasm_backend_succeeds_for_string_symbol_hooks_fixture` CLI witness
passes `1/1`. The exact `S15.5.4.10_A2_T3.js`, `S15.5.4.10_A2_T4.js` and
`S15.5.4.10_A2_T5.js` leaves each pass `2/2`, for `6/6` total with every failure
bucket at zero. The semantic golden was not run. The bounded contract is
[`global-ascii-class-quantifier.md`](../docs/rust-rewrite/contracts/global-ascii-class-quantifier.md).

The complete postal-code match-result-shape policy now has one private
`builtins/string/postal_code_match_result_shape.rs` owner. Its capability-free
two-variant domain and sole raw parameterized emitter moved together; the
20,307-line String parent retains exactly one global and one exec semantic call
and cannot name, construct, import or project the raw result shape. The exact
four-line domain and 357-line emitter retain SHA-256
`2c218b01e482cf283729f52db2c171b9dddd0d6fbe1d4eac5bf2fb79fdc0ac71`
and
`06fe70a126949e33e1cba69b6f349cf83d960a8e9961eecf65bbf5fc33c540d8`;
their combined 361-line hash remains
`46a993e3cd8087a333de80d918ecd59d8a80af99acf1da5edb63ed0af18b4668`.
The resulting 398-line child has SHA-256
`fc2d538c93855feb1e1f011af9d2851d42f9b6c8db6f59a15387ea93e89088b4`,
and the reduced parent has SHA-256
`683c578285995cc8b5ff585753d5de1eea2b4e7505fffd15a1bfa3647e392a4b`.
The recursive guard and module policy pin eight child policy mentions, four
uses of each variant, the sole raw definition plus two private wrapper calls,
zero parent raw names and one parent call to each semantic wrapper. The raw
owner move is byte-equivalent; the two parent calls are deliberately narrowed
from variant-bearing calls to semantic wrappers without changing argument order
or emitted behavior. At the Batch AI shared checkpoint, `cargo xc` exits zero;
`postal_code_match_result_shape_structure`,
`string_literal_replacement_scope_structure` and
`global_ascii_class_quantifier_structure` each pass `3/3`, for `9/9` total.
The exact
`string::run_wasm_backend_succeeds_for_string_match_postal_code_fixture` CLI
witness passes `1/1`. Exact `S15.5.4.10_A2_T6.js`,
`S15.5.4.10_A2_T7.js` and `S15.5.4.10_A2_T8.js` pass sloppy and strict
execution (`6/6`) with every failure bucket at zero. No semantic golden was
needed or run. Final formatter, diff, module-boundary, task-plan and 240-entry
shortcut-inventory gates are green. The bounded contract is
[`postal-code-match-result-shape.md`](../docs/rust-rewrite/contracts/postal-code-match-result-shape.md).

The complete String literal-replacement scope now has one private
`builtins/string/string_literal_replacement_scope.rs` owner. Its capability-
free two-variant domain and sole raw parameterized emitter moved together; the
19,860-line String parent retains exactly one first-occurrence and one all-
occurrences semantic call and cannot name, construct, import or project the raw
scope. The exact four-line domain and 440-line emitter retain SHA-256
`db2e26fd031d6c5ab6f0ce99ab16f58928a202f29e9bee436069cb9368b882ba`
and
`f6a34782a74376adfe9f7b622241e986c8c2bdace5f2fac4967ff4f07cf5170e`;
their combined 444-line hash remains
`0d0392432a791efc6c208fd83b41ab0061fecb60c05b93417928c355a68f2d15`.
The resulting 489-line child has SHA-256
`a85f4aba490a7792db382749383f8e1a9cc17195a902d4821d3f6225e11a1a4f`,
and the reduced parent has SHA-256
`e0c9914263803353e9fd4dbb00abc6b085bdde006930bcb8f779702f5309f54b`.
The recursive guards and module policy pin six child policy mentions, three
uses of each variant, the sole raw definition plus two private wrapper calls,
zero parent raw names and one parent call to each semantic wrapper. The raw
owner move is byte-equivalent; the two parent calls are deliberately narrowed
from scope-bearing calls to semantic wrappers without changing argument order
or emitted behavior. Batch AJ shared `cargo xc` passes; the literal-scope,
symbol-hook and RegExp flag-getter structure targets pass `10/10`, the exact
symbol-hook CLI fixture passes `1/1`, and the first-only `replace` and all-
occurrences `replaceAll` leaves pass all `4/4` sloppy/strict Wasm-AOT
executions with every failure bucket at zero. No semantic golden was needed or
run. The bounded contract is
[`string-literal-replacement-scope.md`](../docs/rust-rewrite/contracts/string-literal-replacement-scope.md).

The complete Map/WeakMap get-or-insert value-source owner now has one private
`builtins/collections/map_get_or_insert.rs` home. Its two-variant domain, all
four existing crate-visible semantic entry points and the sole raw
parameterized emitter moved together; the 6,491-line collections parent cannot
construct the value-source policy or call the raw emitter, while the four
`standard.rs` product calls remain byte-identical. The exact five-line domain
and 312-line method selection retain SHA-256
`b5db66b00f27f10e45c4b98a31220473b159564a3d292e1c9ac765a6a7ae3873`
and
`00a687c5a16c6f0c9c2ffeeeb21f714b31cc58b6dcf9d0539f5ea4a12a54acc7`;
their combined 317-line hash remains
`8666b1d64189818ecd0d108a521afdf4f0ccd9068be169436cc1c1697273d4e7`.
The new 322-line child has SHA-256
`6022280ee176b5a20373e540763ec158a5c2914ce49fb2c8720c5f72df25d7d7`
and the resulting parent has SHA-256
`8d6c436a07bc388cf950cfaf35659d65f6de068101f2382f6e384b738c44ce9e`.
The recursive guards and module policy pin ten child domain mentions, eight
qualified variants, the raw definition plus four calls, zero parent raw names,
and the four unchanged semantic methods and product calls. At the Batch AE
shared checkpoint, `cargo xc` is green; the exact Map get-or-insert, Map
collection and Set collection structure targets pass `3/3`, `4/4` and `4/4`
(`11/11` aggregate); the exact
`iterator::run_wasm_backend_preserves_map_get_or_insert_value_sources` CLI
owner passes `1/1`; and the surveyed ten-file Map/WeakMap upsert cohort passes
all `20/20` sloppy/strict executions with every failure bucket at zero. The
cohort covers direct insertion, present-key callback suppression, callback
mutation overwrite, non-function callbacks and invalid weak keys. No semantic
golden was run for Batch AE. The bounded contract remains
[`map-collection-weak-key-admission.md`](../docs/rust-rewrite/contracts/map-collection-weak-key-admission.md).

The closed runtime-RegExp table reader policy now has one private
`data/runtime_regexp_entry_kind.rs` owner. Its three-way entry-kind domain,
complete ordered domain, ABI-word projection and exhaustive SyntaxError policy
moved together; `data.rs` narrowly re-exports only `RuntimeRegExpEntryKind`,
while the raw record constants, private writer/staging domains and `StringPool`
methods remain parent-owned. The exact 56-line moved body retains SHA-256
`9f5dfffcacf72babf09f87ea8f50e419163a22ec595f236a6cd3af6175ad5957`;
the formatted 58-line child has SHA-256
`4717e0050857a07458f3509d17b993917cdce2a1569f5e14662de8c6d18cccdd`
and reduces `data.rs` from 5,220 to 5,166 lines. The dedicated ownership, ABI,
policy and caller-census structure target passes `3/3`; the exact valid and
invalid computed runtime-pattern CLI witnesses each pass `1/1`. Broader Cargo,
shared module-boundary and Test262 gates pass in the shared checkpoint. The
648-artifact semantic Wasm golden adds only the mixed-BigInt equality fixture;
all prior dumps change only the size summaries attributable to that equality
repair, with no additional RegExp ownership delta. No behavior or conformance
change is claimed for this source-equivalent move.

The closed lexical-environment storage classification now has one private
`analysis/environment_kind.rs` owner. Its nine-way `EnvironmentKind` domain,
three-way stage-A materialization projection and eight-way general
materialization projection moved together; `analysis.rs` narrowly re-exports
only `EnvironmentKind`, so the unchanged analysis and lowering callers retain
their existing paths and visibility. The exact moved and formatted 35-line
body retains SHA-256
`155383c51d50df9afac9590bbd32b9d9a73958dd358c258d78102b8b12ee25bf`
and reduces `analysis.rs` from 6,577 to 6,544 lines. The dedicated ownership,
closed-domain and caller-census structure target passes `3/3`; six focused
analysis environment-planning tests and the direct with-object GetValue CLI
witness each pass `1/1`. Broader Cargo, workspace policy, Test262 and
Wasm-golden gates pass in the shared checkpoint: all 648 artifacts remain
present, and this source-equivalent move adds no import, export, runtime-root,
helper-count, memory or data-segment delta. The combined batch changes 126 dump
summaries through the shared unary-numeric repair recorded under T20; no
behavior or conformance change is claimed for this ownership move.

The closed module-request phase domain now has one private
`modules/import_phase.rs` owner at the module-subsystem boundary. Its three
ordered phases, default evaluation phase, diagnostic-name projection and
exhaustive Boa-AST projection moved together; `modules/mod.rs` narrowly
re-exports `ImportPhaseIr`, preserving both the public crate-root exposure and
the unchanged `pub(super)` reach from sibling `record` and `dynamic` owners.
The exact 31-line moved body retains SHA-256
`29fa6efd4aa1d23f627376a6860d2fc9e5011e06dce5a7269cd7a84dc8ea6195`;
the formatted 33-line child has SHA-256
`907b8cb6bbed0108d2fbcf8b28843028d423a84beee17633fd0da0f000a3083c`
and reduces `modules/record.rs` from 2,604 to 2,572 lines. The dedicated
ownership, visibility, closed-domain and caller-census structure target passes
`3/3`; four focused IR phase tests and the filesystem-resolution engine witness
each pass `1/1`. The shared workspace compile and every repository policy gate
pass. All 648 Wasm-golden artifacts remain present; this move adds no emitted
delta beyond the shared Iterator realm repair recorded under T06/T15. A broader
pinned Test262 module run was not performed, and no behavior or conformance
change is claimed for this source-equivalent move.

The closed module evaluation and runtime-materialization policy now has one
private `modules/evaluation_mode.rs` owner. The public three-way evaluation
domain and default, the private two-way materialization domain, diagnostic
projection and exhaustive `NotEvaluated -> None` crossing moved together;
`modules/mod.rs` narrowly re-exports only `ModuleEvaluationModeIr`. The graph,
linker and namespace owners import `ModuleMaterializationModeIr` directly from
its private owner rather than through a graph compatibility re-export. After
normalizing only the child-relative `pub(super)` spelling required to preserve
the projection's module-subsystem visibility, the exact 59-line body retains
SHA-256
`aeda916fced4931c436d7ad763f786217a335935b3a6e797b09d111287abd644`;
the formatted child has SHA-256
`d1e05690549bd5aab9ebe2680e1f7fc945ae9b972aac39ca1c333586b396a455`
and reduces `modules/graph.rs` from 3,222 to 3,163 lines. The dedicated
ownership, visibility, closed-domain and caller-census structure target passes
`3/3`; seven focused eager, deferred and source-only graph/link/namespace tests
each pass `1/1`. The shared workspace compile and every repository policy gate
pass. All 648 Wasm-golden artifacts remain present; this move adds no emitted
delta beyond the shared Iterator realm repair recorded under T06/T15. A broader
pinned Test262 module run was not performed, and no behavior or conformance
change is claimed for this ownership move.

The complete `ResolveExport` result domain now has one private
`modules/resolved_binding.rs` owner. The three-way `ModuleBindingNameIr` and
three-way `ResolvedBindingIr` domains moved together with unchanged public
visibility, variants and fields; `modules/mod.rs` narrowly re-exports both
public types, while `ModuleLinkErrorIr` and every resolution algorithm remain
in `graph.rs`. The exact 35-line moved body retains SHA-256
`4863d4513360a57c6f94340a0f37604ccfa0358f12219088795ea292c0a767dd`;
the 39-line child has SHA-256
`af9dae260d97f5d7a0ba16e93adcfdc864bacc1c76f1ec032d7f11193fc7f5c2`
and reduces `modules/graph.rs` from 3,163 to 3,127 lines. The dedicated private
ownership, closed-domain and caller-census structure target passes `3/3`; all
51 graph tests pass, and the exact star-re-export linker and nested-namespace
consumer witnesses each pass `1/1`. The shared workspace compile and every
repository policy gate pass. All 648 Wasm-golden artifacts remain present; this
move adds no emitted delta beyond the shared Iterator realm repair recorded
under T06/T15. A broader pinned Test262 module run was not performed, and no
behavior or conformance change is claimed for this source-equivalent ownership
move.

The opaque host-normalized module identity now has one private
`modules/module_key.rs` owner. Its private `String` storage, sole public
`from_host` constructor and read-only `as_str` projection moved together;
`modules/mod.rs` narrowly re-exports `ModuleKey`, so all IR and engine callers
retain their public facade paths and no `From<String>` conversion exists. The
exact 28-line moved body retains SHA-256
`4d63703a82ee07f8103175da1c3268946e1a059dc3e8d5253cbf35874bf98ddd`;
after qualifying the intra-doc link at its new owner, the 28-line child has
SHA-256
`1e5aa8ad05f5cfa7f05b53460f97e0ddc39ac588ae65abeabc948c399af97f08`
and reduces `modules/graph.rs` from 3,127 to 3,098 lines. The dedicated private
ownership, opaque-construction and IR/engine caller-census structure target
passes `3/3`; the contradictory-resolution, inconsistent-load, canonical-file
identity and phase-free filesystem-resolution witnesses each pass `1/1`. The
shared workspace compile and every repository policy gate pass, and all 648
Wasm-golden artifacts are byte-identical to the post-Iterator baseline. A
broader pinned Test262 module run was not performed; no behavior or conformance
change is claimed for this source-equivalent ownership move.

The complete module-link rejection domain now has one private
`modules/link_error.rs` owner. Its eight public variants and exhaustive
`code`, `message` and `to_diagnostic` projections moved together;
`modules/mod.rs` narrowly re-exports `ModuleLinkErrorIr`, while every graph
algorithm and producer remains in `graph.rs`. The error owner imports
`ModuleKey` from its private sibling rather than depending back on the graph
owner. The exact 153-line moved body retains SHA-256
`b73f663e27c87f1258076db05ce42e8e9e65ea69e6265c24935f46ccf217101d`;
after qualifying the intra-doc evaluation-mode link at its new owner, the
159-line child has SHA-256
`8f012bd054aa6502a1d4c121f3268c18d84c1335e9b032974a62304e24697450`
and reduces `modules/graph.rs` from 3,098 to 2,944 lines. The dedicated private
ownership, closed-domain, exhaustive-projection and caller-census structure
target passes `4/4`, and the retargeted resolved-binding owner guard passes
`3/3`. All 51 graph tests pass; the exact duplicate-export early-error and
unresolved-import lowering witnesses each pass `1/1`. The shared workspace
compile and every repository policy gate pass, and all 648 Wasm-golden
artifacts are byte-identical to the post-Iterator baseline. A broader pinned
Test262 module run was not performed; no behavior or conformance change is
claimed for this source-equivalent ownership move.

The complete linked-module unit record now has one private
`modules/module_unit.rs` owner. Its ten public fields moved together without
changing visibility or field order; `modules/mod.rs` narrowly re-exports
`ModuleUnitIr`, while source parsing, graph construction and every graph
algorithm remain in `graph.rs`. The exact 30-line moved body retains SHA-256
`75c59467e8f75671820ab60321319cfca75f385867f6658b80868e68eb9b46c9`;
the 36-line child has SHA-256
`ab895fa1e6002c1b0953130024a5ad8bb274cc901a97641c3ca63ec80b4fd963`
and reduces `modules/graph.rs` from 2,944 to 2,913 lines. The dedicated private
ownership, exact-field and caller-census structure target passes `3/3`; all 51
graph tests pass, and the exact star-re-export linker and nested-namespace
consumer witnesses each pass `1/1`. The shared workspace compile and every
repository policy gate pass, and all 648 Wasm-golden artifacts are
byte-identical to the post-Iterator baseline. A broader pinned Test262 module
run was not performed; no behavior or conformance change is claimed for this
source-equivalent ownership move.

The complete parse-once loaded-source and host-resolution closure family now
has one private `modules/loaded_sources.rs` owner. Public `ModuleSourceIr` and
`ModuleGraphSources`, private `ModuleParse`, all eight source operations, the
three-field closure record and its `single` constructor moved together;
`modules/mod.rs` narrowly re-exports only the two public types. The anonymous
one-node key moved separately to its semantic `module_key.rs` owner, retaining
the exact five-line block and SHA-256
`fa47c27eae037ea7252a5554a80cfd778f8d81d14b70acb9efc3fcb2bb6e4914`.
The exact 168-line loaded-source body before relocation had SHA-256
`e63a4f9fa3c2af9fe3d7fa361aeca61b9db1cd126445943f3d1ffd68219f6b80`;
the relocated body changes only `ModuleParse` and `ModuleSourceIr::parse` to
sibling-only `pub(super)` visibility so graph construction can retain its
existing exhaustive match without exposing either outside the module
subsystem. That 168-line body has SHA-256
`8bbca39b7c7c9e5aaa8713ee0094be22e22bb8110a0956fa1b6edc3fbaf5b428`;
the formatted 175-line child has SHA-256
`9301522ff6987d03c3d50441cbf063151629551863e1dfb9f684c3fb95daeae8`
and reduces `modules/graph.rs` from 2,913 to 2,742 lines while leaving graph
construction, linking and resolution algorithms there. The dedicated
ownership, visibility, exact-record and caller-census structure target passes
`4/4`; the retargeted module-key and module-unit guards each pass `3/3`. The
exact canonical-resolution and inconsistent-load graph witnesses pass, as do
the transitive-closure, phase-free filesystem-resolution and one-node module
engine witnesses. The shared workspace compile and every repository policy
gate pass, and all 648 Wasm-golden artifacts are byte-identical to the
post-Iterator baseline. A broader pinned Test262 module run was not performed;
no behavior or conformance change is claimed for this ownership move.

The complete loaded-closure graph-construction algorithm now has one private
`modules/graph_build.rs` owner. The existing `pub(crate) build_graph` entry,
parse-once dispatch, sole unit-ID mint, unit construction, host-resolution
remap, inconsistent-load detection and no-winner inconsistent-resolution
handling moved together; `modules/mod.rs` narrowly re-exports the crate-only
entry, while `ModuleGraphIr`, every query/resolution method and post-build
linking remain in `graph.rs`. The exact 129-line moved body retains SHA-256
`d2973586812d4d6d1b88570cb1726558e7d050c047d4cb1303736574b8a88472`;
the formatted 141-line child has SHA-256
`cf30ab0f8ce01d166fb122acc17ffd9471d22d392ff78c94b192f2a59af26ecd`
and reduces `modules/graph.rs` from 2,742 to 2,612 lines, including removal of
its obsolete construction-only imports. The dedicated ownership, invariant
and caller-census structure target passes `4/4`; the five retargeted module
domain structure targets pass `17/17`. Five exact graph regressions covering
canonical request attributes, duplicate-key identity, no-winner conflicting
resolutions, typed rejected-parse diagnostics and the unit-ID cap each pass
`1/1`, as does the exact engine module-goal witness. Broad Cargo, golden and
Test262 gates were deferred to the shared verification checkpoint; no behavior
or conformance change is claimed for this ownership move.

The complete host-request and export-resolution method family now has one
private `modules/graph_resolution.rs` owner. The four public inherent methods
`resolve_request`, `resolve_request_key`, `exported_names` and `resolve_export`
moved with their two private recursive helpers and the trailing binding-name
ownership comment; callers retain the same `ModuleGraphIr` API without a
re-export, wrapper or visibility widening, and `graph.rs` has no dependency
back on the child. The exact original 173-line body had SHA-256
`3e554a84e67f6414feb9388ff376cf0af0879fe139c51eb428a479a54aab0f76`;
after qualifying only the relocated `ModuleKey` intra-doc link, that body has
SHA-256
`e3547ec8a79e4752f5893befd3ba60ba5a9d61f3eb96da677e27f3ddf6892c82`.
The formatted 183-line child has SHA-256
`90e39b4131fd13ab247885dd80ef8de7c44c788796053afc6905668020d8c341`
and reduces `modules/graph.rs` from 2,612 to 2,438 lines, including removal of
its obsolete `push_unique_name` import and formatting the resulting impl
boundary. The dedicated ownership, visibility, invariant and caller-census
structure target passes `4/4`; the five retargeted domain structure targets
pass `17/17`. Seven exact graph resolution regressions, the exact namespace
star-default witness and the exact downstream star-re-export linker witness
each pass `1/1`. Broad compile, golden, policy and Test262 gates were deferred
to the shared checkpoint; no behavior or conformance change is claimed for
this ownership move.

The runtime module-materialization query seam now has one private
`modules/graph_materialization.rs` owner. The two sibling-only inherent methods
`materialization_mode` and `materialized_units` moved together, while the
public `evaluation_mode` query remains with the core graph algorithms in
`graph.rs`; no re-export, wrapper or visibility widening was added. The exact
31-line moved body retains SHA-256
`f754027708c47a28e1ae119b6997ec7709bd901b2bea5f1a486023eaec4ace3d`;
the 40-line child has SHA-256
`632468ececdb23b1801f3a36c638d17dde285aa7097b843f1ee1127eb0318da9`
and reduces `modules/graph.rs` from 2,438 to 2,406 lines. The retargeted
ownership, visibility and caller-census structure target passes `3/3`, pinning
the private materialization type to the child and its unchanged linker and
namespace consumers; the retargeted module-unit caller census also passes
`3/3`. Broad compile, golden, policy and Test262 gates remain deferred to the
shared checkpoint; no behavior or conformance change is claimed for this
source-equivalent move.

The complete `InnerModuleEvaluation` ordering algorithm now has one private
`modules/graph_evaluation_order.rs` owner. The iterative Tarjan traversal, its
closed local state record and two-way work-step domain moved together; the
parent imports only the sibling-private `compute_evaluation_order` entry, and
only `evaluation_dependencies_of` widened to sibling-only visibility for that
unchanged caller. The exact original 120-line body had SHA-256
`17d53c44da2e49df0d233536a857cb7a95a2cb77a6ae081ea06e35b528708a7b`;
the 125-line child has SHA-256
`a822af7a705a18a5bc781d35acc5679dab117cc33c250b1382c4059f2e652669`
and reduces `modules/graph.rs` from 2,406 to 2,286 lines. The dedicated private
ownership, closed-state and dependency/component invariant structure target
passes `3/3`; the retargeted evaluation-mode and module-unit structure targets
each pass `3/3`. The exact cycle-contiguity, dependency-before-importer and
defer-occurrence ordering regressions each pass `1/1`. Broad compile, golden,
policy and Test262 gates remain deferred to the shared checkpoint; no behavior
or conformance change is claimed for this source-equivalent move.

Evaluation-mode classification and unsupported phase reporting now have one
private `modules/graph_evaluation_classification.rs` owner. The fixed-point
reachability classifier and its paired defer-policy reporter moved together;
the graph retains link orchestration and imports only the two sibling-private
entry points, with no re-export or wider visibility. The exact original
167-line block had SHA-256
`831f683316ab22d2529b6613896cf8a8b3796dfb6aa3ab093dfff6f7c6b8b097`;
the 179-line child has SHA-256
`df5ba3a5c0d011c2febb1913b7417e583d95af6b925b0a21ee8063d441858a0d`
and reduces `modules/graph.rs` from 2,286 to 2,120 lines. The dedicated private
ownership, phase fixed-point and unsupported-defer-policy structure target
passes `3/3`; the retargeted evaluation-mode, link-error and graph-resolution
structure targets pass `3/3`, `4/4` and `4/4`. The defer-only, source-only,
non-evaluation-cycle and deferred-top-level-await regressions each pass `1/1`.
Broad compile, golden, policy and Test262 gates remain deferred to the shared
checkpoint; no behavior or conformance change is claimed for this
source-equivalent move.

Async-module propagation and pending-dependency queries now have one private
`modules/graph_async_evaluation.rs` inherent-impl owner. The public
`async_evaluation` and `pending_async_dependencies` signatures remain
unchanged, while their component propagation and distinct-component counting
moved together with only the evaluation-mode, graph, unit-id and `BTreeSet`
imports they use; no re-export, wrapper or visibility change was added. The
exact original 86-line block has SHA-256
`2fddf867c82f38205111d9ef6a8d248945f4c1ebd4326074c52a8227631e08ca`;
the 96-line child has SHA-256
`4d5a62278756148d4a1a55e8b6f180f2a15308d126b5f99b9c21974edf013331`
and reduces `modules/graph.rs` from 2,120 to 2,034 lines. The dedicated private
ownership, eager-component propagation and distinct pending-component
structure target passes `3/3`; the retargeted evaluation-order,
evaluation-mode and module-unit structure targets each pass `3/3`. The
synchronous graph, transitive importer, independent sibling, asynchronous
cycle, source-only top-level-await and linked asynchronous-wrapper regressions
each pass `1/1`. Broad compile, golden, policy and Test262 gates remain
deferred to the shared checkpoint; no behavior or conformance change is
claimed for this source-equivalent move.

The `ModuleGraphIr` unit-test suite now has one private adjacent
`modules/graph_tests.rs` owner, selected by an explicit path from the existing
`graph::tests` submodule so every test keeps the same Rust namespace. The
original nested body was 1,690 lines with raw SHA-256
`3564efcd173154ee5aeb0a415c0f5302cfab90a3ccb43bc03f1227041633d1be`;
removing only its four-space module indentation produced SHA-256
`0de4b7d4ab76d43d6c7e6322f12d04b41282038793a2632832abb3599256e7d3`.
Rustfmt then joined one newly unnested function signature, leaving the
1,689-line child at SHA-256
`5d25df1c9f209ed651f568725b1ac0d92b2da073a47b40496bfedf3ee8344151`
and reducing `modules/graph.rs` from 2,034 to 344 lines. No test logic or
production behavior changed. The dedicated path, namespace, helper and exact
51-test census target passes `2/2`; the eight retargeted ownership/caller
structure targets pass `28/28`, and all moved `modules::graph::tests` pass
`51/51`. Broad workspace, golden, policy and Test262 gates remain deferred to
the shared checkpoint; no behavior or conformance change is claimed for this
test-source ownership move.

The shared byte-identity checkpoint now covers the complete module-graph
ownership batch. Both pre- and post-refactor captures pass `2/2` and contain
646 fixture rows (648 files including the manifest and largest-function
report). The recursive diff between
`target/golden/post-graph-completion-realm-domains` and
`target/golden/post-graph-private-owners` is empty. Together with the focused
ownership and behavior tests above, this proves that the materialization,
Tarjan ordering, evaluation classification, async propagation and adjacent
test-owner moves changed no emitted Wasm byte.

The complete ECMAScript `TrimString` emitter now has one private
`operations/string_trim.rs` owner. Private `EcmaTrimMode`, the three existing
`pub(crate)` named wrappers and their owner-private raw emitter moved together;
`operations.rs` retains the shared UTF-8 whitespace scan helpers and the
String-to-BigInt caller, with no re-export, facade wrapper or visibility
change. The extraction reduces `operations.rs` from 12,193 to 12,020 lines;
the formatted child is 177 lines. The retargeted private ownership, exhaustive
mode projection, exact helper/caller census and trim-order structure target
passes `2/2`, and the existing Annex B/String trim CLI fixture passes `1/1`.
Broad workspace, golden, policy and Test262 gates remain deferred; no behavior
or conformance change is claimed for this source-equivalent move.

The complete `for-await-of` well-known-symbol acquisition boundary now has one
private `control_flow/for_await_iterator_symbol.rs` owner. The closed
`ForAwaitIteratorSymbol` domain, its exhaustive name projection and the sole
symbol-key read emitter moved together; sibling-only visibility preserves
their former effective scope, while `compile_async_for_of_iterator` retains
the two unchanged async-first/fallback call sites. The extraction reduces
`control_flow.rs` from 13,554 to 13,495 lines; the formatted child is 65 lines.
The retargeted private ownership, exhaustive projection, typed-key and caller-
order structure target passes `3/3`, and the primitive-String async-iterator
preference behavior passes `1/1`. Byte-equivalence risk is low but unmeasured:
the instruction-emitting body and call order are source-equivalent, but no
golden capture was run. Broad workspace, policy and Test262 gates remain
deferred; no behavior or conformance change is claimed.

The complete current base-10 Number-to-string emitter now has one private
`operations/number_to_string.rs` owner. Its sole existing `pub(crate)`
`emit_number_to_string_payload` API and all 407 implementation lines moved
together; the shared decimal digit scans and writers remain in `operations.rs`
for their other callers. The implementation block has the same pre/post
SHA-256, `0726c7f6449023c6f7b503fffb3a29cac12a771a0f1f81e03fb9fe74c6809ae2`.
The retargeted ownership and unsafe-integral invariant target passes `4/4`, and
the existing dynamic shortest-integral CLI regression passes `1/1`.
Byte-equivalence risk is low but unmeasured: the instruction-emitting body is
byte-for-source identical, but no golden capture was run. Broad workspace,
policy and Test262 gates remain deferred; no formatting or conformance change
is claimed for this preparatory ownership move.

The eight optional host import function indices now cross from `emit.rs` into
`FunctionMetaRegistry` through one non-copyable
`HostImportFunctionIndices` authority with distinct, non-derived role types.
The sole producer can no longer transpose two raw `Option<u32>` positions while
continuing to compile, and the registry stores the authority intact. Its eight
existing named getters are the only raw-index projections. The robust
Rust-lexical `host_import_function_indices_structure` target owns the exact
domain, recursive census, one complete producer, intact storage, and sole
projections; the focused verification record and explicit nonclaims live in
[`host-import-function-indices-authority.md`](../docs/rust-rewrite/contracts/host-import-function-indices-authority.md).

The complete consume-once Wasm module package lifecycle now has one private
`module/compiled_module_package.rs` owner. Type and global section construction,
runtime-root finalization, main compilation, remaining-body append, canonical
section assembly and all three compile-time ownership gates moved together;
`module.rs` narrowly re-exports only `ModuleTypeRegistry`,
`ModuleGlobalSectionBuilder` and `ModuleAssemblySections`. The finalized and
compiled intermediate package states have no re-export from the private child,
so unrelated modules cannot name or construct them. The 292-line child has
SHA-256
`e6c8aab33f1e616bfbf9ae00a7a154226885c3b1520a77c3a45694b4b6e2aaef`
and reduces `module.rs` from 2,450 to 2,165 lines. The dedicated private-owner,
narrow-re-export, consume-once, canonical-order and caller-census structure
target passes `3/3`; the retargeted inline package witness passes `1/1`, and the
module-boundary audit, child/test formatter, diff and task-plan checks pass.
Broad workspace compilation, Wasm golden, runtime and
Test262 gates remain deferred to the shared checkpoint; no behavior or
conformance change is claimed for this source-equivalent ownership move.

The four `Temporal.ZonedDateTime.prototype` arithmetic/difference catalog arms
now enter their family owner through fixed `add`, `subtract`, `until` and
`since` methods. The private, non-derived
`ZonedDateTimeArithmetic::{Add, Subtract}` and
`ZonedDateTimeDifference::{Until, Since}` domains and both raw emitters can no
longer escape through `builtins/mod.rs` or be constructed by `standard.rs`.

The exact former 36-line policy selection retains reconstructed SHA-256
`82f3f206759543894d9ec36a278938c4a17e3f0db2602df13f9c9e7c1f1756a0`.
The visibility-normalized 122-line arithmetic and 217-line difference emitters
retain SHA-256
`0df4c7b1b768c8520b30f505c8d5c5f6e18d1a8dbee0dff7b08149f2aa3bbde2`
and
`8c95229bd602e45445a7c6ad5e2a89b3d120b903be74b73ac185782859d73cdf`.
The focused
[`direction-dispatch contract`](../docs/rust-rewrite/contracts/temporal-zoned-date-time-direction-dispatch.md)
target passes `3/3`, four neighboring structure targets pass `15/15`, and the
exact arithmetic/era and difference-default CLI controls each pass `1/1`.
The capped workspace/all-target, formatting, module-boundary, task-plan and
shortcut gates are green. This adds no new Temporal behavior or conformance
claim; it is source-equivalent hardening.

The five `Intl.Locale` string-returning routes now cross a fixed semantic
boundary: the closed slot domain and raw emitter are private, the non-derived
domain's projections borrow it, and the shared catalog can call only five
fixed entries. The focused
[`string-slot dispatch contract`](../docs/rust-rewrite/contracts/intl-locale-string-slot-dispatch.md)
records exact original domain and raw-emitter SHA-256 witnesses
`00486705af5ad3a89c1386f4ca8b3088d5531ca676a582aa643ca90bca658d6a`
and
`4b346dcd2c819c503603ed7c08842e577d4b893dc98aa1f33c5f7d2c864cd134`.
The contract target passes `4/4`, two neighboring structure targets pass
`8/8`, and the exact canonical-tag roles CLI control passes `1/1`. Formatting,
module-boundary, task-plan and shortcut gates are green. This is
source-equivalent hardening with no new Intl behavior or conformance claim.

The generic Array `sort`/`toSorted` output selector and 604-line raw algorithm
are now private behind two fixed family entries. The standard catalog cannot
name `ArraySortOutput`, select a variant or call the raw algorithm. The focused
[`Array sort output contract`](../docs/rust-rewrite/contracts/array-sort-output.md)
records exact original domain and raw-algorithm SHA-256 witnesses
`1745b093aab4e0643c08de0b1d402f3770ef5a9618635ae7b31ec318a8c74c4c`
and
`aa8c4c988b2c5e64568cfc9f4a294c98a32144af941450cf59ac882948afbf25`.
The output target passes `4/4`, its dispatch-owner neighbor passes `5/5`, and
the three exact Array sort CLI controls pass `3/3`. Formatting,
module-boundary, task-plan and shortcut gates are green. This
source-equivalent hardening has no new Array behavior or conformance claim.

The capability-free `ArrayCallbackReceiverKind` and 442-line raw shared
`forEach` compiler are now private to `builtins/array.rs`. Standard dispatch can
call only fixed Array and TypedArray `forEach` entries, matching the four fixed
reducer entries. The updated
[`callback receiver-kind contract`](../docs/rust-rewrite/contracts/array-callback-receiver-kind.md)
records exact original SHA-256 witnesses
`c073b0a9449fae68b12f82e43fc0bf7dc52a0a0bc98b1a6eb2bf6d5b0bce3ea1`
and
`ea047de76bef8b4c5fbc8eb440c42329e7693feecf848cef753011cf2a541c26`.
The callback and direction targets pass `4/4` each, and the exact
resizable-TypedArray generic `forEach` CLI control passes `1/1`. Formatting,
module-boundary, task-plan and shortcut gates are green. This
source-equivalent hardening has no new Array behavior or conformance claim.

The private `builtins/array/find_via_predicate.rs` owner now also hides its
capability-free `FindViaPredicateKind` and both raw family compilers. Standard
dispatch can call only eight fixed Array/TypedArray find-family entries. The
capability-free `FindDirection` and capability-free `FindProjection` remain
private borrowed projections of that single kind authority.
updated
[`find-via-predicate contract`](../docs/rust-rewrite/contracts/array-find-via-predicate.md)
records exact original SHA-256 witnesses
`3989f2ebe1ce925d23b20d4e06eb35f00e1e840f7509b8226b9b425a639c4e5c`,
`40be1db2dd3ccb1f35a9e022061f4fb23a8adc8fac8e446f06fdb93879b3e92d`
and
`b71e9cfcea61c77cdbef9aeb68917c65e1e54ab1bbe735e49a4175d82f00673e`.
The structure target passes `5/5`, and the exact forward Array, reverse Array
and TypedArray controls pass `3/3`. Formatting, module-boundary, task-plan and
shortcut gates are green. This source-equivalent hardening has no new Array behavior
or conformance claim.

The complete non-ASCII `TrimString` whitespace table now shares the private
`operations/string_trim.rs` owner with its only forward and backward consumers.
The owner-private `ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8` no longer escapes from
the broad builtin namespace. The updated
[`ECMAScript trim-mode contract`](../docs/rust-rewrite/contracts/ecmascript-trim-mode.md)
records the exact original 21-line SHA-256 witness
`3b3f4cb67213c7881b83d193a979ff4ae654805c1e7c783c473d781eb5395bd8`.
The strengthened structure target passes `3/3`; the exact all-whitespace
`trimStart` and `trimEnd` leaves pass all four Wasm-AOT executions with every
failure bucket at zero, and `cargo xc` is green.
This source-equivalent ownership closure has no new String behavior or
conformance claim.

Three crate-visible builtin emitters with no backend call site are deleted in
full: Date time-within-day, obsolete SharedArrayBuffer rejection, and the
zero-start ASCII-word iterator wrapper. Their live neighboring primitives and
direct start-index-aware regexp path remain. The focused
[`obsolete builtin emitter removal contract`](../docs/rust-rewrite/contracts/obsolete-builtin-emitter-removal.md)
records the exact original SHA-256 witnesses
`e69fe8ffc2517b72e18a85800ae0556736ede49cf01cd12c29a563008d7d3767`,
`df9bc99017d1ab0080f962469ea29e263e3d59c15ba720e2eacfe099dacca563`
and
`934f091b5e4b1e04057b0a56b51a7897dc1c2537057b748d4e4f01f411198471`.
The focused absence target passes `3/3`; the active ASCII-word match-all CLI
fixture passes `1/1`; the exact Date plus SharedArrayBuffer leaves pass all four
Wasm-AOT executions with every failure bucket at zero; and `cargo xc` is green
without the three corresponding dead-code diagnostics.
This reachability closure has no new JavaScript behavior or conformance claim.

Five uncalled core backend APIs are deleted in full: String's static numeric
projection, the 32-bit and 16-bit buffer-memory argument wrappers, the
TypeError-only realm-prototype store, and the standard-constructor prototype
lookup. Their live neighboring generic or width-specific paths remain. The
focused
[`obsolete core backend API removal contract`](../docs/rust-rewrite/contracts/obsolete-core-backend-api-removal.md)
records the exact original SHA-256 witnesses
`f3bc9cf6043c6d927bf0d51a9f600cf28f1c2e86291f623c47ba9406b35bc0c7`,
`6af38235bb977a2b2673f8424ea1bfa1b4fb4b958df5f4a06b9490bb8e270b48`,
`7860a2a85f440682f332a7be0a6bee8d1a7f92eaa2de78025329ae026dd699fb`
and
`ceac7d89945f7aeaeff7721ff47901b0c9980405f35d05aa33396ec25aab608b`.
The focused absence target passes `3/3`; the exact String substring plus
ArrayBuffer transfer leaves pass all four Wasm-AOT executions with every
failure bucket at zero; and `cargo xc` is green without the five corresponding
dead-code diagnostics.
This reachability closure has no new JavaScript behavior or conformance claim.

The unreachable planning analysis island is deleted in full: unused call and
function-table walks, unused environment/function-heap roots, the uncalled
large deferred-builtin classifier, the unused registry iterator, and the
unread Wasm metadata copy of `super_constructor_target`. The live IR field and
all active planning authorities remain. The focused
[`obsolete planning analysis removal contract`](../docs/rust-rewrite/contracts/obsolete-planning-analysis-removal.md)
records the exact original SHA-256 witnesses
`be7c5a1e0e9fe6fefc2c8a5db187f192c1e5f55764eeee29d940dc26ad94a177`,
`17b31c1feb5348b2f1e2dc0cdf24a618519ddebf55d39105288c6b898d8fb88f`,
`4050159124cc94d7b65ee22e7bd566c9b600bf5bbb55b5815ca6f4ef537e3ea8`,
`34679124b57a9e0716f4a604d29f5383ffd4c91ce3d1fdb8aa509c65951df238`
and
`c99ecf4f2aca412f218f8e5a6be29cacb0fe51d34635533114bc0d74e698bba5`.
At the Batch BU checkpoint, `cargo xc` is green without the corresponding
planning dead-code diagnostics, the focused absence target passes `3/3`, and
the retained regexp-literal bootstrap-root unit passes `1/1`.
This reachability closure has no new JavaScript behavior or conformance claim.

The shadowed property-read dispatch arms are deleted in full: Dynamic now
returns through its single outer match arm, String has one exhaustive key
match, and the compile census no longer carries unused broad imports for
`RealmRecordLocal` or `NativeErrorKind`. The focused
[`shadowed property-read arm removal contract`](../docs/rust-rewrite/contracts/shadowed-property-read-arm-removal.md)
records the exact original SHA-256 witnesses
`68165b09f3c33dde58a972643a8dd69cf970bca44fff30af6baa600ad1063f76`,
`ed859523f2e4b103fb5b069adf5931321c934efd3ef99f6e6e98b359e63e6c87`,
`763d09a61590ffcf1b4afeac60d93302e8094d3bab928f822518150cd87a02f1`
and
`8abe81e3220990ad0a59d373e364761cc6f47981f226475794efe66ddc9a324c`.
At the Batch BV checkpoint, `cargo xc` is green with no `lila-aot-wasm`
warnings, the focused absence target passes `3/3`, and the three retained
outlined dynamic-property-read module validations pass `3/3`.
This reachability closure has no new JavaScript behavior or conformance claim.

The disconnected IR-lowering specializations are deleted in full: obsolete
generator-expression array projections, the String-valued static-generator
fold, private-brand and operand-shape projections, two unread
`GeneratedFunctionOutput` fields, and their now-unused broad Regex import.
Live generator, iterator, String-folding, class-output and heap-shape
authorities remain. The focused
[`obsolete lowering specialization removal contract`](../docs/rust-rewrite/contracts/obsolete-lowering-specialization-removal.md)
records the exact original SHA-256 witnesses
`5fa129a28e54d16a8d17a6d160906b0c4e018205424be6173ed5571d2fadf9b2`,
`8ee9816ca0c120d3d1513ac8b831c3a0783f39b7db85b431c12ee89502a1c5a9`,
`02dbdf1e8f7aa05681dffa2ef505eade66622569ac53cde246e368a25ae737ff`,
`1320eddae0b215dfd5cc7f4f36bdaae2b85aa0738dea59bdbd6a4835a6faf9d8`,
`092a89c3965593b028c642d690c41a3c5bce5089396c747c49cdbf30c3a7d518`,
`55261d8d96ceb75dbbece9833835c68d4a56c695ad8f3bbce3288145cb6efeba`,
`92e5b6db98afaf7bb5c97c1db79246f5b3d5ea40408b15b0f48d82d65c5958e3`,
`326e77a61a4c63276a206c7eb836621ba4b8bfb3f1e3bb44ce7ca914904abef6`,
`02a744bb3487bffa56d2fc11df81f51d862a0ee57da8aa56c88af443e5465530`
and
`c78546c57688c1d6cbb796baf74584b5c4bc61c448ff6541212aed7efa88d974`.
At the Batch BW checkpoint, `cargo xc` is green with no `lila-ir` or
`lila-aot-wasm` warnings, the focused absence target passes `3/3`, and the
retained generator-expression, class-field and regexp-literal units pass
`3/3`.
This reachability closure has no new JavaScript behavior or conformance claim.

The write-never static-generator caches are deleted in full. Their two maps
were constructed empty and had no insertion or non-empty replacement; all
flow-fact propagation, heap visits, invalidation, declaration filtering and
consumer branches therefore represented an unreachable state. Ordinary
expression lowering now appears directly where the cache miss was the only
reachable branch. Live generator-expression overrides, numeric generator-
declaration parsing, object-iterator folding, iterator-binding values and
array-literal folding remain. The focused
[`obsolete static-generator cache removal contract`](../docs/rust-rewrite/contracts/obsolete-static-generator-cache-removal.md)
records the exact original SHA-256 witnesses
`8043d5ff10f4b61f90d5caea850ee1f648d81a7c5bfd413715fd1776194bd27c`,
`51ca4e5119307e3df723701e54632dc8f37cfe0f231ea0bd6401c10e7d1bd0d2`,
`1f6bb5a929cb2250a07ba4d1deb96379788633d9da460d1e95e43c9d61360c1e`,
`455ea8b701e57fe6497169d2cfcf94bae1f804a6f21e37c7f47e044ea3eba1bb`
and
`8291e01437badcde47d9d2d412b4b68fb4fb6025daa5d26459ab55b70b6dae79`.
At the Batch BY checkpoint, `cargo check -p lila-ir` is green without new
project warnings, the focused absence target passes `3/3`, and the retained
iterator-fallback and generator-suspension units pass `3/3`.
This state-invariant closure has no new JavaScript behavior or conformance
claim.

The obsolete static-generator backend protocol is deleted after the cache
closure removed its only ordinary IR producer. The two IR names, unconditional
string registrations, special array-iterator method variant, marker writer,
marker reader and its three iterator-close consumers are gone. This also fixes
an accidental source boundary: JavaScript could forge the marker as an ordinary
own property and force an Array iterator's private done slot during close. The
[`obsolete static-generator cache removal contract`](../docs/rust-rewrite/contracts/obsolete-static-generator-cache-removal.md)
now records the backend closure; a source-wide absence target and a Wasm CLI
property-spoof regression pin it without adding a conformance-count claim. At
the Batch BZ checkpoint, the focused structure targets pass `6/6`, the new and
neighboring CLI controls pass `3/3`, `cargo check -p lila-aot-wasm` is green,
and the repository policy gates are green with 240 exact shortcut entries.

The complete activation-backed plain-async synchronous `for-of` emitter now has
one private `control_flow/async_function_for_of_iterator.rs` implementation
owner. The parent keeps its single semantic dispatch call, while the existing
crate-visible method remains available to the sibling emission-site ledger.
The parent neither imports nor re-exports the private child. The 420-line
method selection moved from parent lines 8,411-8,830 to child lines 4-423 and
retains raw pre/post SHA-256
`504ebf1b2e551d7ca52161fd9fa64716d97961a45e0600a8be192d25d6aba422`.
The 424-line child reduces the current `control_flow.rs` parent to 13,220 lines.
Reviewers can refresh the selected-source witness with:

```sh
sed -n '4,423p' \
  crates/lila-aot-wasm/src/control_flow/async_function_for_of_iterator.rs \
  | sha256sum
```

The recursive ownership guard pins the private module declaration, sole child
method definition, zero parent definitions, one parent dispatch call and one
emission-site ledger reference. The source move preserves activation and
Iterator Record storage, fresh iteration-environment publication, head and body
ordering, suspension state transitions, IteratorClose precedence, completion
dispatch and temporary-local release order. The
[`plain-async synchronous for-of lexical-pattern contract`](../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-lexical-pattern-heads.md)
remains the behavior boundary. Post-move compilation, structure, runtime,
formatting and module-boundary checks pass. The two focused structure targets
and the affected recursive policy census pass `15/15`, and the exact
lexical-pattern Wasm CLI oracle passes `1/1`. No new JavaScript behavior,
emitted-Wasm byte-identity result, Test262 result or published conformance
count is claimed. The move admits no additional head shape, suspension point,
async-generator owner or `for await` path.

The compiled-module package now encodes the backend's unconditional
callable-function table as a construction invariant. The sole emitter had
already fixed `uses_function_table` to literal `true`; the dead policy argument
and both conditional branches are gone. `ModuleTypeRegistry::new()` always
registers the JavaScript function signature, and `ModuleAssemblySections`
requires the callable range while retaining optional memory and data sections.
Its private paired owner constructs the `funcref` table and active element
segment from one first-index/count range, so raw empty or mismatched sections
cannot cross the assembly boundary. Compile-time function-pointer gates and the
strengthened
[`compiled module package ownership contract`](../docs/rust-rewrite/contracts/compiled-module-package-ownership.md)
make omission of either table half a type error and pin the index-1 JavaScript
signature plus exact sole-emitter construction. The former literal-true path is
source-equivalent, so this claims no changed Wasm bytes, JavaScript behavior or
conformance count. At the coordinated checkpoint, the AOT package check
(`cargo check -p lila-aot-wasm`) is green; the package structure target passes
`4/4`, the obsolete-planning target passes `3/3`, and the exact module-assembly,
emitted-module-validation and String memory/data controls are green. No Wasm-
golden, broad runtime or
published-status result is claimed.

## Objective

Split the current monolithic compiler implementation into stable ownership boundaries without changing JavaScript behavior or emitted semantics. At the time this plan was written, `lila-ir/src/lib.rs` and `lila-aot-wasm/src/lib.rs` are tens of thousands of lines and are the primary merge-conflict bottleneck.

## Required module boundaries

The exact filenames may change, but the resulting architecture must expose equivalent boundaries.

### `lila-ir`

- `ir/`: public `ProgramIr`, statements, expressions, functions, classes, properties, shapes, value information and IDs.
- `lowering/`: AST-to-spec-IR lowering, split by declarations, expressions, statements, functions/classes and modules.
- `early_errors/`: checks that are not delegated blindly to parser diagnostics.
- `builtins/`: builtin IDs, metadata, intrinsic ownership and feature registration.
- `analysis/`: scope/capture analysis, static shape/value analysis and unsupported-feature reporting.
- `operations/`: typed representations of shared ECMAScript abstract operations consumed by backends.
- `diagnostics/`: structured diagnostic codes and source locations.

### `lila-aot-wasm`

- `module/`: sections, imports/exports, tables, globals, data and validation.
- `abi/`: tagged values, call/construct convention, completion convention and host imports.
- `heap/`: allocation/layout/string/object/environment storage and memory growth.
- `emit/`: statement/expression/control-flow dispatch.
- `operations/`: code generation for shared abstract operations.
- `objects/`, `functions/`, `environments/`: internal-method emitters.
- `builtins/<family>.rs`: separate builtin families such as array, string, regexp, typed-array, date and intl.
- `intrinsics/`: realm bootstrap and property descriptor installation.

Keep a small `lib.rs` that re-exports the public API and invokes the top-level pipeline.

## Implementation sequence

1. Add characterization tests before moving code: compile representative fixtures, validate Wasm, execute with the project's lower-bound runtime (experimental Wasmtime feature set per `AGENTS.md`), and record observable outputs/completion kinds. Do not gate characterization on engines that lack the lower-bound features (Wasm GC, typed function references, `exnref`).
2. Extract pure data types and constants first, then helpers, then large emit branches.
3. Replace magic global/table/layout numbers with named registries that can assert uniqueness and stable ordering.
4. Use private modules and narrow `pub(crate)` interfaces. Do not make every helper public to avoid import work.
5. Preserve error text only where tests rely on classification; otherwise prefer stable diagnostic codes over brittle strings.
6. Move tests beside the module they cover while retaining end-to-end crate tests.
7. Keep each extraction commit buildable so regressions are bisectable.

## Non-goals

- No new builtin coverage.
- No redesign of the value representation or completion ABI; those belong to T04/T05.
- No bulk formatting of legacy JavaScript code.
- No generated Test262 count changes beyond accidental stale-status cleanup.

## Acceptance criteria

- Both giant `lib.rs` files become orchestration/re-export surfaces rather than implementation stores.
- Feature families have clear files that separate agents can own.
- There are no cyclic module dependencies or duplicate constant registries.
- Public APIs used by `lila-engine` remain coherent and documented.
- Representative emitted artifacts behave identically before and after extraction. If byte identity is not practical, compare imports, exports, validation, output, completion kind and thrown error class.
- Workspace compile time and binary size do not regress materially solely because of module movement.

## Required tests

```sh
cargo fmt --all --check
cargo check --workspace
cargo test -p lila-ir --quiet
cargo test -p lila-aot-wasm --quiet
cargo test -p lila-engine --quiet
cargo test -p lila-cli --quiet
./target/debug/lila test262 run language/wasm/pass \
  --suite-root crates/lila-test262/tests/fixtures/fake_test262/vendor/test262 \
  --execution-backend wasm
```

Also run several previously green real Test262 filters from different families to detect moved-helper regressions.
