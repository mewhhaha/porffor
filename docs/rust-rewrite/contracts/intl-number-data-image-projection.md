# Coupled NumberFormat and PluralRules projection

`NumberProfilesDataImage::for_custom_projection` selects one public locale
catalogue for NumberFormat and PluralRules. It takes the Custom ID, public locale
requests, optional RelativeTime and Duration locale requests, and the actual
Locale and List images. Selection canonicalizes against the selected Locale
authority, rejects duplicate canonical or absent source rows, sorts once, and
retains `en-US` exactly once as the real fallback. `None` for a dependent selector
means that component's complete existing native catalogue.

The producer starts with `PinnedNumberSource`, which admits the exact locked
native descriptor and 6,091,829-byte LNF47v2 binary. Its complete typed serializer
must reproduce those bytes before selection. Full Minimal and named Custom image
production keeps the existing native payload bytes. Projected data uses a
separate bounded native frame inside the same immutable Number component and
must carry a Custom label; Conformance remains unavailable.

The physical locale roots are the union of the public catalogue, the actual
selected RelativeTime template catalogue, and the actual selected Duration
catalogue. The two dependency catalogues are derived with the existing complete
pinned source validators and their real canonical row selectors before those
dependent images are constructed. The descriptor binds their selected domains
and original native source pins, without a circular dependent-image digest.
Duration source validation consumes the supplied List image's real private
Duration associations. Projected Number admission also binds that List digest
and the selected Locale digest.

The serializer follows every typed table edge reachable from those locale
roots: Number symbols, grouping and signed patterns, compact choices and
exponents, currency patterns and spacing, currency labels and overrides, unit
choices, cardinal and ordinal rules, and plural ranges. Selected table IDs are
remapped densely in original source order. The first producer retains all 78
numbering systems, their digits and each selected locale's complete system
matrix, the sanctioned unit inventory, currency-fraction defaults and overrides,
and the three global Unicode character sets. It does not claim to filter those
global domains.

`NumberCatalogue` is private admission evidence joining exact selected binary
bytes and their public/dependent domains. Dynamic admission bounds the frame,
recomputes the complete canonical descriptor and selected binary from locked
source data and supplied foundations, and requires byte equality. The existing
LNF47v2 decoder then admits every table and index and requires the physical
locale inventory to equal the derived union. Caller-supplied metadata, row
payloads, replacement source hashes, and reordered or noncanonical frames cannot
mint the catalogue.

One admitted `Arc<NumberProfiles>` serves all consumers. A closed borrowed
`NumberLocaleView` binds either the public, RelativeTime, or Duration domain to
that same Arc. Public Number/Plural resolution, supported-locales queries, and
wire reminting admit only public rows. RelativeTime and Duration's actual
configuration/template and wire consumers use their corresponding private view;
hidden data cannot expand public support. Their image constructors require their
actual published catalogue to equal the recorded Number dependency domain, and
retain the same Number Arc. Duration additionally requires the selected List
digest. Existing operation owner checks remain in force.

`Intl.Locale.getNumberingSystems` remains unfiltered. A small complete 1,082-row
default-numbering map preserves its original locale-prefix behavior without
retaining excluded locales' heavy Number tables. It is included in canonical
projection rederivation and shares the retained 78-system inventory.

Native controls cover physical shrinking with hidden rows retained, actual
RelativeTime and Duration formatting and wire decoding, public wire refusal and
selected-owner reminting, all original default-numbering rows, exact dependent
domains and List pairing, altered metadata/data rejection, canonical selection,
and owner lifetime after frame handles drop. Serializer controls cover complete
pinned parity and the typed compact/currency/unit/plural closure. These controls
are authored source; compilation and runtime verification remain pending for
the complete batch.
