# Zoned Temporal arithmetic and range authority

This contract accompanies the complete named-zone compiler feature batch.
Rust compilation, emitted-Wasm execution and pinned replay remain pending.
Reference-engine observations below are preparation evidence only.

The arithmetic authority is the immutable Temporal Stage4 integration source at
`ptomato/ecma262` commit `3d4a6e7124a6878cb5af3132af7e01e01a88317f`,
[ECMA-262 PR3966](https://github.com/tc39/ecma262/pull/3966).
Downloaded source hashes are in `ecma262-stage4/FILES.json`. The rendered
proposal page is historical evidence: maintainers explain its stale rounding
text in [issue3316](https://github.com/tc39/proposal-temporal/issues/3316).

The construction range is checked against the approved Stage4 proposal source
at immutable commit `e8cc03fc970a65a3359e8870e3b35e687ac94e55`:
[ISODateTimeWithinLimits source](https://github.com/tc39/proposal-temporal/blob/e8cc03fc970a65a3359e8870e3b35e687ac94e55/spec/plaindatetime.html).
It admits the exclusive Instant interval expanded by one day. The integration
draft's first day guard drops that extra day and contradicts its own required
valid-Instant projections. We infer a transcription defect; no upstream ruling
has confirmed that inference. Existing PlainDate/PlainDateTime construction
bounds and positive extreme-offset projection controls are preserved. The
separate ValidateISODaysRange operation retains its inclusive ±100000000-day
limit at the prescribed inverse/offset interpretation steps. Investigation
sources and reference observations are retained separately under `target`.

`temporal_zoned_arithmetic.rs` owns AddZonedDateTime and the public add/subtract
entries. Zero date duration preserves the exact epoch, including a later fold.
Nonzero date duration projects through the actual retained zone, performs the
calendar addition and prescribed intermediate range check, resolves the local
date-time compatibly, then adds elapsed time. Seconds and fractional nanos
remain separate throughout the full Instant range.

The private `difference.rs` child owns DifferenceZonedDateTime and the public
until/since entries. Exact time differences need no same-zone requirement.
Date differences compare PrimaryIdentifiers after options; equal epochs return
zero before contextual probes. The raw date difference uses at most three
compatible inverse probes and preserves calendar days separately from elapsed
hours. RoundRelativeDuration is supplied by the Duration lane's private
`relative.rs` child; the parent does not duplicate its window arithmetic.

`rounding.rs` owns exact epoch rounding. String rounding mints only its private
grid proof: admitted quanta divide nsPerDay and cannot round a valid Instant
beyond either endpoint. Day rounding uses the correlated checked boundary
handle, clamps origin to end minus one nanosecond when necessary, and scales
only the bounded span. This includes post-midnight backward transitions.

The foundational provider's current GetStartOfDay validates BOTH possible-epoch
and gap-result branches. Both day endpoints therefore carry an Instant proof,
even if rounding will choose only one. The former raw-gap endpoint premise is
superseded; it must not be restored from the rendered proposal.

Prepared outputs reserve their locals below intermediate input proofs.
Intermediates release in reverse order before returning the output handle.
No consumer can manufacture an Instant, balanced ISO record, resolved zone or
validated allocation input by relabelling arbitrary local IDs.

Difference option fields are private in the existing single settings owner;
read-only accessors serve the shared algorithms. All currently admitted
calendars have proleptic Gregorian month/day arithmetic on stored ISO records.
A new calendar arithmetic domain makes this authority's destructuring fail to
compile until the implementation is extended.

Eight independent engine fixtures cover DST and half-hour transitions, later
folds, negative subsecond normalization, retained calendars and overflow,
observable coercion order, raw and rounded differences, PrimaryIdentifier and
calendar guards, skipped-day correction asymmetry and exact epoch bounds.
Their sixteen sloppy/strict observations pass in the independently downloaded
Node26.10.0 reference engine. That is differential evidence only; it is neither
Lila execution nor Test262 conformance. The archive checksum and reference
observations are saved separately under `target`.

Known upstream issue3310 leaves legal skipped-day contextual rounding windows
with coincident interpreted endpoints. The Duration lane tracks that narrow
case with a distinct T22 semantic-gap diagnostic; it is not a JavaScript throw,
passing negative test, unowned trap or reason to reject every named zone.
No full Date/Temporal, Intl or calendar conformance is claimed.
