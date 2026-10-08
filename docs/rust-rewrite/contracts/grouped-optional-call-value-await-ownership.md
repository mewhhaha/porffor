# Grouped optional terminal-Call Value await ownership

Status: dry source implementation. Compilation, emitted Wasm validation and
runtime controls remain unverified; published conformance results are unchanged.

A grouped awaited optional chain ending in a Call supplies an ordinary Value
to an outer Call or tag. Its inner property Reference has already been consumed.
Plain async functions outside all loops admit this route through the existing
checked optional tail and suspended invocation owners. A terminal property
keeps the established grouped Reference route and its captured raw receiver,
including target-only Await before a synchronous Property tail.

The actual grouped AST mints a private paired chain and closed terminal kind.
The context factory turns that pair into a property-Reference source or a
terminal-Call Value source whose fields remain private. The two actual capture
consumers accept their corresponding source owners; a terminal Call cannot be
passed to the terminal-property Reference emitter. The mandatory shared prefix
walk and recursive first-Call source proof run before any continuation states.

Both grouped terminal roles admit link suspension and target-only suspension.
Their actual-source factory requires Await and assembles the same finite
Property/Call tail through `from_links`; the ordinary value factory retains its
previous link-await admission.
A target-only terminal Call consumes the full evaluated-base/tail producer,
including a synchronous tail. Thus `((object?.[await key])?.())` retains the
inner method Reference, while `((await object)?.method())` and
`((await functionValue)?.())` consume their actual completed bases. Every
recursive grouped first-Call target passes the same source proof before state
consumption. Synchronous chains and other existing target-only value routes
retain their original admission.

The completed terminal Call is retained as the outer callee with no receiver.
The existing callee pin precedes every outer argument, spread capture,
GetTemplateObject and substitution. An ordinary outer Call/tag evaluates its
operands even when the inner optional chain shorted to undefined, then reaches
the existing callability error. An optional outer Call tests the retained Value
first and skips its complete suffix when nullish. Grouping ends inner shorting;
it does not condition an ordinary outer argument or substitution. Inner keys,
Gets and Calls retain their original receiver, once-only observation and order.

Existing private conditional scopes, branch state plans, sequence-exit checks,
ordinary zero-suspension If completion and fact merges own these continuations.
Only Normal completion publishes the inner or final result. Rejection or a
getter/Call/spread throw bypasses later outer operands and retains its arbitrary
value and Realm through the existing catch/finally owners. Existing template
site identity, frozen arrays and captured spread vectors remain the consumers.
No IR variant, dispatcher, opcode, activation layout or object model is added.

Target-only terminal Property References now consume the same closed grouped
source and existing Reference producer. Delete, direct private/super targets,
private links, loop regions, mixed Await/Yield, ordinary
generators and async generators retain their current preflight boundaries.
Construct and the bounded generator optional-chain Value route retain their
separate existing owners.

Meaningful IR controls inspect the actual terminal result, outer receiver None,
callee pin and operand order, retained nested first-Call receiver, optional versus
ordinary outer evaluation, state/activation-cell associations and remaining
preflight refusals. The three existing paired Engine fixtures cover returned
Calls/tags, direct and nested target-only bases, spread snapshots, template cache,
nullish/noncallable ordering, rejection and foreign abrupt identity. These are
authored controls, not executable evidence.
