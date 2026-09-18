# Intl.Locale likely subtags

`Intl.Locale.prototype.maximize` and `minimize` use the pinned ICU4X 2.0.0
extended CLDR 47 likely-subtag data through typed `MaximizeLocale` and
`MinimizeLocale` host operations. Minimization favors region as required by the
[ECMA-402 methods](https://tc39.es/ecma402/#sec-intl.locale.prototype.minimize).
The existing locale provider identity already includes this pinned archive and
advertises the likely-subtags capability; no data version or digest changes.

The three locale operations share validated `LocaleTransformRequest`,
`LocaleTransformResult`, and `LocaleTransformError` domains. Operation markers
retain the specific capability and wire-tag contracts. Both Wasm emission and
host request/result handling constrain the shared path to locale transforms;
a time-zone operation cannot use it. Wasm still performs JavaScript coercion,
brand checks, Realm selection, object allocation, and all observable property
operations. The provider sees validated text only.

Alias canonicalization precedes likely-subtag lookup and follows the result as
part of constructing a new Locale. ICU changes only the base language, script,
and region. Variants, transform and Unicode extension values, other extensions,
and private-use subtags retain their canonical content. Languages absent from
the pinned data remain unchanged. Reserved 5–8-letter languages stay outside
ICU lookup: their structural `und` placeholder never becomes a semantic value.

Before lookup, base script `Zzzz` and region `ZZ` become absent as required by
[UTS35 Add Likely Subtags](https://unicode.org/reports/tr35/#Likely_Subtags).
Only a complete maximal result replaces the original canonical identifier;
failed lookup retains that identifier, including its unknown fields. Unknown
codes inside transform, Unicode, and private-use extensions are unaffected.

Each method checks the receiver's Locale brand before crossing the host
boundary. It reads the internal tag, refreshes every cached component from the
canonical result, and reserves a fresh object with the executing intrinsic's
Realm prototype. The existing move-only reserved/initialized lifecycle governs
both constructor and method results. Subclasses, receiver properties, public
Intl bindings, extra arguments, and Proxy forwarding do not alter these steps.
Missing Realm bootstrap records remain internal invariant failures.

Regression coverage includes pinned likely-subtag mappings, region preference,
aliases, unknown/reserved languages, extension and variant preservation,
freshness, slot updates, method descriptors and brands, observable-read absence,
subclass behavior, and cross-Realm results and errors. Provider unit tests,
`aot_intl_locale_likely_subtags`, `intl_host_imports`, and existing lifecycle and
canonical-invocation structural targets cover the complete boundary. Verification
outcomes belong to the parent checkpoint; this contract makes no suite claim.

The frozen checkpoint7 `invalid-tag-throws.js` failure occurs in a generated
RegExp inside the unmodified canonical `testIntl.js` helper. Frozen checkpoint9
passes the directly isolated invalid Locale inputs and constructor aliases,
but reproduces that generated-pattern failure. This Locale feature does not
claim to repair the separate RegExp lowering/runtime defect.
