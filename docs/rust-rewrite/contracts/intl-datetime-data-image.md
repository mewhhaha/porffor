# Native DateTime and calendar data image

The DateTime component consumes the exact native CLDR47 schema2 profile and all
calendar payloads used by the existing sixteen admitted calendar identities.
It does not substitute ICU DateTimeFormat for the native pattern renderer.
The 9,879,674-byte profile retains the thirteen locale rows, 135 calendar pools,
nine zone-name pools, 78 positional digit systems, two algorithmic field rules,
available formats, interval patterns, day periods, geography and names. Its
explicit CLDR48 era and `tols`/Unicode17 supplements retain their existing source
validation; they are not silently described as CLDR47-only data.

`DateTimeDataImage` admits one bounded versioned frame containing native JSON
and a deterministic postcard ICU calendar blob. The ordinary Minimal and named
Custom factories retain the exact complete pinned bytes. The five consumed
schema descriptors are the native `lila/datetime/profiles/v2`, ICU Chinese,
Dangi and modern Japanese singleton markers, and the vendored
`lila/calendar/hijri/ummalqura/v1` marker. The last marker serializes the actual
pre-existing 301-year Umm al-Qura table (1300–1600); it is not an upstream ICU
marker inventory claim. Safe little-endian storage is derived from that table
only for the build-time baked provider. The runtime calendar retains the
deserialized payload, validates extent and year continuity once, and reads it
for calendar arithmetic. The existing Friday tabular calculation outside that
table is preserved without introducing a wider correctness claim.

The closed `CalendarId` inventory constructs every retained kernel through the
actual buffer provider before publication. The other calendars have no data
payload and retain their existing algorithms. The native pool admission and
AvailableCalendars constructor independently validate the rendering domain.
Modern Japanese era projection, Chinese/Dangi leap-month identities and
proleptic calculations keep their existing semantics. Generic ICU constructors
for unadmitted extended Japanese or simulated Hijri calendars do not expand this
component's accepted domain. Explicit compiled-data oracle constructors remain
only in test fixtures; product formatting uses the selected buffer owner.

Admission consumes the selected Locale data authority to check native catalogue
identifiers and retains its exact owner. Calendar option aliases use the same
selected immutable keyword authority, rather than global generated rows.
The image also retains the selected IANA image and actual named-zone Arc.
Installation requires matching profiles, foundation digests and exact owners.
DateTime formatting obtains offsets, transitions and gaps from that retained
IANA owner. Standalone time-zone display-name data has a separate component;
the native DateTime JSON already owns the CLDR names and patterns its renderer
actually consumes.

Opaque plans use `LILADTF2`, the complete DateTime image digest, selected Locale
and IANA image digests and the existing primitive recipe. Every scalar or range entry
checks these bindings before date projection, named-zone lookup or range
input-kind decisions. Plans remain portable between separately admitted owners
with identical content; process-local Arc addresses are not serialized. The
actual stateless wire transports the bound plan, and the Engine selects the
admitted DateTime owner before operation execution. Host code receives primitive
requests, with JavaScript observation and coercion remaining in compiled Wasm.

Four authored image controls exercise all sixteen calendar selections and
actual lunar/era/cache-boundary fields, calendar aliases, owned dynamic payload
lifetime, selected plan wire portability and foreign identity precedence,
New York DST parts and range endpoints, exact Locale/IANA owner predicates,
altered framed payloads and foreign Conformance foundation refusal. The existing native
calendar, profile, pattern, range and far-date controls remain required. These
new sources and controls have not been compiled or executed in this source
batch. Whole Intl placement is Embedded through the complete typed provider;
artifact integration and runtime acceptance remain unverified. Current Custom
identifiers label the same finite pinned data when no projection is requested.
The explicit Conformance producer admits the complete pinned DateTime source.

The checked `for_custom_projection(id, locales, locale, named)` factory selects
actual canonical rows from the thirteen-row source catalogue and always retains
`en-US`. Empty, duplicate canonical and absent locale requests are rejected.
The complete source decoder validates every original row, pool, supplement and
global authority before selection. Both calendar and zone-name pool closures
retain only records reachable from the selected rows, in original source order,
then rewrite associations to dense indices. Distinct private calendar and
zone-name index types prevent a remap from accepting the other pool's IDs. All
sixteen calendar associations per row, 78 positional digit systems, both
algorithmic rules, geography, supplements and the complete four-marker calendar
blob remain. The IANA image retains its complete named-zone inventory.

The native field of the existing frame can carry a bounded `LILADTP1` projection
descriptor and canonical JSON. The descriptor binds the Custom ID, complete
native source hash, Locale and IANA digests, sorted public rows and both original
pool-index subsequences. Admission derives the complete projection again and
requires byte equality before minting the private `DateTimeCatalogue`. Only
that owner can authorize the reduced locale recipe in the existing native
decoder. Ordinary `Profile::from_json` still requires the exact thirteen-row
recipe, and both branches share every other validation and physical consumer.
Runtime lookup and supported-locales queries read the actual selected catalogue;
unselected rows cannot leak from a hidden full table. For example, a projection
of `ar-EG` and `zh` admits these rows plus `en-US`, resolves `zh-Hans-CN` through
`zh`, and falls back from unsupported `fr` to `en-US`.

Four additional source-only controls cover physical shrinkage of both pools,
unchanged globals and public lookup, scalar parity across all sixteen calendars
and all positional numbering systems, independent Chinese leap-month and Arabic
field goldens, DST range parts, portable selected plans after input-owner drop,
foreign-plan refusal, and damaged row/pool/foundation admission. Existing control
names and source cohorts are retained. These controls have not been compiled or
run; the projection does not establish a broader data or Conformance claim.
