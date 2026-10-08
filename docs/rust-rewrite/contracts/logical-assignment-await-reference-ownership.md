# Awaited logical-assignment Reference ownership

Plain async functions can lower `&&=`, `||=` and `??=` with awaited left operands or
RHS in a checked async branch context. Identifier, ordinary property, private and
Super targets consume the same retained Reference producer as the other resumable
protocols. The Get happens once after complete left evaluation and before branch selection.
The skipped branch returns that original value; only the selected branch evaluates
the RHS, awaits it and performs PutValue. The result is published after a successful
write. `??=` uses strict saved-value comparisons to null and undefined, preserving
falsy non-nullish values and HTMLDDA.

`AwaitedLogicalAssignmentSource` is consumed by the actual assignment lowerer and
required by the shared prefix admission walk before state allocation. The walk skips
nested activation bodies and visits current class keys, heritage and static
execution. Declarative references retain the original located binding and consume
the existing TDZ/immutable write owners. A TDZ Get is abrupt before RHS evaluation;
an immutable selected write fails after the RHS. Resolution is not repeated.

Ordinary properties consume `OrdinaryPropertyReferencePlan::capture_get`. Its private
role slots separate the original receiver, actual ToObject target and normalized
String/Symbol key. The Get emitter writes those legitimate JS values into the
existing activation environment. The non-Clone write authority creates
`CapturedOrdinaryPropertyWriteIr`; its emitter accepts neither a raw key nor an
independently constructed reference. It reads the three original slots, calls the
existing Set transition, propagates arbitrary throws and strict false-Set failure,
and then returns the RHS. Synchronous optional values may form an outer ordinary
property receiver. Getter/setter reachability, heap/data collection, native and
arbitrary throw inference, source call-flow proofs and both expression emission
routes consume the new IR variants. Local planning covers the eight-local Get
carrier and nine-local Put carrier plus the actual seven-local Set/error peaks.

The same existing branch/result owner handles nesting in conditional/logical values,
call/new/tag arguments, declarations, return values, outer Await and switch selectors.
Runtime, With, script-global-object and unresolvable identifiers use the existing
captured Identifier slot and original resolution outcome. Private targets retain
the original base and brand; Super targets retain receiver, base and normalized key.
The skipped Identifier arm retires its captured slot. Other skipped targets keep
their original Get value. Awaited left operands with an eager RHS use the ordinary
If without inventing branch resume states. Unowned loop contexts remain refused.
Plain and compound assignments, special reads, updates, ImportCall and Delete now
consume the shared resumable operand and Reference helpers, preserving their own
Get/coercion/Put order rather than treating every assignment as a logical one.

The ordering and Reference mutation follow
[assignment evaluation](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-assignment-operators-runtime-semantics-evaluation)
and
[GetValue/PutValue](https://tc39.es/ecma262/2026/multipage/ecmascript-data-types-and-values.html#sec-getvalue).
GetValue converts a property name once on the original Reference; its later PutValue
uses that converted name and original receiver.

Source controls cover state joins, original declarative storage, three distinct
activation slots, selected immutable failure, TDZ preflight and unsupported source
boundaries. Strict/sloppy Engine fixtures cover skipped primitive/object values,
thenable mutation and job order, Proxy/accessor receiver identity, key hooks and
Symbols, primitive boxing, optional-value receivers, foreign arbitrary exceptions,
strict/sloppy failed Set and awaited finally. Maintained switch controls exercise
logical assignment in discriminant and selector prefixes. This packet has only
source authoring only; compile, Wasm validation and all new runtime/test acceptance
remain pending. The additional complete-reference and reference-operation fixtures
cover awaited LHS, skipped RHS, private brand order, captured Super base mutation,
plain raw-key conversion after RHS and Super Delete's ReferenceError before coercion.
