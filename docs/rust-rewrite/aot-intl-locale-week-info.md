# Intl.Locale week information

## Source batch — 2026-10-02

This isolated source batch adds `Intl.Locale.prototype.getWeekInfo` through the
Rust compiler, emitted Wasm and pinned native Intl provider. Source compilation,
Engine regressions, exact pinned cases and broad verification remain pending.
T23 remains open. No full-suite totals or published status change follows from
this proposal.

The current normative method returns a fresh ordinary object with `firstDay`
then `weekend`, both writable, enumerable and configurable data properties.
The weekend is a fresh dense Array of ISO weekday numbers in ascending order.
There is no `minimalDays` property. Receiver branding, ignored arguments,
result construction and errors belong to the emitted JavaScript algorithm.
The result object, Array and TypeError use the method's defining Realm.

The native operation accepts only a checked canonical locale and returns
private-field checked week data. Explicit region precedes `sd`; likely subtags
and `001` supply the remaining fallback. An available canonical `rg` selects
regional data first. A recognized `fw` overrides only the first day. Both `rg`
and `sd` use the normative subdivision production; multi-subtag values and
Unicode-looking private-use text cannot become preferences.

The generated profile covers the complete pinned region domain, including
regions that inherit `001` values without their own sparse override. It derives
from the existing exact CLDR 47 supplemental data. Data and production-kernel
digests remain distinct. Provider setup validates the profile before granting
its operation, and the provider composite binds the native kernel identity.

Operation 39 extends the closed host catalog under ABI 13. A response is exactly
two little-endian u32 words: first day and the seven-bit weekend mask. Only
checked native records can create this response. The Wasm reader requires
exactly eight written bytes and validates both domains before passing a consuming
private response type to materialization. The native host neither constructs
JavaScript objects nor reads source or executes JavaScript.

## Verification still required

Native and compiler leaves have been joined. Genuine source checks reproduce
the week profile and all seven existing component identities under ABI 13;
the Duration profile generator refreshes its captured shared ListFormat input.
Ten Python profile-admission controls, formatting and module-boundary checks
pass. Independent compiler review, fresh compilation and focused execution
remain required:

```sh
cargo xc --offline --locked
cargo test --offline --locked -p lila-intl --lib provider::locale_week::
cargo test --offline --locked -p lila-intl --lib locale_week_wire::
cargo test --offline --locked -p lila-engine --test aot_intl -- aot_intl_locale_week_info:: --test-threads=2
cargo test --offline --locked -p lila-aot-wasm --test intl_temporal -- intl_namespace_plan_structure::
```

The Engine file runs each semantic control in both Script modes. Receiver
poisoning, proxy branding, metadata, descriptors, independent mutable results,
preference order, singleton and alternate weekend masks, and cross-Realm
prototypes are observable controls. Native
controls cover the full pinned data admission and extension/fallback boundaries.
The exact canonical Test262 scope `intl402/Locale/prototype/getWeekInfo` has
seven physical files and fourteen sloppy/strict executions; all require a fresh
source-bound CLI and genuine version-7 results. No previous pass is inherited.
Broad workspace, fake-suite and whole Intl/Temporal checkpoints remain required.

Other Locale info methods and broader Intl acceptance criteria remain T23 debt.
This batch does not migrate the JavaScript heap or establish weak reachability.

Normative sources: [getWeekInfo](https://tc39.es/ecma402/#sec-Intl.Locale.prototype.getWeekInfo),
[CanonicalUnicodeSubdivision](https://tc39.es/ecma402/#sec-canonicalunicodesubdivision),
[RegionPreference](https://tc39.es/ecma402/#sec-regionpreference),
[WeekInfoOfLocale](https://tc39.es/ecma402/#sec-weekinfooflocale).
