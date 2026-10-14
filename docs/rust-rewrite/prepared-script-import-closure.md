# Prepared direct eval imports from a Script entry

This candidate closes literal `import()` requests in finite direct eval source
through the actual Script entry's existing module graph. Nested direct evals
retain that same Script Record. Discovery keeps parsed finite source, import
phase and attribute eligibility; the host resolves each request against the
Script key before artifact identity is computed. No source is parsed at runtime.

A private typed origin distinguishes discovery from a graph-bound Script
record. Module activations and their closures, indirect eval, separately
compiled Function-constructor source, Realm Script source and Module preludes
cannot inherit the entry origin by sharing allocation counters. Their import
origins remain an explicit unsupported capability in this bounded candidate.
Computed specifiers inside prepared eval remain unsupported as well; an
unmatched evaluated source still reaches the existing fatal typed source
rejection. It cannot become a caught JavaScript failure or a suite pass.

Prepared sites are metadata on the retained Script source, separate from that
source's parser-retained import heads. Rewriting still checks each original
head one-for-one. The filesystem loader closes the additional exact requests;
RejectAll retains its denial. A complete embedded catalog gains no ambient
loading: only already declared Script-referrer edges can satisfy the sites.
The Test262 host independently declares these finite literal requests from its
already parsed original Script under the existing directory-confined policy.
It does not grant sibling fixtures to computed or unproved eval requests.

Before finite-source discovery, the Test262 catalog checks the retained Script
AST with the compiler's existing whole-tree direct-eval visitor. It unwraps
parentheses, matches the decoded identifier symbol and visits nested functions
and methods. A Script with no possible bare-reference eval call skips this extra
IR lowering. Property, indirect and optional eval calls do not themselves trigger
that discovery. A positive scan supplies no authority: the same lowering session
still proves finite source and origin, and computed or unknown values retain
their unsupported bounds. No substring search decides this guard.

The compiled graph carries trusted Function IDs for its Script import
callbacks. Prepared code invokes the existing intrinsic async dispatcher in
the selected Realm, using the same independent function-record and canonical
module-cache machinery already used for Realm import jobs. Both operands are
evaluated before invocation; the dispatcher owns intrinsic promise creation,
coercion, exact attributes, module evaluation, microtask continuations, one
namespace identity and cached rejection values. No module-job implementation
or runtime ABI is duplicated. See
[ImportCall evaluation](https://tc39.es/ecma262/#sec-import-call-runtime-semantics-evaluation).

The original sloppy and strict execution IDs are
`language/expressions/dynamic-import/usage-from-eval.js`. Their original body,
fixture module, harness and observed NotImplemented outcomes are retained in
the private preparation packet. Eleven new controls cover cached nested-source
syntax, graph/referrer ownership, dependency identity and RejectAll, the
unchanged pinned case in both modes, relative Script bases, fresh promises and
one namespace, exact cached rejection values, fatal computed/module-origin
bounds, complete Test262 catalog authority, a callback that must stay uncalled for
ordinary scripts, and real finite-source discovery for nested, escaped and
parenthesized direct eval calls. The existing exact source-owner
inventory assertion includes the new metadata field and methods; its ownership,
private-parse and original scanner assertions remain exact.

The candidate is unverified. Rustfmt and patch/source/hash checks are preparation
evidence only. Root must compile the joined final source, run the new controls,
replay both unchanged original execution IDs and run the affected broad gates.
No native acceptance or conformance-count change is established here. The AST
guard removes the unconditional extra lowering path; its cost and the remaining
finite-source discovery cost have not been measured. No speedup
is established before the joined source is compiled and measured.

The canonical Script import dispatcher owns the existing options-validation
helper in its own activation. Prepared eval creates an independent callable in
the selected Realm's global environment; it must not capture the original
Script's private prelude slot. Planning admits such dispatchers through a
private proof that their compiled source has no lexical, private, this or
arguments captures. Module dispatchers retain their initializer closures. The
options helper, validation order, intrinsic async promise and module-job source
are unchanged. Existing failed witnesses remain intact; capture ownership and
live/reentrant/abrupt options controls require the next actual verification.
