# Native object property publication

The ordinary ObjectLiteral allocator and a resumed ObjectPropertyDefinition use
one private physical property body in `objects/object_literal_property.rs`.
Its exhaustive property match retains the existing prototype setter, spread,
data attributes, computed name inference, method HomeObject and merged accessor
definitions. No property algorithm is copied into the resumed path.

The checked IR carrier accepts an exact Object target. Its actual source producer
retains the freshly allocated ordinary literal in an activation binding; each
definition acquires that whole target before its property operands, roots the
ordinary object reference and returns the same target after normal publication.
Computed source keys finish evaluation and ToPropertyKey before their value
prefix can suspend. The shared native body then consumes that normalized key;
ordinary eager literals continue to evaluate raw keys through the same body.

Pending whole abrupt completions follow the original literal error route.
Functions receive the original inferred name and the retained literal itself as
HomeObject. Getter/setter definitions preserve existing attributes and counterpart
merging. Prototype setters update the same ordinary object; spread delegates to
the existing CopyDataProperties consumer with its whole source value.

The ordinary property-body extraction is mechanical: removing its enclosing
loop changes two `continue` exits into normal function returns, and parameters
replace local borrows. The semantic Engine/IR cohorts are owned by the staged
object-literal source lane. This native packet is source-only; no compilation,
test, runtime or guard execution has run, and complete T09/T15 acceptance remains
open until combined verification and the other source owners are complete.
