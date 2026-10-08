# Actual timezone names image

`TimeZoneNamesDataImage` admits the complete checked CLDR47 English timezone-name
catalogue, independently of the IANA transition image. Its native marker is
`lila/time-zone/names/cldr47/v1`; its inert Wasm custom section is
`lila.intl-time-zone-names-data.v1`. The selected provider retains both actual
owners and checks their profile, IANA image digest and exact IANA `Arc` pairing
before publication.

The 160,640-byte native JSON contains all 446 zones, 600 aliases, 190 metazones,
669 half-open UTC periods and seven source patterns. Its SHA-256 is
`52508dfd4f0bca47b39443589c97e959361f1f57d7324876f0b681b0509012f4`.
`scripts/generate-intl-time-zone-names.py --native-image` projects the exact
existing parsed CLDR47/ICU country rows into that carrier, checking every frozen
source digest first. The source manifest, normalized rows, historical generated
Rust tables and historical report remain byte-identical. The historical Rust
tables are no longer registered as a production data consumer.

Admission checks the exact pinned payload, frame component/inventory/versions
and selected IANA profile before decoding. The native constructor validates the
closed schema, source identity, complete finite domain, sorted unique search
tables, terminal alias targets, preferred/golden zone reverse periods, ordered
nonempty UTC intervals, nonempty names and all pattern placeholders. It owns
the decoded strings, rows and indices; no global generated table or second
default decoder supplies formatting data.

The formatter retains the selected IANA owner and consumes it in the complete
`ResolveTimeZone` operation: lookup and exact transition selection precede
formatting from the admitted native names. Fixed offsets retain their original
path. Standard/daylight selection, direct zone overrides, primary identifiers,
half-open UTC metazone selection, generic-name stability proof, preferred-zone
qualification, generic location fallback and second-precision GMT offsets
retain their existing order and behavior. The supported name locale inventory
remains `en`/`en-US` with the existing validated `ca`, `hc` and `nu` extension
policy; the renderer continues using Latin digits for its GMT skeleton. The
source's standard/daylight region patterns are validated with the full pattern
domain; the existing generic location fallback consumes its generic pattern.

`from_bytes(bytes, &named_image)` captures that actual IANA owner. Equal image
digests do not authorize replacing it with an independently decoded owner.
`uses_named_zones` is the publication predicate, while `resolve` directly uses
the captured owner. A borrowed default accessor comes from one admitted image
cache. Named Custom profiles label these same exact pinned component rows;
non-English source expansion remains open for this internal operation. The
complete Conformance producer retains this same honest English contract;
DateTimeFormat's localized names come from its separate selected DateTime owner.
The complete twelve-owner provider declares Embedded placement.

Meaningful source controls cover real retained formatting after image/bytes
handles drop, actual standard/daylight and newer-IANA fallback, equal-digest
foreign owner rejection, mixed profile admission and checksum-consistent
altered payload rejection. Existing injected-transition controls retain the
full exact-boundary, qualification, seasonal-evidence, locale and GMT-offset
matrix; constructor controls reject broken aliases, periods, patterns and
golden zones. These controls are authored but have not run. The native JSON was
generated once under the confirmed 4096 MiB, zero-swap, one-CPU cap; no Rust
compilation, image export or runtime verification was performed in this batch.
