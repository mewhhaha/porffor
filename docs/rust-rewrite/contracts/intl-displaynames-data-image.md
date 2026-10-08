# DisplayNames native data image

DisplayNames operations consume the admitted immutable native image through
`DisplayNamesDataImage::display_name`. The payload is the actual 1,617,788-byte
schema-one JSON used by the checked native renderer. Its native descriptor is
`lila/display-names/profiles/v1`; it is not an ICU provider marker. The artifact
section is `lila.intl-display-names-data.v1`.

The exact captured CLDR47 payload contains thirteen locales and 105 typed name
pools. Every locale has complete long, short and narrow associations for the
six public domains: language, region, script, currency, calendar and date-time
field. Language-script and variant pools support the existing genuine language
composition algorithm. Admission preserves the exact source and BCP47 manifest
identities, canonical pool order, reference domains, complete twelve date-time
fields, nonempty resolved names and full pool coverage. The existing pure
native renderer and its fallback, composition and lookup algorithms are shared.

`for_profile` preserves the exact complete locked payload for Minimal and
matching named Custom profiles. `for_custom_projection` accepts one Custom ID,
a nonempty list of structural locale IDs and the actual selected Locale image.
It canonicalizes the request with that image, rejects duplicate canonical IDs
and locales absent from the thirteen-row pinned catalogue, and retains the real
en-US fallback once. The selected public support set consists of those actual
rows. Ordinary supported-locales lookup and fallback still use the shared
renderer; a request for an excluded locale does not gain support from another
Intl service's larger catalogue. Conformance remains explicitly unavailable.

The native projector first runs complete pinned-source admission. It then keeps
the selected locale rows, their locale patterns and separators, and every name
pool referenced by all eight typed domains at all three widths. Original pool
order is preserved, unreachable pools are removed, and all twenty-four pool
indices in each selected row are densely remapped. Each retained pool keeps its
complete code/name entries, including language-script and variant composition
data. The locale-only producer does not filter individual codes. Selecting French
with the required en-US fallback reaches twenty-six of the original 105 pools.

The separate schema-2 currency projection keeps only checked requested Currency
entries in those retained pools, deduplicates identical filtered pools and
remaps every association. Its exact private catalogue permits a genuinely empty
Currency pool; complete-source and ordinary locale-only admission remain strict.
All other name domains retain their original entries. Omitted codes use the
existing code/none fallback, and provider admission pairs the selected codes
with Number's actual currency domain. No substitute name data is created.

The inner `LILADNP1` frame has a twenty-byte header, a bounded canonical JSON
descriptor and the actual projected native JSON. Its descriptor binds schema,
Custom ID, en-US default, the full pinned source SHA-256, selected Locale image
digest, sorted public locales and original source pool indices. The descriptor
is limited to 16 KiB, the JSON to the complete source payload's byte length, and
the whole component to the existing immutable-image extent limit. Admission
reconstructs the full source validation and exact selected row/pool closure,
then requires equality with the entire canonical projected payload. Recomputed
outer digests cannot authorize altered names, different pool order, arbitrary
associations, missing rows, extra pools or self-described replacement pins.

Only that admission can mint the private borrowed `DisplayNamesCatalogue`.
Its physical JSON bytes and selected locale domain are inseparable; the native
subset decoder accepts the owner rather than unrelated bytes and a caller's
row list. It reuses complete schema, source identity, typed pool, composition,
date-time-field and all-pools-used validation. The full public `from_json` and
ordinary image branch retain their complete thirteen-row checksum requirement.

The constructor accepts the actual selected `LocaleDataImage`. Its data-only
canonicalization owner retains that image's ICU canonicalizer and alias payload.
Reserved-language alias evidence is derived once per complete image admission,
and reused for every source name and every later language-code operation. The
same `ParsedLocale` algorithm handles ordinary and reserved languages. The
selected Locale image retains its actual Unicode keyword alias authority;
the DisplayNames projection preserves that foundation rather than filtering
another component's data or manufacturing a whole provider identity.

The image retains both its native tables and canonicalization owner. Every
resolved DisplayNames locale owns the exact admitted locale-template Arc.
Selected image operations reject a configuration from a different template
owner before parsing its lossless UTF-16 code or looking up a name, even when
both images have identical bytes. Wire decoding admits its configuration against
the selected retained native profiles, so wire records can cross the primitive
boundary without importing another kernel's template proof. Clones of an image
share one admitted owner. Independently admitted images own separate proofs.

Provider installation must check matching component profile names and the
retained Locale image digest, then retain this image and call its operation.
Artifact and compiler-cache admission must bind the frame digest before use.
The complete twelve-component successor declares Embedded placement for its
finite selected data closure. DisplayNames locale projection does not widen its
captured thirteen-locale source coverage or filter the remaining components.
Conformance and general twelve-component Custom production remain open;
compilation and runtime acceptance of this successor remain pending.

Four source controls exercise all thirteen locale/type/width associations with
actual image data and dynamic source ownership; foreign-template rejection
before invalid-surrogate code validation; selected wire-template admission,
real language aliases, reserved-language fallback and composition; and exact
payload/profile refusal despite a newly computed valid outer frame. Existing
native controls retain malformed pool, source identity, missing-field and
fallback coverage. Four additional projection controls cover complete physical
pool closure and remapping with six-domain/three-width operation parity,
canonical ordering and unchanged full images, exact refusal of altered framed
data and provenance, and real selected wire admission, foreign-template gates
and retained owner lifetime. Existing control names and source cohorts remain.
The projection controls are authored source; compilation, data generation,
execution and tests have not run.
