# Compiled Temporal named-zone provider boundary

Temporal user semantics remain in the Wasm AOT compiler. The host supplies
pinned IANA identities, integer offset seconds, complete raw inverse candidate
lists and strict true-offset transitions. UTC and numeric offsets are pure
compiled paths. The existing six-slot ZonedDateTime record and BigInt epoch
representation remain the sole object representation.

Private proof handles connect actual factories to allocation. Receiver and
argument guards produce branded records; zone conversion retains normalized
Identifier and terminal PrimaryIdentifier; prescribed ToBigInt/range validation
produces a canonical `(floor seconds, nanosecond)` Instant. The allocator accepts
only borrowed Instant, zone and calendar proofs. Constructor, From, Now and
withTimeZone retain their first completed proofs rather than resolving or
normalizing them a second time. Branded input copy reads options inside the
actual brand guard before recovering internal slots, without public getters.

Result destinations are reserved below retained scratch or inputs. Prepared
zone/calendar/Instant/ISO/options outputs expose no usable raw coordinates.
Only the actual resolver, conversion, arithmetic or policy emitter can turn
them into proof handles. Releases follow the emitter's strict LIFO discipline.
Relative duration contexts retain zoned origin Instant, resolved zone and actual
calendar; their proofs are available only inside the emitted kind guard.

The existing owned-span import carries sealed operations 14, 15 and 16.
Offset and transition queries accept an Instant; inverse accepts its separate
extended local domain. Capacity queries are bounded per closed operation,
writers return the exact requested extent, and every reader stays within the
returned span. Identity lookup validates both returned names and request case
folding. Candidate decoding validates complete extent, canonical fractions,
contextual arithmetic, strict ordering/deduplication and exact roundtrip.
Every inverse candidate passes Instant range checks before policy selection;
none is silently filtered into a gap or truncated to two results.

The immutable native catalogue certificate proves an empty gap has globally
nearest sole local endpoints. Its boundary witness is minted by the same zone's
shared forward selector. Compiled disambiguation validates the endpoint epochs,
uses the actual offset difference, and performs the required inverse query
again. It does not replace nearest-endpoint semantics with day probes.
GetStartOfDay remains a distinct operation and checks both its candidate and
after-gap result as an Instant. Transition decoding checks strict previous
nanosecond behavior and strict next behavior; metadata-only records are absent.

Balanced ISO projection is distinct from construction limits. The inverse
ValidateISODaysRange operation retains its prescribed inclusive +/-100000000
checkpoints. Existing wider shared PlainDate/PDT construction bounds are
preserved: the immutable approved proposal and the unmerged integration draft
contradict one another at the minimum valid Instant projected through -01:00.
The integration range change is treated as an inferred transcription defect,
not an upstream ruling. Projection and raw AddDaysToISODate do not add creation
range checks. GetStartOfDay's explicit after-gap Instant validation follows the
current integration source.

Syntactically valid far-date ZDT strings finish zone/calendar conversion and
ordered options before range failure. Outside the extended local context,
all less-than-day offsets and certified gap endpoints necessarily produce
an out-of-range epoch; the compiler emits the JavaScript RangeError before
entering the provider's trusted internal coordinate constructor. Property bags
perform their prescribed shared ISODateWithinLimits check after regulation.
A local day outside the inverse checked-day domain can still map to the exact
minimum Instant on a wall-time named path; there is no blanket day-range guard.

Now converts its zone before sampling the existing host clock. At this
earlier named-zone checkpoint the default policy chose UTC. The later authored
[configured-zone primitive](date-system-time-zone.md) supplies actual immutable
Realm defaults through its consumed host import; verification remains pending.
String rounding quanta divide the endpoint day grid, proving the rounded pair
remains an Instant without adding a runtime range error. Snapshot projection
always belongs to its exact, possibly rounded, epoch. Exact offset getters
preserve historical seconds; formatting minute rounding is a separate compiled
policy.

This contract is staged for the complete named-zone consumer batch. Source and
patch checks and Node reference controls are separate from Lila execution.
The existing production named-zone semantic gap stays until atomic integration
and required compiler, focused, pinned and broad verification complete. Source
identity generation runs only after the final actual owner graph is integrated.
