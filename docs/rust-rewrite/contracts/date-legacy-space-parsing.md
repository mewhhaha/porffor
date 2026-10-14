# Date.parse legacy space grammar

Status: source-prepared on 2026-10-13; native acceptance is pending.

Authority is ECMA-262 commit `5345883164f463e87f8b40aca4956157ecba8783`,
[Date.parse](https://tc39.es/ecma262/#sec-date.parse),
[Date Time String Format](https://tc39.es/ecma262/#sec-date-time-string-format)
and [expanded years](https://tc39.es/ecma262/#sec-expanded-years). Date.parse
first recognizes the standard format and may then use implementation-specific
formats. Unrecognizable strings and out-of-bounds fields return NaN. The pinned
`staging/sm/Date/non-iso.js` has `esid: pending`; its browser compatibility
extension is a product policy choice rather than a new normative ISO rule.

The fallback recognizes one complete grammar:

```text
year "-" month "-" day " " hour ":" minute [ ":" second [ "." millisecond ] ] [ zone ]
year        = four digits | ("+" | "-") six digits, except "-000000"
month/day/hour/minute/second = one or two ASCII decimal digits
millisecond = exactly three ASCII decimal digits
zone        = "Z" | ("+" | "-") hour2 [ minute2 | ":" minute2 ]
hour2/minute2 = exactly two ASCII decimal digits
```

The date is complete, with exactly one ASCII space before the required
hour-and-minute clock. Hours-only clocks, shortened offset digits, whitespace
variants, Unicode digit lookalikes and trailing units are rejected. Zone hours
are at most 23 and minutes at most 59. No-zone date-times use the executing
Realm's system zone and the existing compatible gap/fold inverse; explicit
zones bypass that choice. The same consuming component finalizer validates
the written calendar fields, then applies zero-only 24:00 rollover, offset
selection and TimeClip. Invalid calendar dates never become normalized dates.

The T parser remains unchanged: fixed two-digit component widths, colonized
two-part numeric offsets and exactly three fractional digits. Legacy fractions
also retain the existing three-digit policy. In particular, the pinned .01
comparison currently has NaN on both sides; its equality provides no evidence
that either fractional spelling is accepted. The repair is the finite
space-separated and short-field dates, not a broader fractional policy.

ISO, legacy space and owned-display attempts borrow the same rooted immutable
UTF-16 StringValue. Every attempt receives a fresh bounded cursor and sticky
validity state; EOF cannot read adjacent code units. One failed attempt cannot
weaken another grammar or restore an earlier invalid field. These are runtime
string operations, with no input-path or literal-source recognition.

Date.parse and the string Date constructor share this parser. Their existing
ToString/ToPrimitive ordering, conversion counts, abrupt completions and called
Realm intrinsic errors remain at the original builtin boundaries.

`aot_builtins::aot_date_legacy_space` adds three named native controls, each
executed in sloppy and strict scripts. They check finite expected epochs for
every supported pinned field spelling, hour-only/compact/colonized offsets,
extended years, positive zero, rollover and clipping; reject malformed legacy
and strict inputs; and observe completion, output, conversion counts, foreign
Realm errors and local-zone gap/fold behavior. The existing twelve Date parsing
controls and named system-zone inventory must also retain every prior pass.
The two original historical execution IDs stay in the exact-list denominator;
their unmodified replay must establish elimination independently of new tests.

No Date/Temporal task or full-suite closure is claimed by this source patch.
