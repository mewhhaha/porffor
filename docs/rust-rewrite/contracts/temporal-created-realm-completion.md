# Temporal created-Realm completion

The 2026-10-07 continuation retains the original paired-mode fixtures. Both
newTarget and object-prototype controls pass in `tasks-remaining-native1`.
Shared PlainDate/Duration conversion exposes missing literals in the emitted
Zoned converter; both converter pools now follow the actual calendar-helper
gate during builtin discovery. `tasks-temporal-literals1` passes the workspace
check, the actual PlainDate-only Wasm emission/validation control and all five
affected/remaining native controls in both modes. Publication's earlier strict
compile exceeded the 4 GiB cap; both modes now pass in a fresh test process with
one retained native module and a 64 MiB image-cache limit. The aggregate cap,
original assertions and deadlines remain unchanged. All seven created-Realm
controls have paired-mode passes across these two checkpoints; wider T22 and
conformance acceptance remains open.

Status: integrated; all-target checking and eight compiler structure controls pass on 2026-09-30. Attempt9 passes all six paired created-Realm controls and all 50 selected Engine tests, including the additional Instant string control and repaired getter metadata. Both previously failed named-zone Instant string modes now pass. The pinned replay then stops after 133 passing physical files/266 modes because enumeration is missing before a transition-boundary loop. The subsequent 2026-10-01 enumeration checkpoint passes that complete transition-boundary loop in both modes across all 447 primary zones. The failed run is retained and the full T22 checkpoint remains open. This completion closes the missing created-Realm families discovered by the named conversion and ZonedDateTime locale controls; it does not change their Realm or option-order expectations.

Entry and created bootstrap consume one closed `TemporalIntrinsicFamily` domain and the same ordered member definitions for Instant, PlainDate, ZonedDateTime, PlainTime, PlainDateTime, PlainYearMonth, PlainMonthDay and Duration. The ZonedDateTime data methods keep their existing single IR table. Created Temporal.Now consumes the rooted namespace witness and uses created-Realm function materialization for every advertised method. Publication happens after materialization and retains the original nonenumerable, configurable namespace descriptors.

The six newly required GC pointer slots are PlainDate520, ZonedDateTime528, PlainTime536, PlainDateTime544, PlainYearMonth552 and PlainMonthDay560; the Realm intrinsic record is568 bytes. Entry and created bootstraps store every slot. Every constructor fallback passes a closed family to `RequiredResolvedRealmOrdinary`, so primitive NewTarget.prototype resolves the original NewTarget's function Realm and missing intrinsic state traps. Returned intrinsic objects use the active builtin's defining Realm; public constructor/prototype replacements do not supply those defaults. Existing representation and option/coercion order are retained.

The initial staged source closure migrated28 direct GlobalGet/LocalSet result loads and the shared partial-date intrinsic branch. Its seven emitted constructor sites covered eight families through the then-shared partial-date allocation branch. The final constructor interface below has eight explicit actual constructor producers. The existing created-Realm structure guard now proves all families, rooted Now publication, required constructor fallback and both materialization/publication order. Paired actual Engine controls pass constructor defaults, object-valued prototypes, called-Realm results, Now methods and nested-error Realm ownership. The initial descriptor failure exposed the missing partial-date getter-name prefixes; the repaired catalogue and expanded entry/created-Realm getter checks now pass in all six paired controls.

This contiguous live layout requires an explicit future PluralRules rebase: its sealed slot520 must move after the completed568-byte Temporal record. The original PluralRules and Date stage receipts remain history and cannot be applied over these overlapping sources without a reviewed derivative. No runtime count, support or broad conformance is inferred from source review, formatting or metadata checks.

