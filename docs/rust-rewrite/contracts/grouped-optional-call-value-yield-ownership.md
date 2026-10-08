# Grouped optional Property Reference and Call Value yield ownership

Status: 2026-10-04 dry source implementation. This batch's compilation, Wasm
validation and runtime controls remain pending; published conformance counts
are unchanged. The preceding shared foundation has its own all-target typecheck
checkpoint and does not establish acceptance of this new batch.

An ordinary generator outside every loop may use a yielded optional chain
ending in Property or Call as the completed callee of an ordinary outer Call
or tag. The actual terminal AST link mints the closed
`GeneratorGroupedOptionalInvocationSource::PropertyReference` or `CallValue`
role, each containing the complete existing checked chain. Private fields keep
the terminal role inseparable from that source. The base remains eager; each
key or argument is eager or contains one plain Yield, and several links may
yield. Private/super links, yielded bases, mixed Await/Yield, delegated yields,
async generators and unowned value-arm contexts retain their existing source
boundaries.

The mandatory generator expression planner derives this same closed owner
before any states and appends the complete guarded optional-chain state plan
atomically. It does not flatten guarded yields into the single-arm count. The
Value producer remains the entry for ordinary expressions and Construct. The
Reference producer accepts only the checked terminal-Property wrapper plus the
existing captured receiver destination; a generic chain cannot select it.

Call/tag acquisition declares that receiver as undefined before the entire
inner prefix. The private generator destination is consumed only by the final
selected Property Get. Earlier Gets retain their current values, and inner
Calls retain their own original References before yielded arguments. The final
Get consumes the existing `OptionalCallReferenceCaptureIr`, preserving the
terminal base's original raw identity even when it is primitive. It neither
re-reads the property nor copies an earlier receiver. Nullish shorting leaves
the terminal receiver undefined and publishes the existing undefined callee.
The terminal-Call role continues to supply a completed ordinary Value with no
outer receiver.

The existing suspended-call callee pin precedes every outer argument, spread,
GetTemplateObject and substitution. Grouping ends inner shorting: an ordinary
outer Call/tag evaluates its operands after the inner chain returns undefined,
then reaches the existing callability error. The original parsed template
supplies the cached/frozen site object. An inner getter/Call/spread throw or
Throw/Return injected at a selected Yield bypasses later outer operands and
retains its arbitrary completion through the existing generator owners.
Optional Calls remain indirect eval; ordinary direct eval handling is unchanged.

A yielded optional base, including an optional outer Call over a yielded inner
group, retains its existing refusal. Construct keeps its separate existing Value
route for either terminal kind, without publishing a call receiver. No IR
expression, opcode, dispatcher, frame/ABI field, object model or backend is added.

Meaningful existing IR controls observe complete chain state joins, selected
terminal receiver capture, retained intermediate Get values, normal/undefined
publication, receiver-free Call/Construct Values, unconditional outer operands,
spread snapshots and the actual template owner. Only the newly admitted
terminal-Property refusal rows are retired; loop, mixed, protocol and unowned
value-arm boundaries remain checked before state publication. The existing
paired Engine fixtures author mutation, primitive receiver, shorting, template
identity and arbitrary foreign/injected completion assertions. Authored controls
cannot establish executable acceptance.
