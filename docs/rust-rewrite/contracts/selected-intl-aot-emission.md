# Selected Intl data during AOT emission

The public emit_with_intl_profile entry accepts the closed IntlCompilationProfile
domain: Minimal or a validated named Custom profile. The default emit and promise
policy entry points select Minimal. Conformance cannot be constructed through
this API; named Custom currently selects the complete pinned component domains,
with its own canonical profile identity, rather than arbitrary data filtering.

One cheap IntlDataSelection is created after the existing blocking-diagnostic and
trusted Module-entry checks, before the builtin compilation retry loop. Every
pass receives the same selection. Its admitted SelectedIntlDataBundle owns all
twelve immutable components, their exact foundation topology, actual provider,
canonical identity and checked supported-values catalogue. No emitted frame can
be chosen independently from its provider or catalogue.

StringPool resolves the selected catalogue only when supportedValuesOf is
actually compiled. Each complete list retains its immutable Arc owner and is
checked against the selected provider identity before its names are interned.
The native builtin clones that same owner while compiling rooted whole String
values and a fresh called-Realm Array. Native localized String case consumers
capture this selection's typed DefaultLocale during collection; they do not
choose a catalogue identity or fall back to a global default provider.

The existing physical module assembly owner validates any collected catalogues,
then emits the selected bundle's canonical identity and exact twelve component
frames whenever the module imports Intl or the system-time-zone host, or consumes
an actual compiled supported-values catalogue. A supported-values-only module
uses its selected names as compiled Wasm and carries the full bound data group
without inventing a host import. Both localized case catalog entries already
require the Intl import and therefore carry the selected runtime frames.
An ordinary source without data consumers leaves image selection unresolved and
emits no Intl metadata. The existing inert Engine artifact gate remains the
consumer boundary before native compilation, cache use, start or execution.

The retained actual artifact control now covers Minimal frames, named Custom
frames and identity together with a compiled primitive catalogue, plus inert
ordinary source and the supported-values-only full data binding. The existing source
ownership witness follows the real selection, pool and assembly consumers.
Engine's coordinated profile integration adds ordinary Custom localized-case and
supported-values product controls. Code and controls in this packet are source
only: no compilation, validation execution, runtime, conformance, guard or image
export has run. Full Conformance and general custom-data generation remain open.
