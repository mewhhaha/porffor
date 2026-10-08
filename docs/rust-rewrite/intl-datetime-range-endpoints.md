# DateTimeFormat range endpoint patterns

The CLDR47 selector keeps supplied ASCII date/time alternatives for single events. English `hm` interval data uses the genuine default U+202F before AM/PM, and no `hms` interval is supplied. A range fallback must retain the genuine default endpoint pattern so a seconds-bearing range uses the same CLDR pattern family as its shorter interval.

The extractor now retains an optional `range_pattern` companion from the default resolver beside each selected alternative. Both resolvers verify the same pinned sources, and the report records the additionally consumed default leaves. The constructor admits a companion only with identical ordered fields and numbering overrides; nested companions and companions on connectors are rejected. The checked pattern owner carries both forms through field widths, hour cycles, fractional seconds, zone append items and date/time composition. Range fallback chooses the default companion before rendering. Existing interval patterns remain exact. Equal ranges use the original single-event form with all parts shared.

This batch adds three native controls and one Engine control covering two script modes. All three native controls pass in the 2026-10-01 combined checkpoint: 283 library tests and three public API tests, with zero failures and ignores. The native checks cover default spacing, equal ranges, endpoint ownership, reversed endpoints, hour cycles, fractional seconds, date/time glue, styles, zone composition and malformed companion domains. The actual receipt is `target/continuation-intl-datetime-range-native-checkpoint-attempt1/ROOT-ACTUAL-NATIVE.json`.

The isolated complete calendar successor also passes its Engine endpoint control
in strict and sloppy modes, alongside two complete-calendar Engine controls.
The actual focused receipt is
`target/continuation-intl-calendar-complete-focused-attempt1/ROOT-ACTUAL-FOCUSED.json`.
The four pinned `temporal-objects-resolved-time-zone.js` range modes, complete
workspace verification and MAIN admission remain pending. The predecessor
pooling and complete208 checkpoints retain their original scopes, failures
and receipts.
