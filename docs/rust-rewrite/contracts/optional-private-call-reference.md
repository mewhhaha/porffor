# Private optional Call References

Status: T09 source authored; compilation, emitted Wasm, runtime and pinned
acceptance remain pending.

`lowering/optional_chain.rs` admits an initial property Reference through a
closed ordinary/private domain. For `base.#method?.(arguments)` and its
parenthesized forms, the base is the chain target and the first operation is a
nonshorted `PrivateProperty`. The existing optional Call follows it with the
same private identity and `ReferenceOrUndefined` receiver authority. An absent
or null private callee skips argument evaluation; an invalid or null base still
performs the required private brand check and throws before that guard.

The real backend already roots the acquired private receiver, performs
`compile_private_read_to_locals`, propagates its whole Throw and then checks the
optional callee before creating the argument vector. The captured receiver is
the Call receiver even when arguments replace the source binding or the
getter-returned callee. Non-nullish noncallables still evaluate arguments before
the callability error. A getter or argument's Symbol or other arbitrary Throw
retains its original identity.

Private-chain analysis distinguishes a proven data read or absent getter from
an accessor or unknown private read. The latter may run user code, so it
invalidates live caller facts and clears captured receiver/result heap shapes
before analyzing following arguments. This applies to existing optional
private property reads as well as the newly admitted initial private Reference.
It adds no synthetic function target or primitive fallback. Existing suspended
Reference ownership and prepared-source policies retain their independent
owners; this patch does not expand generator loop admission.

One IR control retains the mandatory private read, guarded argument call,
private identity and receiver through direct, parenthesized and null-base
forms. Two Engine controls each run Script and strict Script. They retain the
original receiver under argument mutation, static private receivers, missing
and null callees, wrong brands including Proxy wrappers, arguments before
callability, getter mutation, a getter-returned callable Proxy, callee
replacement and exact abrupt-value cutoffs. These are authored controls, not
reported executions.

The same coherent T02 source batch gives optional-chain analysis, captured
conditional facts and their join algebra, and ordered object literals private
owners. Conditional and object algorithms are retained; the optional-call
source-authority control follows its actual physical owner while preserving
the existing construction-route census. No public facade, compatibility
forwarder, second implementation or implementation `include!` is added.

The original 605 IR unit controls and their source literals remain unchanged;
the new private-reference control makes that inventory 606. Mandatory
verification includes the affected optional-source and existing class/optional
cohorts, the two new Engine controls, representative extraction equivalence,
then the joined broad and pinned checkpoints under the shared resource cap.
No compilation, tests, guards, JavaScript parsing, generators or runtime
execution ran while authoring this packet.
