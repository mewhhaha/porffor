# Immutable validated DateTimeFormat plan reuse

DateTimeFormat selection performs locale/calendar admission, builds pattern and
fraction variants, combines date/time candidate families, and selects seven
input-kind formats. The serialized `SCRIPTWDTF2` plan remains the existing recipe
plus exact DateTime, Locale, and named-zone image digests. Formatting may reuse
the immutable result of that selection under one admitted DateTimeProvider.

A cache miss first admits the unchanged magic and all three provider identities,
decodes the complete recipe, and checks named-zone availability. A private
decoded-plan owner retains that exact wire value, recipe, and provider together.
Only its successful original `plan::select` can mint the cache admission type.
Rejected plans, errors, inputs, and formatter results never enter the cache.
Range operations still admit decoding/identity/zone errors before their kind
check and perform uncached selection after it.

The validated plan owns shared immutable Locale and Calendar records and its
seven selected Formats. Locale records are shared by Arc; physical Calendar
records already belong to checked Arc pools. No self-reference, unchecked
reconstructed index, complete-profile clone, Realm, Store, JS heap object,
completion root, or job state enters this ownership graph.

One provider owns one cache. Keys compare all encoded bytes and retain all three
provider identities; a foreign image cannot consult or populate another owner's
cache. Independently admitted owners with identical bytes continue to accept
portable plans through their own original admission paths. This introduces no
new wire schema, identity digest, data producer, or reduced availability policy.

Retention is FIFO and bounded to 32 entries, 16 KiB per key, and 2 MiB of
conservatively counted owned allocations. Accounting includes the fixed queue,
Arc plan header, exact key allocation, selected plan, String and Vec capacities,
patterns, optional date/time patterns, range companions, and range glue. Shared
immutable profile records already retained by the provider are not duplicated.
An over-budget plan remains usable through the original uncached path. Cache
poisoning bypasses reuse. Cache locks are released before selection and before
rendering; concurrent misses may compute equivalent immutable plans, and only
one exact key is retained.

Every call still performs its actual Wasm argument conversions, fresh host
response encoding, live input/calendar conversion, named-zone transition lookup,
range ordering, parts construction, JS allocation, and called-Realm publication.
No output or transition snapshot is memoized. Old mutable parts and errors cannot
poison a later result.

Controls cover exact-owner reuse across threads, byte/key corruption, all three
identities, independent equal and foreign image owners, entry eviction, actual
allocation-budget eviction, uncached oversized plans, live plan ownership after
eviction, decode/kind/selection error order, fresh mutable parts, called Realms,
coercion effects, DST transitions, ranges, and the unchanged original pinned
14,544-date Temporal comparison in strict and sloppy modes. That original case
retains its canonical materializer prefix and 60,000 ms execution deadline.
