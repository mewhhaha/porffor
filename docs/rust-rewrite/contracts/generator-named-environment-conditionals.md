# Generator conditionals with named Environment Records

The original Arguments reference enables live named records. Even an empty
braced branch then owns a real Environment Record. The old direct-yield splitter
correctly refused to erase that record, but its fallback emitted an ordinary If
with a hidden suspension. Backend statement entry/exit census could no longer
see the conditional, so the function's pre-yield lexical initializers executed
again on resume and overwrote mutations made through captured closures.

An actual analyzed braced branch record with a Yield now selects the existing
checked OrdinaryGeneratorIf owner outside foreign iterator bodies. The record
stays inside its selected branch region. The complete branch exposes entry and
exit phases to the enclosing statement sequence, which gates preceding lexical
initialization to first entry and restores the same cells on resume. Function
GeneratorPlan metadata for named environments is finalized through the existing
checked sequence_end and suspension census; source admission stays intact.

The original ownership control keeps parameter/local checks and the exact
source, and now requires the selected branch record, its one original Yield,
and agreement between complete continuation metadata and the function plan.
The native regression retains the exact observed32c failing captured-write
trace and requires second=1 rather than the recorded second=2, in both modes.
Skipped conditional completion remains independently checked.

The other two retained library controls had representation assumptions after
Arguments switched their scripts to named environments. A forward async default
now owns a named Read of the actual later Parameter cell. Backend entry creates
all formal cells before initialize_parameter evaluates defaults in source order;
GetBindingValue retains the ordinary runtime TDZ check. The test verifies the
named parameter record and both actual slots rather than requiring an embedded
RuntimeThrow expression. The root update keeps its literal Number RHS and named
Assign before the final named Read; conservative Dynamic result metadata does
not assert a semantic result. Paired native controls require the original final
value and the default's actual ReferenceError with valid lexical captures.

These are private repairs against32c197968. Root must compile the registered
controls, execute the captured diagnostic and complete affected inventories.
No native acceptance is inherited from private authoring.


The analyzed branch-record admission and checked plan finalization live in the
private `if_statement::checked_environment` child. The ordinary if orchestrator
consumes the admission once; function finalization consumes the metadata helper
once. The existing240-line if budget stays fixed, and the child has a64-line
raw budget. This extraction changes visibility and location only; the actual
analyzed-record check and continuation census remain unchanged.
