# Number and DateTime numbering-system projection

The Custom selection applies the same checked `NumberingSystemOption` list to
`NumberProfilesDataImage::for_custom_numbering_projection` and
`DateTimeDataImage::for_custom_numbering_projection`. Both producers require a
nonempty, duplicate-free subset of the actual pinned 78 positional systems and
canonicalize its order. Locale, currency and localized calendar selectors compose
with this dimension. Existing full images and earlier projection descriptors keep
their original bytes when no numbering selector is supplied.

Number's typed table closure retains only the selected localized numbering
profiles and each physical locale profile's original default. This includes the
private RelativeTime and Duration locale domains on the same Number owner. The
producer removes unreachable symbol, compact and pattern rows through the original
typed dense table maps. Its schema-three descriptor binds the selected names,
all original default-nu rows and the actual Locale/List foundation pins. The
version-three binary represents excluded localized references as `None`; only a
rederived private catalogue permits that reader. The full reader remains strict.
Validation requires the exact selected/default references and rejects absent
default records before publishing the owner.

DateTime's schema-three descriptor binds the same typed selection with its locale
and calendar selectors. Each selected locale keeps only its requested decimal and
minus-symbol rows plus its original default. The existing checked recipe validates
that exact row set. Internal calendar pattern overrides and algorithmic fields
keep their real dependencies; the complete positional digit kernels remain
available for those fields. No alternate Number or DateTime implementation is
introduced.

Service negotiation tests actual localized records. An excluded option or Unicode
`nu` extension falls back to that locale's original default; an excluded extension
does not appear in the resolved locale. Public wire admission rejects caller-made
resolved values for absent service records. Number partitioning also consumes the
checked optional reference, and DateTime plan validation consumes its actual
localized symbol map. RelativeTime and Duration retain their original default
closure and use the same admitted Number Arc.

Global availability remains separate. Number retains all 78 original digit rows
and the complete small default-nu authority used by `Intl.Locale.numberingSystems`.
DateTime exposes the actual admitted full digit-kernel names. SupportedValues
compares those global inventories rather than requiring every selected service
row to accept every global option. Provider admission requires equal requested
domains on the actual Number and DateTime images before joining them.

Four authored native controls inspect removed physical tables and symbol rows,
compare selected/default formatting with the complete source, exercise currency
and calendar composition, retain global/default-nu authority and owner lifetime,
and reject changed descriptors, missing defaults, extra rows and forged service
selections. SDK, manifest-v4, cache and CLI controls consume these same producers.
Formatting, compilation, exports and tests remain deferred; no measured size or
conformance claim follows from the source changes.
