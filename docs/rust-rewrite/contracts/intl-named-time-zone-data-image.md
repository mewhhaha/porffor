# Named timezone and country data image

`NamedTimeZoneDataImage` carries the complete existing IANA2026a authority:
598 normalized catalogue rows and their actual slim rearguard TZif records,
the original 418-row `zone.tab`, and the checked 247-country projection.
Its native inventory is `lila/time-zone/iana/v1`; its artifact section is
`lila.intl-named-time-zone-data.v1`.

The source-owned build exporter reads the exact locked `jiff-tzdb` 0.1.6
archive and pinned native catalogue/membership inputs. It emits one closed
binary payload with checked section lengths and every ordered identifier/TZif
pair. Runtime admission reads those physical bytes; it does not query
`jiff_tzdb`, the operating system, or a second transition archive.

Admission requires the exact exported pinned payload and complete frame
integrity before constructing native consumers. It validates the catalogue
digest, all individual TZif digests, exact 598-record extent and spelling,
case-insensitive uniqueness, terminal country-preserving primary identities
and the canonical UTC identity. Existing 64-bit transition/index/offset,
leap-second, POSIX-tail, offset-catalogue and complete gap-topology validators
remain mandatory. Forward, inverse and transition query algorithms are
unchanged.

Country projection admission consumes the same retained `Arc<NamedTimeZones>`.
It checks source and projection digests, derives membership through actual
terminal primaries, and requires exact source/projection equality and extent.
The retained country owner keeps its actual named-zone foundation alive after
image and source handles drop. It continues querying explicit base countries;
Unicode region/subdivision/timezone keywords do not substitute another country.

Minimal and named Custom frames currently label this same exact pinned data.
The complete Conformance producer retains this same full authority. Standalone CLDR timezone names have a separate
native image; this IANA image does not claim their locale/text coverage. A
DateTime image must retain this actual named-zone owner and bind its image
digest before publishing plans or rendering.

Four authored controls consume admitted images and cover retained country/
transition owners, independent same-byte foundations, checksum-consistent
altered or incomplete payload rejection, and named Custom catalogue/membership
consumption. Existing complete-catalogue, forward/inverse, transition, malformed
data and country-semantic controls remain connected to the admitted image.
No compilation, exporter execution, tests or runtime verification has run in
this source-only batch. Root owns shared build/module/provider/artifact and
provenance joins. The complete twelve-component successor declares Embedded
placement for its finite selected data closure; verification remains pending.