The bounded constructor-prototype follow-up replaces every payload-only Temporal allocation input with `TemporalPrototypeSource`. Its constructor variant borrows the sole opaque result of the actual required `GetPrototypeFromConstructor` emission, retains both payload and representation tag, and checks the attached family against the allocator. Function, Array and Proxy prototypes therefore reach the existing GC object allocator without being relabelled as ordinary Objects. Its intrinsic variant derives the correct required prototype from the active builtin Realm inside the allocation owner. All seven allocator definitions cover the eight implemented families; all 68 production allocation calls now select the closed source (eight constructor calls, 60 intrinsic result calls). The Date-to-Instant conversion uses this same result authority. The constructor pair is allocated after retained inputs and released before them, tag before payload.

The integrated follow-up passes all-target checking. The paired object-valued NewTarget control passes both sloppy and strict execution in the root verification batch. The original 28 migrated loads/34 loader calls/7 constructor helper sites are historical receipts for the first staged completion, not the final allocation interface.

## Instant string consumer follow-up

The current pinned replay exposed the separate named-zone
`Temporal.Instant.prototype.toString` consumer: both modes of
`intl402/Temporal/Instant/prototype/toString/timezone-offset.js` failed with the
old numeric-only time-zone rejection in attempt8. The integrated typed repair retains the
resolved zone and exact rounded Instant, then projects and obtains its offset
at that rounded epoch. Historical seconds are retained in the civil fields;
`FormatDateTimeUTCOffsetRounded` supplies the separate nearest-minute suffix.
Default/undefined zone uses `Z`; explicitly supplied UTC uses `+00:00`.

The [captured Instant
algorithm](https://github.com/ptomato/ecma262/blob/3d4a6e7124a6878cb5af3132af7e01e01a88317f/temporal/instant.emu)
checks brand before options, reads fractionalSecondDigits, roundingMode,
smallestUnit and timeZone in order, then validates unit before zone conversion.
Thus a timeZone getter throw wins unit validation, and an invalid hour/date
unit wins the TypeError for a non-string zone after that Get. Primitive options
and brand errors remain defining-Realm intrinsic exceptions. Branded
ZonedDateTime zone input uses retained private slots without public getter
observation.

The existing named conversion target adds one paired fixture, bringing its
authored inventory to seven tests/14 modes. Independent vectors cover Berlin,
New York and Monrovia, negative subsecond time, spring/fall rounding across
offset changes, both Instant endpoints and fixed-zone projections beyond Date
TimeClip, exact getter/coercion order and abrupt identity, branding and borrowed
created-Realm errors. Existing six controls, helper budgets and assertions are
unchanged. All seven paired named conversion controls and both previously
failed focused pinned modes pass in attempt9. Its remaining full checkpoint
stops later on missing enumeration before a transition-boundary loop; all
failed replay evidence is retained. The subsequent enumeration checkpoint passes
six paired Engine controls and that full transition-boundary loop in both modes.
Its complete real API/consumer scope records 54/78 passes, with all 24 Runtime
Bugs owned; the five Chinese Temporal mismatch files contribute ten failed
modes. The separate smoke suite passes 191/191. The complete T22 replay and wider
calendar/Intl conformance remain open; see the [enumeration contract](../intl-supported-values.md).

The completed 2026-10-01 Buddhist predecessor checkpoint records 518/558
real modes across 279 physical files, with all 40 Runtime Bugs retained and no
exclusions. The five exact Temporal calendar-mismatch files pass 10/10 modes,
and the complete transition-boundary loop passes 2/2 across 447 primary zones.
The whole DateTimeFormat cohort records 468/496 passes (248 physical files);
the whole enumeration cohort records 38/50 passes (25 physical files). The
separate smoke suite passes 191/191 executions from 190 physical files.
Calendar and locale coverage, Chinese Temporal construction, range formatting
and missing Intl consumers retain T22/T23 ownership. These results apply to the
Buddhist predecessor, before the joined PluralRules, ListFormat, Collator,
system-zone and Tolong Siki source. That composition requires its own product
verification; the complete T22 checkpoint and full conformance remain open.
