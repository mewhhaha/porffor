# Retained zoned Duration relative arithmetic

The Wasm Duration compare, round and total entries retain the exact relativeTo
Instant, resolved zone and actual calendar. They finish observable option reads
in specification order before arithmetic. Branded relative values use private
slots; public epoch/date/time/zone getters do not replace those slots. String
shorthand starts with a fully initialized absent context.

The conversion authority exposes zoned and plain proofs only inside their
emitted kind guards. Completed Duration input and round/total option views have
private fields and sole constructors after actual conversion, receiver brand
recovery or full option validation. Shared rounding accepts the closed
Difference/Duration policy enum; date difference receives a closed largest-unit
view, rather than an interchangeable free mode/unit integer.

Zoned calendar steps use AddZonedDateTime and DifferenceZonedDateTime. Calendar
nudge and bubble endpoints each use the retained zone's compatible inverse.
CalendarDateAdd performs its prescribed date check; contextual combination and
raw AddDaysToISODate do not inject AddZonedDateTime's extra combined local check.
A zero complete start Date Duration reuses the exact origin occurrence. Time
unit totals use exact elapsed epochs. Date-unit results balance only the time
legs up to hour and copy all four actual date legs, preserving calendar days.

The common calendar projection and calendar-to-ISO steps call the
[shared pure calendar helpers](temporal-east-asian-calendar-ownership.md#shared-pure-calendar-helpers--2026-10-06).
They preserve the owned projection and completed ISO fields while sharing the
Chinese, Dangi and Umm al-Qura year algorithms across arithmetic steps. The
emitted-size control covers Duration round, total and compare and ZonedDateTime
until and since; validation and runtime acceptance of this sharing batch remain
pending.

Calendar total projection now accumulates (|innerBound|*window +
progressDistance*increment)/window as two-limb integers and rounds once to
binary64. Completed contextual endpoints are less than 200,000,004 days apart,
so each distance is below 2^74 nanoseconds. Valid date coefficients are below
2^37 and increments below 2^30; the numerator is below 2^112. A two-limb long
division retains 54 significand bits plus the exact remainder and remaining
numerator bits for nearest-even rounding. The denominator never narrows to
u64. Half-even selection uses the bucket ordinal innerBound/increment. These
repairs also apply to the existing plain relative calendar nudge. The shared raw AddDaysToISODate helper now balances civil fields without its former extra allocation range check; all prescribed CalendarDateAdd, PDT and Instant checks remain in their owning algorithms. These changes alter
emitted behavior and are not a byte-preserving refactor.

## Remaining T22 rounding-window gap

The authoritative current Stage4 text still assumes unequal contextual nudge
endpoints and a correctly signed day span. Legal 24-hour forward shifts such
as Pacific/Apia violate these assumptions. Upstream
[issue3310](https://github.com/tc39/proposal-temporal/issues/3310) remains open;
this compiler does not invent a division result or skip the case.

`RuntimeSemanticGap::TemporalZonedRoundingWindow` is T22, ABI8, and is emitted
only when an actual contextual nudge window is zero, has the wrong direction or still fails to bracket the destination after the one prescribed additionalShift recomputation.
It is a fatal compiler semantic rejection outside JavaScript completion. It
cannot be caught as RangeError, satisfy a negative exception test, or be called
an ABI/runtime crash. Wires0–7 retain their previous meanings, including T19's
RegExp gap7 and the named-zone authority gap6. Normal named-zone operations are
not blocked by this rounding diagnostic. Removing the blanket named-zone
lookup blocker belongs to the complete provider/consumer graph integration.

The exact issue reproducer (-1 day from 2012-01-01T12 in Apia) and its -25 hour companion are authored as six owned Unsupported controls: Duration.round, Duration.total and ZDT.until for each input, each in sloppy and strict mode. Adjacent supported elapsed/compare controls
remain positive. No upstream expected numeric outcome is fabricated.

## Authority and verification

Duration algorithms follow immutable ECMA-262 Stage4 PR3966 head
`3d4a6e7124a6878cb5af3132af7e01e01a88317f`, temporal/duration.emu, including
the consensus issue3316 correction to the whole start DateDuration sign.
The root's SOURCE/FILES receipts and copied stage clauses record exact bytes.

Creation and contextual PlainDateTime bounds retain the approved proposal
head `e8cc03fc970a65a3359e8870e3b35e687ac94e55`, spec/plaindatetime.html:
`|ISO epoch days| <= 100000001`, followed by the exclusive Instant±day
nanosecond interval. PlainDate applies that operation at noon. PR3966's
narrower first day check conflicts with its non-throwing creation after valid
Instant projection and the approved any-time-zone premise. Treating that
change as an integration transcription defect is a supported inference, not
a confirmed upstream ruling. The exact source/history receipts are retained
in the range investigation. The existing normalized plain Duration window
guard remains unchanged. Distinct inverse ValidateISODaysRange checks use
inclusive ±100000000 only at their own prescribed algorithm steps; they do
not supply a shadow creation or universal contextual bound.

Seven native tests are authored: six positive fixtures in twelve modes, plus
twelve exact semantic-gap observations. Node26.10.0 passes all twelve positive
reference executions; that is reference evidence only. Native Wasm tests and
the pinned27physical/54planned-execution cohort remain unexecuted. Cargo, Wasm
validation and runtime verification belong to the complete coherent batch.
No published conformance count or README number changes in this staging.
